package xyz.bluspring.unitytranslate.library.translator

import xyz.bluspring.unitytranslate.library.UnityTranslateLib

class ModelBasedTranslator(library: UnityTranslateLib, code: String) : Translator(library, code) {
    lateinit var modelPtrs: Map<String, Long>
    var isReady = false
        private set

    override suspend fun load(useCuda: Boolean) {
        if (isReady)
            return

        modelPtrs = library.packageIndex.tryLoadModels(code, useCuda)
        isReady = true
    }

    override fun batchTranslate(texts: List<String>): List<String> {
        if (!isReady) {
            UnityTranslateLib.logger.warn("Model $code is not ready!")
            return texts
        }

        val translatedTexts = mutableListOf<String>().apply {
            this.addAll(texts)
        }

        for ((code, modelPtr) in modelPtrs) {
            val translated = library.batchTranslate(modelPtr, translatedTexts.toTypedArray())
            translatedTexts.clear()
            translatedTexts.addAll(translated.map { it.trim() })
        }

        return translatedTexts
    }
}