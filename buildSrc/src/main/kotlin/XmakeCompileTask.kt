import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

@DisableCachingByDefault
abstract class XmakeCompileTask : Exec() {
    @Input
    var platform: String = "windows"

    @Input
    var arch: String = "x64"

    @Input
    var extraArgs: MutableList<String> = mutableListOf()

    @TaskAction
    override fun exec() {
        commandLine("xmake", "f", "-y", "-c", "-p", platform, "-a", arch, "-m", "release")
        commandLine.addAll(extraArgs)
        super.exec()
        commandLine("xmake", "-y")
        super.exec()
    }
}