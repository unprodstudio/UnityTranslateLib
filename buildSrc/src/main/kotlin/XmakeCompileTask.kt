import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

@DisableCachingByDefault
abstract class XmakeCompileTask : Exec() {
    companion object {
        var llvmPath = "C:\\Program Files\\LLVM"
        var llvmMingWPath = "C:\\llvm-mingw"
    }

    @Input
    var platform: String = "windows"

    @Input
    var arch: String = "x64"

    @Input
    var extraArgs: MutableList<String> = mutableListOf()

    @TaskAction
    override fun exec() {
        val currentOs = OperatingSystem.type
        val extraArgs = this.extraArgs.toMutableList()
        var platform = this.platform

        commandLine("xmake", "f", "-y", "-c", "-p", platform, "-a", arch, "-m", "release", *extraArgs.toTypedArray())
        super.exec()
        commandLine("xmake", "install", "-y", "-o", "build/install/${this.platform}/${this.arch}")
        super.exec()
    }
}