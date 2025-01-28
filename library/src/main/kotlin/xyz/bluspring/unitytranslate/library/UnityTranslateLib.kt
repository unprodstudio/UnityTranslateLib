package xyz.bluspring.unitytranslate.library

import org.jetbrains.annotations.ApiStatus
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.bluspring.unitytranslate.library.models.ModelPackageManager
import xyz.bluspring.unitytranslate.library.models.argos.ArgosPackageIndex
import xyz.bluspring.unitytranslate.library.translator.DummyTranslator
import xyz.bluspring.unitytranslate.library.translator.ModelBasedTranslator
import xyz.bluspring.unitytranslate.library.translator.Translator
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.nio.file.StandardCopyOption
import java.util.concurrent.ConcurrentHashMap
import kotlin.io.path.absolutePathString
import kotlin.io.path.exists

class UnityTranslateLib(val path: Path) {
    val packageIndex = ModelPackageManager(this).apply {
        registerIndex(ArgosPackageIndex(path))
    }

    val translators = ConcurrentHashMap<String, Translator>()

    fun load() {
        packageIndex.load()
    }

    private suspend fun createTranslator(code: String): Translator {
        val split = code.split("_")

        val translator = if (split[0] == split[1])
            // Just passthrough if they are the same
            DummyTranslator(this, code)
        else
            ModelBasedTranslator(this, code)

        translator.load()

        return translator
    }

    suspend fun getTranslator(fromCode: String, toCode: String): Translator {
        return getTranslator("${fromCode}_${toCode}")
    }

    suspend fun getTranslator(code: String): Translator {
        if (!translators.containsKey(code))
            translators[code] = createTranslator(code)

        return translators[code]!!
    }

    @ApiStatus.Internal
    external fun loadModel(modelPath: String, spModelPath: String?, bpeModelPath: String?, useCuda: Boolean): Long

    @ApiStatus.Internal
    external fun batchTranslate(modelPtr: Long, textArray: Array<String>): Array<String>

    companion object {
        val logger: Logger = LoggerFactory.getLogger(UnityTranslateLib::class.java)

        // Modified from ImGui-java's library loading - https://github.com/SpaiR/imgui-java/blob/main/imgui-binding/src/main/java/imgui/ImGui.java
        init {
            val libPath = System.getProperty("unitytranslate.library.path")
            val libName = System.getProperty("unitytranslate.library.name", "UnityTranslateLib")
            val fullLibName = resolveFullLibName()

            if (libPath != null) {
                System.load(Paths.get(libPath).resolve(fullLibName).absolutePathString())
            } else {
                try {
                    System.loadLibrary(libName)
                } catch (e: Throwable) {
                    val extractedPath = try {
                        tryLoadFromClassPath(fullLibName)
                    } catch (e2: Exception) {
                        val joined = RuntimeException("Failed to load natives for UnityTranslateLib!")
                        joined.addSuppressed(e2)
                        joined.addSuppressed(e)

                        throw joined
                    }

                    System.load(extractedPath)
                }
            }
        }

        private fun resolveFullLibName(): String {
            val osName = System.getProperty("os.name").lowercase()
            val isWindows = osName.contains("win")
            val isMac = osName.contains("mac")

            val libPrefix = if (isWindows) "" else "lib"
            val libSuffix = if (isWindows) ".dll" else if (isMac) ".dylib" else ".so"

            return System.getProperty("unitytranslate.library.name", "${libPrefix}UnityTranslateLib${libSuffix}")
        }

        private val platformLibs: List<String>
            get() {
                val osName = System.getProperty("os.name").lowercase()
                val osArch = System.getProperty("os.arch").lowercase()
                val isWindows = osName.contains("win")
                val isMac = osName.contains("mac")

                val dir = "unitytranslate/${if (isWindows) "windows" else if (isMac) "osx" else "linux"}-${osArch}"

                return if (osArch == "amd64") {
                    if (isWindows)
                        listOf(
                            "$dir/UnityTranslateLib.dll",
                            "$dir/ctranslate2.dll",
                            "$dir/cudnn64_9.dll",
                            "$dir/libiomp5md.dll"
                        )
                    else if (isMac)
                        listOf()
                    else
                        listOf(
                            "$dir/libctranslate2.so",
                            "$dir/libcudnn.so",
                            "$dir/libgomp.so",
                            "$dir/libUnityTranslateLib.so"
                        )
                } else emptyList()
            }

        private fun tryLoadFromClassPath(fullLibName: String): String {
            val classLoader = UnityTranslateLib::class.java.classLoader
            val libs = platformLibs

            if (libs.isEmpty())
                throw Exception("Unsupported platform ${System.getProperty("os.name")} (${System.getProperty("os.arch")})!")

            val tmpDir = Paths.get(System.getProperty("java.io.tmpdir")).resolve("unitytranslate-natives")

            if (!tmpDir.exists())
                tmpDir.toFile().mkdirs()

            for (packedLibPath in libs) {
                val libName = packedLibPath.split("/").last()

                classLoader.getResourceAsStream(packedLibPath)?.use {
                    val libPath = tmpDir.resolve(libName)
                    try {
                        Files.copy(it, libPath, StandardCopyOption.REPLACE_EXISTING)
                    } catch (e: AccessDeniedException) {
                        if (!libPath.exists())
                            throw e
                    }
                }
            }

            val unityTranslatePath = tmpDir.resolve(fullLibName)
            if (!unityTranslatePath.exists())
                throw Exception("Failed to load library files for UnityTranslateLib!")

            return unityTranslatePath.absolutePathString()
        }
    }
}