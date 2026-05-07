import org.gradle.api.Project
import org.gradle.jvm.tasks.Jar
import java.nio.file.Path
import java.util.Locale

fun String.uppercaseFirstChar(): String =
    replaceFirstChar { it.uppercase(Locale.US) }

class XmakeSetup(val project: Project) {
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

        project.tasks.register<XmakeCompileTask>("compileCpp$taskName", XmakeCompileTask::class.java) {
            it.workingDir(path)
            it.platform = platform
            it.arch = arch
            it.extraArgs.addAll(extraArgs)
        }

        project.tasks.register<Jar>("nativesJar$taskName", Jar::class.java) {
            it.dependsOn("compileCpp$taskName")
            it.group = "build"

            it.into("unitytranslate/$platform/$arch") {
                it.from(path.resolve("build/install/$platform/$arch/"))
            }

            it.archiveClassifier.set("natives-$platform-$arch")
        }
    }
}

fun Project.natives(setup: XmakeSetup.() -> Unit): XmakeSetup {
    val xmake = XmakeSetup(this)
    setup.invoke(xmake)
    return xmake
}
