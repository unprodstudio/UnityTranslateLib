package xyz.bluspring.unitytranslate.library.models.argos

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromStream
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import xyz.bluspring.unitytranslate.library.models.ModelInfo
import xyz.bluspring.unitytranslate.library.models.PackageIndex
import xyz.bluspring.unitytranslate.library.util.DownloadHelper
import java.io.InputStream
import java.net.URI
import java.nio.file.Path
import java.util.zip.ZipFile
import kotlin.io.path.exists
import kotlin.io.resolve
import kotlin.time.Duration.Companion.days

class ArgosPackageIndex(path: Path) : PackageIndex<ArgosPackage>(path, "argos") {
    private val json = Json {
        ignoreUnknownKeys = true
    }

    private var lastIndexTime = 0L

    override fun loadIndex() {
        val url = URI.create(PACKAGE_INDEX_URL).toURL()
        url.openStream().use { loadIndexFromStream(it, true) }

        lastIndexTime = System.currentTimeMillis()
    }

    @OptIn(ExperimentalSerializationApi::class)
    private fun loadIndexFromStream(stream: InputStream, cache: Boolean = false) {
        val indexData = json.decodeFromStream<List<ArgosPackage>>(stream)

        this.packages.clear()
        this.packages.addAll(indexData)

        if (cache) {
            val cachedFile = path.resolve("index.json")
            if (!cachedFile.exists())
                cachedFile.createNewFile()

            cachedFile.writeText(json.encodeToString(indexData), Charsets.UTF_8)
        }
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
                    cachedFile.inputStream().use { loadIndexFromStream(it) }
                else {
                    UnityTranslateLib.logger.debug("Failed to update Argos index, and no cached index could be found! $e")
                    e.printStackTrace()
                }
            }
        } else if (cachedFile.exists()) {
            cachedFile.inputStream().use { loadIndexFromStream(it) }
        }
    }

    override fun getAvailableModelInfo(pkg: ArgosPackage): ModelInfo? {
        val pkgDir = path.resolve("${pkg.code}_${pkg.packageVersion}")
        val downloadId = "argos_${pkg.code}"

        // A download is running, so it's not available at the moment.
        if (DownloadHelper.getDownloadInfo(downloadId) != null)
            return null

        if (pkgDir.exists())
            return createModelInfo(pkg, pkgDir.toPath())

        return null
    }

    override suspend fun tryDownloadModelInfo(pkg: ArgosPackage): ModelInfo {
        val pkgDir = path.resolve("${pkg.code}_${pkg.packageVersion}")
        val downloadId = "argos_${pkg.code}"

        if (DownloadHelper.getDownloadInfo(downloadId) != null) {
            while (DownloadHelper.getDownloadInfo(downloadId) != null) {
                // just keep blocking I guess? idk how else to handle this
            }
        }

        if (pkgDir.exists()) {
            return createModelInfo(pkg, pkgDir.toPath())
        }

        val exception = RuntimeException("Failed to download Argos models for ${pkg.code} v${pkg.packageVersion}!")

        for (link in pkg.links) {
            try {
                val url = URI.create(link).toURL()
                val zipPath = path.resolve("${pkg.code}_${pkg.packageVersion}.argosmodel")

                DownloadHelper.download(downloadId, url.openConnection(), zipPath)

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

                return createModelInfo(pkg, pkgDir.toPath())
            } catch (e: Exception) {
                exception.addSuppressed(RuntimeException("Failed to download from URL $link", e))
            }
        }

        throw exception
    }

    private fun createModelInfo(pkg: ArgosPackage, pkgDir: Path): ModelInfo {
        return ModelInfo(
            pkg.code,
            pkgDir.resolve("model"),
            pkgDir.resolve("bpe.model").run { if (this.exists()) this else null },
            pkgDir.resolve("sentencepiece.model").run { if (this.exists()) this else null }
        )
    }

    companion object {
        const val PACKAGE_INDEX_ROOT_URL = "https://raw.githubusercontent.com/argosopentech/argospm-index/main"
        const val PACKAGE_INDEX_URL = "$PACKAGE_INDEX_ROOT_URL/index.json"
    }
}