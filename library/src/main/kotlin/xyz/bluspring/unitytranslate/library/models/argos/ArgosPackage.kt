package xyz.bluspring.unitytranslate.library.models.argos

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import xyz.bluspring.unitytranslate.library.models.ModelPackage

@Serializable
data class ArgosPackage(
    @SerialName("package_version")
    val packageVersion: String,
    @SerialName("argos_version")
    val argosVersion: String,

    @SerialName("from_code")
    override val fromCode: String,
    @SerialName("from_name")
    override val fromName: String,

    @SerialName("to_code")
    override val toCode: String,
    @SerialName("to_name")
    override val toName: String,

    @SerialName("links")
    val links: List<String>,

    @SerialName("code")
    override val code: String
) : ModelPackage
