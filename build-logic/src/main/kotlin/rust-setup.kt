import org.gradle.api.Project
import org.gradle.jvm.tasks.Jar
import org.gradle.kotlin.dsl.get
import java.nio.file.Path
import java.util.*

fun String.uppercaseFirstChar(): String =
    replaceFirstChar { it.uppercase(Locale.US) }

class RustSetup(val project: Project, val libraryName: String) {
    lateinit var path: Path
    val platforms: MutableList<String> = mutableListOf()

    val platformTaskNames: List<String>
        get() = platforms.map {
            val b = it.split("-")
            "${b[0].uppercaseFirstChar()}${b[1].uppercaseFirstChar()}"
        }

    fun platform(platform: String, arch: String, extraArgs: List<String> = listOf()) {
        val taskName = "${platform.uppercaseFirstChar()}${arch.uppercaseFirstChar()}"
        platforms.add("$platform-$arch")

        project.tasks.register("compileRust$taskName", RustCompileTask::class.java) {
            workingDir(this@RustSetup.path)
            this.platform = platform
            this.arch = arch
            this.extraArgs.addAll(extraArgs)
            this.libraryName = "starmedia_natives"
        }

        val platformTask = project.tasks.register("nativesJar$taskName", Jar::class.java) {
            dependsOn("compileRust$taskName")
            group = "build"

            into("$libraryName/$platform/$arch") {
                val path = this@RustSetup.path.resolve("target/release/${System.mapLibraryName(libraryName)}")
                println(path)
                from(path)
            }

            archiveClassifier.set("natives-$platform-$arch")
        }

        project.tasks["jar"]
            .dependsOn(platformTask.get())
    }
}

fun Project.natives(libraryName: String, setup: RustSetup.() -> Unit): RustSetup {
    val rust = RustSetup(this, libraryName)
    setup.invoke(rust)
    return rust
}
