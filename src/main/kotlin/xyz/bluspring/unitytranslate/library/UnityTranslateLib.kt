package xyz.bluspring.unitytranslate.library

import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.bluspring.unitytranslate.library.util.LangPair
import xyz.bluspring.unitytranslate.library.util.TokenizerType
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import kotlin.io.path.Path
import kotlin.io.path.absolutePathString
import kotlin.io.path.exists

/**
 * [path] - Specifies the path where UnityTranslateLib will download files such as translation models to.
 */
class UnityTranslateLib(val path: Path) {
    fun createInstance(lang: LangPair, type: TokenizerType, tokenizerPath: Path, translatorPath: Path, useCuda: Boolean): UnityTranslateLibInstance {
        return UnityTranslateLibInstance(this, lang, createInstance(lang.toCode, translatorPath.absolutePathString(), type.ordinal, tokenizerPath.absolutePathString(), useCuda))
    }

    private external fun createInstance(toLang: String, translatorModelPath: String, type: Int, tokenizerModelPath: String, useCuda: Boolean): Long
    internal external fun batchTranslate(instance: Long, textToTranslate: Array<String>, results: Array<String>)
    internal external fun freeInstance(instance: Long)

    companion object {
        val logger: Logger = LoggerFactory.getLogger(UnityTranslateLib::class.java)
        private val platformLibs: List<String>
            get() {
                val osName = System.getProperty("os.name").lowercase()
                val isWindows = osName.contains("win")
                val isMac = osName.contains("mac")

                val osArch = System.getProperty("os.arch").lowercase().run {
                    if (isWindows && this == "amd64")
                        "x64"
                    else this
                }

                val dir = "unitytranslate/${if (isWindows) "windows" else if (isMac) "osx" else "linux"}/${osArch}"

                return if (osArch == "x64") {
                    if (isWindows)
                        listOf(
                            "$dir/bin/UnityTranslateLib.dll",
                            "$dir/bin/ctranslate2.dll",
                            "$dir/bin/re2.dll",
//                            "$dir/bin/cudnn64_9.dll",
                            "$dir/bin/libiomp5md.dll",
                        )
                    else if (isMac)
                        listOf()
                    else
                        listOf(
                            "$dir/bin/libUnityTranslateLib.so",
                            "$dir/bin/libctranslate2.so",
                            "$dir/bin/libcudnn.so",
                            "$dir/bin/libgomp.so"
                        )
                } else emptyList()
            }
        private val cachedPlatformLibs = platformLibs

        // Modified from ImGui-java's library loading - https://github.com/SpaiR/imgui-java/blob/main/imgui-binding/src/main/java/imgui/ImGui.java
        fun autoLoad() {
            if (isAvailable()) {
                val libPath = System.getProperty("unitytranslate.library.path")
                val libName = System.getProperty("unitytranslate.library.name", "UnityTranslateLib")
                val fullLibName = resolveFullLibName()

                if (libPath != null) {
                    System.load(Path(libPath).resolve(fullLibName).absolutePathString())
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

                        val osName = System.getProperty("os.name").lowercase()
                        val isWindows = osName.contains("win")
                        val isMac = osName.contains("mac")

                        val osArch = System.getProperty("os.arch").lowercase().run {
                            if (isWindows && this == "amd64")
                                "x64"
                            else this
                        }

                        val dir = "unitytranslate/${if (isWindows) "windows" else if (isMac) "osx" else "linux"}/${osArch}/bin/"

                        for (lib in platformLibs.reversed()) {
                            System.load(extractedPath.resolve(lib.removePrefix(dir)).absolutePathString())
                        }
                    }
                }
            } else {
                logger.warn("UnityTranslateLib is unsupported on platform ${System.getProperty("os.name")} (${System.getProperty("os.arch")})!")
                logger.warn("As a result, UnityTranslateLib will not be translating, and may cause errors if any native calls are attempted.")
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

        private fun tryLoadFromClassPath(fullLibName: String): Path {
            val classLoader = UnityTranslateLib::class.java.classLoader
            val libs = platformLibs

            if (libs.isEmpty())
                throw Exception("Unsupported platform ${System.getProperty("os.name")} (${System.getProperty("os.arch")})!")

            val tmpDir = Path(System.getProperty("java.io.tmpdir")).resolve("unitytranslate-natives")

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
                throw Exception("Failed to extract library files for UnityTranslateLib!")

            return tmpDir
        }

        @JvmStatic
        fun isAvailable(): Boolean {
            return cachedPlatformLibs.isNotEmpty()
        }
    }
}
