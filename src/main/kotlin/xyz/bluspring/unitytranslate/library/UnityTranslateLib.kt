package xyz.bluspring.unitytranslate.library

import org.jetbrains.annotations.ApiStatus
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.bluspring.unitytranslate.library.util.LangPair
import xyz.bluspring.unitytranslate.library.util.TokenizerType
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.StandardOpenOption
import java.security.DigestInputStream
import java.security.MessageDigest
import kotlin.io.path.*

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

        private var hasTriedLoading = false
        private var isLoaded = false

        fun autoLoad() {
            try {
                this.autoLoadOrThrow()
            } catch (e: Throwable) {
                logger.error("Failed to load UnityTranslateLib!", e)
                logger.warn("UnityTranslateLib may not be supported on platform ${System.getProperty("os.name")} (${System.getProperty("os.arch")})!")
                logger.warn("As a result, UnityTranslateLib will not be translating, and may cause errors if any native calls are attempted.")
            } finally {
                this.hasTriedLoading = true
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

            val osName = System.getProperty("os.name").lowercase()
            val isWindows = osName.contains("win")
            val isMac = osName.contains("mac")

            val osArch = System.getProperty("os.arch").lowercase().run {
                if (isWindows && this == "amd64")
                    "x64"
                else this
            }

            val dir = "unitytranslate/${if (isWindows) "windows" else if (isMac) "osx" else "linux"}/${osArch}"

            val libraryResource = classLoader.getResource("$dir/$fullLibName")
                ?: throw Exception("Could not locate UnityTranslateLib natives for platform $osName $osArch!")

            val tmpDir = Path(System.getProperty("java.io.tmpdir")).resolve("unitytranslate-natives")

            if (!tmpDir.exists())
                tmpDir.createDirectories()

            // first, try to hash the file
            val digest = MessageDigest.getInstance("MD5")
            libraryResource.openStream().use {
                DigestInputStream(it, digest).readAllBytes()
            }

            val expectedHash = digest.digest().toHexString()
            val hashedTmpDir = tmpDir.resolve(expectedHash)
            if (!hashedTmpDir.exists())
                hashedTmpDir.createDirectories()

            val unityTranslateLibPath = hashedTmpDir.resolve(fullLibName)
            val lockFile = hashedTmpDir.resolve("session.lock")

            if (unityTranslateLibPath.exists()) {
                // hash the file and see if it matches
                unityTranslateLibPath.inputStream(StandardOpenOption.READ).use {
                    DigestInputStream(it, digest).readAllBytes()
                }

                // hash matches, we're good
                if (expectedHash == digest.digest().toHexString())
                    return hashedTmpDir

                // nope, let's see if someone's currently copying it.
                // if so, we should block the thread until the lock is invalid.
                if (lockFile.exists()) {
                    val pid = lockFile.readText().trim().toLongOrNull()
                    if (pid != null && ProcessHandle.of(pid).isPresent) {
                        while (true) {
                            // check if the lock file still exists
                            val pid = if (lockFile.exists()) {
                                lockFile.readText().trim().toLongOrNull()
                            } else break

                            // also check if the process still exists
                            if (pid == null || ProcessHandle.of(pid).isEmpty)
                                break

                            // let's not check too frequently...
                            Thread.sleep(2_500L)
                        }

                        // okay, let's check again just to be safe.
                        unityTranslateLibPath.inputStream(StandardOpenOption.READ).use {
                            DigestInputStream(it, digest).readAllBytes()
                        }

                        if (expectedHash == digest.digest().toHexString())
                            return hashedTmpDir
                    }

                    // fuck, okay let's continue extracting us I guess, we assume the program crashed or failed or something.
                }
            }

            libraryResource.openStream().use {
                try {
                    // we want to make sure we're not copying all at once.
                    lockFile.writeText("${ProcessHandle.current().pid()}", options = arrayOf(StandardOpenOption.WRITE, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING))

                    Files.copy(it, unityTranslateLibPath, StandardCopyOption.REPLACE_EXISTING)
                } catch (e: AccessDeniedException) {
                    if (!unityTranslateLibPath.exists())
                        throw e
                } finally {
                    // okay, we're done here.
                    lockFile.deleteIfExists()
                }
            }

            if (!unityTranslateLibPath.exists())
                throw Exception("Failed to extract library files for UnityTranslateLib!")
            else {
                unityTranslateLibPath.inputStream(StandardOpenOption.READ).use {
                    DigestInputStream(it, digest).readAllBytes()
                }

                // uh oh
                val actualHash = digest.digest().toHexString()
                if (expectedHash != actualHash)
                    throw IllegalStateException("Extracted library hash for UnityTranslateLib does not match! (expected: $expectedHash, got: $actualHash)")
            }

            return hashedTmpDir
        }

        @JvmStatic
        fun isAvailable(): Boolean {
            if (!this.hasTriedLoading) {
                this.autoLoad()
            }

            return this.isLoaded
        }
    }
}
