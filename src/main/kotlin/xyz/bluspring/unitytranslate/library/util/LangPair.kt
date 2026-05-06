package xyz.bluspring.unitytranslate.library.util

@JvmRecord
data class LangPair(
    val fromCode: String,
    val toCode: String
) {
    override fun toString(): String {
        return "${fromCode}_${toCode}"
    }
}
