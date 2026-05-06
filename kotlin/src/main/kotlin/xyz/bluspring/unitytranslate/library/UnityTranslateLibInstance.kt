package xyz.bluspring.unitytranslate.library

import xyz.bluspring.unitytranslate.library.util.LangPair

class UnityTranslateLibInstance(val library: UnityTranslateLib, val langPair: LangPair, val handle: Long) {
    private var isFreed = false

    private fun ensureAllocated() {
        if (this.isFreed)
            throw IllegalStateException("Translator instance $langPair was already freed!")
    }

    fun batchTranslate(text: List<String>): List<String> {
        this.ensureAllocated()

        val original = text.toTypedArray()
        val results = text.toTypedArray()

        this.library.batchTranslate(this.handle, original, results)
        return results.toList()
    }

    fun free() {
        this.ensureAllocated()
        this.library.freeInstance(this.handle)
        this.isFreed = true
    }
}
