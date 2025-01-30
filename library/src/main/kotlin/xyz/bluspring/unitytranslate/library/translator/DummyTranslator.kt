package xyz.bluspring.unitytranslate.library.translator

import xyz.bluspring.unitytranslate.library.UnityTranslateLib

class DummyTranslator(library: UnityTranslateLib, code: String) : Translator(library, code) {
    override suspend fun load(useCuda: Boolean) {
    }

    override fun batchTranslate(texts: List<String>): List<String> {
        return texts
    }
}