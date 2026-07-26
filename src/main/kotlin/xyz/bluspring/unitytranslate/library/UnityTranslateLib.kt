package xyz.bluspring.unitytranslate.library

import org.jetbrains.annotations.ApiStatus
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.bluspring.unitytranslate.library.util.LangPair
import xyz.bluspring.unitytranslate.library.util.TokenizerType
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import kotlin.io.path.Path
import kotlin.io.path.absolutePathString
import kotlin.io.path.createDirectories
import kotlin.io.path.exists

class UnityTranslateLib {
    fun createInstance(lang: LangPair, type: TokenizerType, tokenizerPath: Path, translatorPath: Path, useCuda: Boolean): UnityTranslateLibInstance {
        if (!tokenizerPath.exists())
            throw IllegalArgumentException("Could not find tokenizer path ${tokenizerPath.absolutePathString()}!")

        if (!translatorPath.exists())
            throw IllegalArgumentException("Could not find translator path ${tokenizerPath.absolutePathString()}!")

        return UnityTranslateLibInstance(this, lang, createInstance(lang.fromCode, lang.toCode, translatorPath.absolutePathString(), type.ordinal, tokenizerPath.absolutePathString(), useCuda))
    }

    private external fun createInstance(fromLang: String, toLang: String, translatorModelPath: String, type: Int, tokenizerModelPath: String, useCuda: Boolean): Long
    @ApiStatus.Internal
    external fun batchTranslate(instance: Long, textToTranslate: Array<String>, results: Array<String>)
    @ApiStatus.Internal
    external fun freeInstance(instance: Long)

    companion object {
        val logger: Logger = LoggerFactory.getLogger(UnityTranslateLib::class.java)

        private var isLoaded = false

        fun autoLoad() {
            try {
                this.autoLoadOrThrow()
            } catch (e: Throwable) {
                logger.error("Failed to load UnityTranslateLib!", e)
                logger.warn("UnityTranslateLib may not be supported on platform ${System.getProperty("os.name")} (${System.getProperty("os.arch")})!")
                logger.warn("As a result, UnityTranslateLib will not be translating, and may cause errors if any native calls are attempted.")
            }
        }

        // Modified from ImGui-java's library loading - https://github.com/SpaiR/imgui-java/blob/main/imgui-binding/src/main/java/imgui/ImGui.java
        fun autoLoadOrThrow() {
            if (this.isLoaded)
                return

            val libPath = System.getProperty("unitytranslate.library.path")
            val fullLibName = System.getProperty("unitytranslate.library.name", System.mapLibraryName("unitytranslatelib"))

            if (libPath != null) {
                System.load(Path(libPath).resolve(fullLibName).absolutePathString())
            } else {
                try {
                    System.loadLibrary(fullLibName)
                } catch (e: Throwable) {
                    val extractedPath = try {
                        tryLoadFromClassPath(fullLibName)
                    } catch (e2: Exception) {
                        val joined = RuntimeException("Failed to load natives for UnityTranslateLib!")
                        joined.addSuppressed(e2)
                        joined.addSuppressed(e)

                        throw joined
                    }

                    System.load(extractedPath.resolve(fullLibName).absolutePathString())
                }
            }

            this.isLoaded = true
        }

        private fun tryLoadFromClassPath(fullLibName: String): Path {
            val classLoader = UnityTranslateLib::class.java.classLoader

            val tmpDir = Path(System.getProperty("java.io.tmpdir")).resolve("unitytranslate-natives")

            if (!tmpDir.exists())
                tmpDir.createDirectories()

            val osName = System.getProperty("os.name").lowercase()
            val isWindows = osName.contains("win")
            val isMac = osName.contains("mac")

            val osArch = System.getProperty("os.arch").lowercase().run {
                if (isWindows && this == "amd64")
                    "x64"
                else this
            }

            val dir = "unitytranslate/${if (isWindows) "windows" else if (isMac) "osx" else "linux"}/${osArch}"

            classLoader.getResourceAsStream("$dir/$fullLibName")?.use {
                val libPath = tmpDir.resolve(fullLibName)
                try {
                    Files.copy(it, libPath, StandardCopyOption.REPLACE_EXISTING)
                } catch (e: AccessDeniedException) {
                    if (!libPath.exists())
                        throw e
                }
            }

            val unityTranslatePath = tmpDir.resolve(fullLibName)
            if (!unityTranslatePath.exists())
                throw Exception("Failed to extract library files for UnityTranslateLib!")

            return tmpDir
        }

        @JvmStatic
        fun isAvailable(): Boolean {
            this.autoLoad()
            return this.isLoaded
        }
    }
}
