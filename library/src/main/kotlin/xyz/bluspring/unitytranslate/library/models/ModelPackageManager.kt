package xyz.bluspring.unitytranslate.library.models

import kotlinx.coroutines.flow.asFlow
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import xyz.bluspring.unitytranslate.library.util.collect
import xyz.bluspring.unitytranslate.library.util.concurrent
import java.util.concurrent.ConcurrentHashMap
import kotlin.io.path.absolutePathString

class ModelPackageManager(val library: UnityTranslateLib) {
    val indexList = mutableListOf<PackageIndex<*>>()
    private val loadedModelPtrs = ConcurrentHashMap<String, Long>()

    val packages: List<ModelPackage>
        get() {
            return indexList.map { it.packages }.flatten()
        }

    fun <T : ModelPackage> registerIndex(index: PackageIndex<T>) {
        indexList.add(index)
        UnityTranslateLib.logger.debug("Registered model index for ${index.name}.")
    }

    fun load() {
        for (index in indexList) {
            index.loadIndex()
        }
    }

    fun getTranslationPackages(fromLang: String, toLang: String): List<ModelPackage> {
        for (index in indexList) {
            val available = index.getTranslationPackage(fromLang, toLang)
            if (available.isNotEmpty())
                return available
        }

        return emptyList()
    }

    fun getAvailableModelInfos(fromLang: String, toLang: String): Map<out ModelPackage, ModelInfo> {
        for (index in indexList) {
            val available = index.getAvailableModelInfos(fromLang, toLang)
            if (available.isNotEmpty())
                return available
        }

        return mapOf()
    }

    fun isModelAvailable(fromLang: String, toLang: String): Boolean {
        for (index in indexList) {
            if (index.isModelAvailable(fromLang, toLang))
                return true
        }

        return false
    }

    suspend fun tryDownloadModelInfos(fromLang: String, toLang: String): Map<out ModelPackage, ModelInfo> {
        for (index in indexList) {
            val available = index.tryDownloadModelInfos(fromLang, toLang)
            if (available.isNotEmpty())
                return available
        }

        return mapOf()
    }

    suspend fun tryLoadModels(code: String, useCuda: Boolean): Map<String, Long> {
        val split = code.split("_")
        val fromCode = split[0]
        val toCode = split[1]
        return tryLoadModels(fromCode, toCode, useCuda)
    }

    suspend fun tryLoadModels(fromCode: String, toCode: String, useCuda: Boolean): Map<String, Long> {
        val modelInfos = this.getAvailableModelInfos(fromCode, toCode)

        if (modelInfos.isEmpty())
            throw Exception("No translation models available for $fromCode-$toCode!")

        val infos = mutableMapOf<ModelPackage, ModelInfo>()

        for ((pkg, modelInfo) in modelInfos) {
            if (loadedModelPtrs.containsKey(pkg.code))
                continue

            infos[pkg] = modelInfo
        }

        infos.toList().asFlow().concurrent().collect { (pkg, modelInfo) ->
            val modelPtr = library.loadModel(modelInfo.code.split("_")[1], modelInfo.modelPath.absolutePathString(), modelInfo.spModelPath?.absolutePathString(), modelInfo.bpeModelPath?.absolutePathString(), useCuda)

            if (modelPtr != 0L) {
                loadedModelPtrs[pkg.code] = modelPtr
            } else {
                throw IllegalStateException("Failed to load model ${modelInfo.code} (${modelInfo.modelPath.absolutePathString()})!")
            }
        }

        val modelPtrs = mutableMapOf<String, Long>()
        for ((pkg, _) in modelInfos) {
            modelPtrs[pkg.code] = loadedModelPtrs[pkg.code]!!
        }

        return modelPtrs
    }
}