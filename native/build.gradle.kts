import com.google.gson.JsonParser
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.util.zip.ZipFile
import kotlin.time.Duration.Companion.minutes
import kotlin.time.toJavaDuration

val CT2_INDEX = "https://pypi.org/simple/ctranslate2/"
val ct2Version = rootProject.property("ct2_version")!! as String

data class RustPlatform(
    val targetName: String,
    val systemName: String,
    val architecture: String,
    val exportedFiles: List<String>
) {
    val ct2Name =
        "${if (systemName == "osx") "macos" else systemName}-${if (architecture == "amd64") "auto64" else if (systemName == "osx") "arm64" else "aarch64"}"

    fun isHost(): Boolean {
        val hostOs = System.getProperty("os.name").lowercase()

        // is there seriously no better way to do this
        if (
            (systemName == "windows" && !hostOs.contains("windows")) ||
            (systemName == "osx" && !hostOs.contains("mac") && !hostOs.contains("darwin")) ||
            (systemName == "linux" && (!hostOs.contains("linux")))
        )
            return false

        return System.getProperty("os.arch") == this.architecture
    }
}

val RUST_TARGETS = listOf(
    RustPlatform("x86_64-apple-darwin", "osx", "x86_64", listOf("libUnityTranslateLib.dylib")), // macOS x86_64
    RustPlatform("aarch64-apple-darwin", "osx", "aarch64", listOf("libUnityTranslateLib.dylib")), // macOS arm64
    RustPlatform("aarch64-unknown-linux-gnu", "linux", "aarch64", listOf("libUnityTranslateLib.so")), // Linux aarch64
    RustPlatform("x86_64-pc-windows-msvc", "windows", "amd64", listOf("UnityTranslateLib.dll")), // Windows x86-64
    RustPlatform("x86_64-unknown-linux-gnu", "linux", "amd64", listOf("libUnityTranslateLib.so")), // Linux x86-64
)

open class ExecutableTask @Inject constructor(@Internal val execOperations: ExecOperations) : DefaultTask()

data class Platform(
    val distribution: String,
    val architecture: String,
    val outputDist: String,
    val outputArch: String
)

interface FileMatcher {
    val fileName: String

    fun match(text: String): Boolean
}

class StringBased(val text: String, override val fileName: String) : FileMatcher {
    override fun match(text: String): Boolean {
        return this.text == text
    }
}

class RegexBased(val regex: Regex, override val fileName: String) : FileMatcher {
    override fun match(text: String): Boolean {
        return regex.matches(text)
    }
}

// These files provide Python wheels, but we can technically access any one of them. We're looking for these
// files in those wheels specifically.
val ct2Files = mapOf(
    Platform("win", "amd64", "Windows", "auto64") to listOf(
        StringBased("ctranslate2/ctranslate2.dll", "ctranslate2.dll"),
        StringBased("ctranslate2/cudnn64_9.dll", "cudnn64_9.dll"),
        StringBased("ctranslate2/libiomp5md.dll", "libiomp5md.dll")
    ),
    Platform("macosx_10_13", "x86_64", "macOS", "x86_64") to listOf(
        RegexBased(Regex("ctranslate2/\\.dylibs/libctranslate2\\.\\d\\.\\d\\.\\d\\.dylib"), "libctranslate2.dylib"),
        RegexBased(Regex("ctranslate2/\\.dylibs/libiomp5\\.dylib"), "libiomp5.dylib")
    ),
    Platform("macosx_11_0", "arm64", "macOS", "arm64") to listOf(
        RegexBased(Regex("ctranslate2/\\.dylibs/libctranslate2\\.\\d\\.\\d\\.\\d\\.dylib"), "libctranslate2.dylib")
    ),
    Platform("manylinux_2_17", "x86_64.manylinux2014_x86_64", "Linux", "auto64") to listOf(
        RegexBased(Regex("ctranslate2\\.libs/libctranslate2-\\w+\\.so\\.\\d\\.\\d\\.\\d"), "libctranslate2.so"),
        RegexBased(Regex("ctranslate2\\.libs/libcudnn-\\w+\\.so\\.\\d\\.\\d\\.\\d"), "libcudnn.so"),
        RegexBased(Regex("ctranslate2\\.libs/libgomp-\\w+\\.so\\.\\d\\.\\d\\.\\d"), "libgomp.so"),
    ),
    Platform("manylinux_2_17", "aarch64.manylinux2014_aarch64", "Linux", "aarch64") to listOf(
        RegexBased(Regex("ctranslate2\\.libs/libctranslate2-\\w+\\.so\\.\\d\\.\\d\\.\\d"), "libctranslate2.so"),
        RegexBased(Regex("ctranslate2\\.libs/libgomp-\\w+\\.so\\.\\d\\.\\d\\.\\d"), "libgomp.so"),
    )
)

tasks {
    create("downloadCTranslate2") {
        doFirst {
            val ct2Dir = layout.buildDirectory.get().dir("ctranslate2")
            if (!ct2Dir.asFile.exists())
                ct2Dir.asFile.mkdirs()

            val versionedDir = ct2Dir.dir(ct2Version)
            if (!versionedDir.asFile.exists())
                versionedDir.asFile.mkdirs()

            val client = HttpClient.newBuilder()
                .connectTimeout(5.minutes.toJavaDuration())
                .build()
            val request = HttpRequest.newBuilder(URI.create(CT2_INDEX))
                .GET()
                .header("Accept", "application/vnd.pypi.simple.v1+json")
                .build()

            val response = client.send(request, HttpResponse.BodyHandlers.ofString())
            val json = JsonParser.parseString(response.body())
                .asJsonObject
                .getAsJsonArray("files")

            for ((platform, fileMatchers) in ct2Files) {
                val platformDir = versionedDir.dir("${platform.outputDist}-${platform.outputArch}".lowercase())
                val file =
                    versionedDir.file("ctranslate2-${ct2Version}-cp39-cp39-${platform.distribution}_${platform.architecture}.whl").asFile

                if (file.exists())
                    continue
                else
                    file.createNewFile()

                if (!platformDir.asFile.exists())
                    platformDir.asFile.mkdirs()
                println("Downloading: ${"ctranslate2-${ct2Version}-cp39-cp39-${platform.distribution}_${platform.architecture}.whl"}")

                val url = URI.create(
                    json.first {
                        it.asJsonObject.get("filename").asString == "ctranslate2-${ct2Version}-cp39-cp39-${platform.distribution}_${platform.architecture}.whl"
                    }
                        .asJsonObject.get("url").asString
                )
                    .toURL()

                logger.info("Downloading CTranslate2 wheels for ${platform.outputDist} (${platform.outputArch})...")
                file.outputStream().use { f ->
                    url.openStream().use {
                        it.transferTo(f)
                    }
                }

                logger.info("Finished downloading! Getting required files...")

                val wheelZip = ZipFile(file)
                for (entry in wheelZip.entries()) {
                    val matchedFile = fileMatchers.firstOrNull { it.match(entry.name) } ?: continue
                    val filePath = platformDir.file(matchedFile.fileName).asFile

                    if (filePath.exists()) {
                        logger.info("File ${matchedFile.fileName} already exists, skipping.")
                        continue
                    } else
                        filePath.createNewFile()

                    logger.info("Extracting file ${matchedFile.fileName}...")
                    wheelZip.getInputStream(entry).use {
                        filePath.outputStream().use { f ->
                            it.transferTo(f)
                        }
                    }
                    logger.info("Extracted file ${matchedFile.fileName}!")
                }
            }
        }
    }

    create<ExecutableTask>("rustBuild") {
        outputs.upToDateWhen { false }

        doFirst {
            for (target in RUST_TARGETS) {
                if (!target.isHost()) // TODO: figure out cross-compilation.
                    continue

                execOperations.exec {
                    commandLine(if (target.isHost()) "cargo" else "cross")

                    val args = mutableListOf(
                        "build", "--release",
                        "--target", target.targetName,
                        "--package", "UnityTranslateLib",
                        "--lib"
                    )

                    if (target.systemName == "osx") {
                        args.add("-F")
                        args.add("accelerate")
                    }

                    args(args)
                    standardOutput = System.out
                }
                    .assertNormalExitValue()
            }
        }
    }

    create("moveTargetFiles") {
        dependsOn("rustBuild", "downloadCTranslate2")

        doFirst {
            val nativesDir = layout.buildDirectory.get().dir("ut_natives")
            if (!nativesDir.asFile.exists())
                nativesDir.asFile.mkdirs()

            val ct2Dir = layout.buildDirectory.get().dir("ctranslate2").dir(ct2Version)
            val targetsDir = layout.projectDirectory.dir("target")
            for (target in RUST_TARGETS) {
                val utNativesDir = nativesDir.dir("unitytranslate").dir("${target.systemName}-${target.architecture}")

                if (!target.isHost())
                    continue

                if (!utNativesDir.asFile.exists())
                    utNativesDir.asFile.mkdirs()

                val libDir = ct2Dir.dir(target.ct2Name)
                val libFile = libDir.asFile
                val files = libFile.listFiles().map { it.name }

                for (fileName in files) {
                    val srcFile = libDir.file(fileName).asFile
                    val file = utNativesDir.file(fileName).asFile
                    if (!file.exists())
                        file.createNewFile()

                    srcFile.copyTo(file, true)
                }

                val targetDir = targetsDir.dir(target.targetName).dir("release")
                for (fileName in target.exportedFiles) {
                    val srcFile = targetDir.file(fileName).asFile
                    val file = utNativesDir.file(fileName).asFile
                    if (!file.exists())
                        file.createNewFile()

                    srcFile.copyTo(file, true)
                }
            }
        }
    }

    processResources {
        dependsOn("downloadCTranslate2", "rustBuild", "moveTargetFiles")

        from(layout.buildDirectory.get().dir("ut_natives")).into("natives")
    }

    jar {
        val target = RUST_TARGETS.first { it.isHost() }

        archiveBaseName.set("UnityTranslateLib-natives-${target.systemName}-${target.architecture}")
    }
}

publishing {
    val target = RUST_TARGETS.first { it.isHost() }

    publications {
        register("maven", MavenPublication::class) {
            groupId = "xyz.bluspring.unitytranslate"
            artifactId = "UnityTranslateLib-natives-${target.systemName}-${target.architecture}"
            version = "${rootProject.property("unitytranslate_version")}"
            from(components.getByName("java"))
        }
    }
}