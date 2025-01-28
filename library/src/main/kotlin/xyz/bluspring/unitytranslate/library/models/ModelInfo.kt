package xyz.bluspring.unitytranslate.library.models

import java.nio.file.Path

data class ModelInfo(
    val code: String,
    val modelPath: Path,
    val bpeModelPath: Path?,
    val spModelPath: Path?
)
