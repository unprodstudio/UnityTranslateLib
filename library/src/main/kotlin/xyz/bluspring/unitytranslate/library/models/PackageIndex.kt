package xyz.bluspring.unitytranslate.library.models

import java.io.File
import java.nio.file.Path

abstract class PackageIndex<T : ModelPackage>(path: Path, val name: String) {
    val path = File(path.toFile(), name).apply {
        if (!this.exists())
            this.mkdirs()
    }
    val packages = mutableListOf<T>()

    abstract fun loadIndex()
    abstract fun loadIndexOrCache()
    abstract suspend fun getOrDownloadModelInfo(pkg: T): ModelInfo

    open suspend fun getOrDownloadModelInfos(fromLang: String, toLang: String): Map<T, ModelInfo> {
        val packages = this.getTranslationPackage(fromLang, toLang)
        if (packages.isEmpty())
            return mapOf()

        val modelInfos = mutableMapOf<T, ModelInfo>()
        for (pkg in packages) {
            modelInfos[pkg] = getOrDownloadModelInfo(pkg)
        }

        return modelInfos
    }

    open fun getTranslationPackage(fromLang: String, toLang: String): List<T> {
        if (this.packages.isEmpty())
            this.loadIndexOrCache()

        // Check if there's a direct translation
        if (this.packages.any { it.fromCode == fromLang && it.toCode == toLang }) {
            return listOf(this.packages.first { it.fromCode == fromLang && it.toCode == toLang })
        }

        // If not, see if it's possible to do translations via another language
        val fromLanguages = this.packages.filter { it.fromCode == fromLang }
        val toLanguages = this.packages.filter { it.toCode == toLang }

        if (fromLanguages.isEmpty() || toLanguages.isEmpty())
            return emptyList()

        val possibleLanguages = toLanguages.filter { fromLanguages.any { a -> a.toCode == it.fromCode } }

        if (possibleLanguages.isEmpty())
            return emptyList()

        val toLanguage = possibleLanguages.first()
        val fromLanguage = fromLanguages.first { it.toCode == toLanguage.fromCode }

        return listOf(fromLanguage, toLanguage)
    }
}