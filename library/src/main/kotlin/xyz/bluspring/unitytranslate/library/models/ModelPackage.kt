package xyz.bluspring.unitytranslate.library.models

interface ModelPackage {
    val fromCode: String
    val fromName: String
    val toCode: String
    val toName: String

    val code: String
}