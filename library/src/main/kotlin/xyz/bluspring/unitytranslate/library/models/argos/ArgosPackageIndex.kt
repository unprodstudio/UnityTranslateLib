package xyz.bluspring.unitytranslate.library.models.argos

import kotlinx.serialization.json.Json
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import xyz.bluspring.unitytranslate.library.models.ModelInfo
import xyz.bluspring.unitytranslate.library.models.PackageIndex
import java.net.URI
import java.nio.file.Path
import java.util.zip.ZipFile
import kotlin.time.Duration.Companion.days

class ArgosPackageIndex(path: Path) : PackageIndex<ArgosPackage>(path, "argos") {
    private val json = Json {
        ignoreUnknownKeys = true
    }

    private var lastIndexTime = 0L

    override fun loadIndex() {
        val url = URI.create(PACKAGE_INDEX_URL).toURL()
        val text = url.readText(Charsets.UTF_8)
        loadIndexFromString(text)

        val cachedFile = path.resolve("index.json")
        if (!cachedFile.exists())
            cachedFile.createNewFile()

        cachedFile.writeText(text, Charsets.UTF_8)
        lastIndexTime = System.currentTimeMillis()
    }

    private fun loadIndexFromString(data: String) {
        val indexData = json.decodeFromString<List<ArgosPackage>>(data)

        this.packages.clear()
        this.packages.addAll(indexData)
    }

    override fun loadIndexOrCache() {
        val cachedFile = path.resolve("index.json")

        if (lastIndexTime == 0L && cachedFile.exists()) {
            lastIndexTime = cachedFile.lastModified()
        }

        if (System.currentTimeMillis() - lastIndexTime >= 1.days.inWholeMilliseconds) {
            try {
                UnityTranslateLib.logger.debug("Cache outdated! Updating Argos index.")
                loadIndex()
            } catch (e: Exception) {
                if (cachedFile.exists())
                    loadIndexFromString(cachedFile.readText(Charsets.UTF_8))
                else {
                    UnityTranslateLib.logger.debug("Failed to update Argos index, and no cached index could be found! $e")
                    e.printStackTrace()
                }
            }
        } else if (cachedFile.exists()) {
            loadIndexFromString(cachedFile.readText(Charsets.UTF_8))
        }
    }

    override suspend fun getOrDownloadModelInfo(pkg: ArgosPackage): ModelInfo {
        val pkgDir = path.resolve("${pkg.code}_${pkg.packageVersion}")

        if (pkgDir.exists()) {
            return ModelInfo(
                pkg.code,
                pkgDir.resolve("model").toPath(),
                pkgDir.resolve("bpe.model").run { if (this.exists()) this else null }?.toPath(),
                pkgDir.resolve("sentencepiece.model").run { if (this.exists()) this else null }?.toPath()
            )
        }

        val exception = RuntimeException("Failed to download Argos models for ${pkg.code} v${pkg.packageVersion}!")

        for (link in pkg.links) {
            try {
                val url = URI.create(link).toURL()
                val zipPath = path.resolve("${pkg.code}_${pkg.packageVersion}.argosmodel")
                url.openStream().use {
                    zipPath.outputStream().use { o ->
                        it.copyTo(o)
                    }
                }

                val zipFile = ZipFile(zipPath)
                for (entry in zipFile.entries()) {
                    if (entry.name.endsWith("/"))
                        continue

                    val file = pkgDir.resolve(entry.name.replaceBefore("/", "").replaceFirst("/", ""))
                    if (!file.parentFile.exists())
                        file.parentFile.mkdirs()

                    file.createNewFile()
                    zipFile.getInputStream(entry).use {
                        file.outputStream().use { f ->
                            it.copyTo(f)
                        }
                    }
                }

                zipPath.delete()

                return ModelInfo(
                    pkg.code,
                    pkgDir.resolve("model").toPath(),
                    pkgDir.resolve("bpe.model").run { if (this.exists()) this else null }?.toPath(),
                    pkgDir.resolve("sentencepiece.model").run { if (this.exists()) this else null }?.toPath()
                )
            } catch (e: Exception) {
                exception.addSuppressed(RuntimeException("Failed to download from URL $link", e))
            }
        }

        throw exception
    }

    companion object {
        const val PACKAGE_INDEX_ROOT_URL = "https://raw.githubusercontent.com/argosopentech/argospm-index/main"
        const val PACKAGE_INDEX_URL = "$PACKAGE_INDEX_ROOT_URL/index.json"
    }
}