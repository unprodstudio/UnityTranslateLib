package xyz.bluspring.unitytranslate.library.models

import kotlinx.coroutines.flow.asFlow
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import xyz.bluspring.unitytranslate.library.util.collect
import xyz.bluspring.unitytranslate.library.util.concurrent
import java.util.concurrent.ConcurrentHashMap
import kotlin.io.path.absolutePathString

class ModelPackageManager(val library: UnityTranslateLib) {
    private val indexList = mutableListOf<PackageIndex<*>>()
    private val loadedModelPtrs = ConcurrentHashMap<String, Long>()

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

    suspend fun getModelInfos(fromLang: String, toLang: String): Map<out ModelPackage, ModelInfo> {
        for (index in indexList) {
            val available = index.getOrDownloadModelInfos(fromLang, toLang)
            if (available.isNotEmpty())
                return available
        }

        return mapOf()
    }

    suspend fun tryLoadModels(code: String): Map<String, Long> {
        val split = code.split("_")
        val fromCode = split[0]
        val toCode = split[1]
        return tryLoadModels(fromCode, toCode)
    }

    suspend fun tryLoadModels(fromCode: String, toCode: String): Map<String, Long> {
        val modelInfos = this.getModelInfos(fromCode, toCode)

        if (modelInfos.isEmpty())
            throw Exception("No translation models available for $fromCode-$toCode!")

        val infos = mutableMapOf<ModelPackage, ModelInfo>()

        for ((pkg, modelInfo) in modelInfos) {
            if (loadedModelPtrs.containsKey(pkg.code))
                continue

            infos[pkg] = modelInfo
        }

        infos.toList().asFlow().concurrent().collect { (pkg, modelInfo) ->
            val modelPtr = library.loadModel(modelInfo.modelPath.absolutePathString(), modelInfo.spModelPath?.absolutePathString(), modelInfo.bpeModelPath?.absolutePathString(), false)

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