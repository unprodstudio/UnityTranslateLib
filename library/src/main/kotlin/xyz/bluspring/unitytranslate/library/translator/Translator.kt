package xyz.bluspring.unitytranslate.library.translator

import xyz.bluspring.unitytranslate.library.UnityTranslateLib

abstract class Translator(protected val library: UnityTranslateLib, protected val code: String) {
    abstract suspend fun load(useCuda: Boolean)
    abstract fun batchTranslate(texts: List<String>): List<String>
}