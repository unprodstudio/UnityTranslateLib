import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

@DisableCachingByDefault
abstract class RustCompileTask : Exec() {
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

        commandLine("cargo", "build", "--profile", "release", "--package", "unitytranslatelib", *extraArgs.toTypedArray())
        super.exec()
    }
}
