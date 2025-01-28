package xyz.bluspring.unitytranslate.library

import org.jetbrains.annotations.ApiStatus
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.bluspring.unitytranslate.library.models.ModelPackageManager
import xyz.bluspring.unitytranslate.library.models.argos.ArgosPackageIndex
import xyz.bluspring.unitytranslate.library.translator.DummyTranslator
import xyz.bluspring.unitytranslate.library.translator.ModelBasedTranslator
import xyz.bluspring.unitytranslate.library.translator.Translator
import java.nio.file.Path
import java.util.concurrent.ConcurrentHashMap
import kotlin.io.path.Path
import kotlin.io.path.absolutePathString

class UnityTranslateLib(val path: Path) {
    val packageIndex = ModelPackageManager(this).apply {
        registerIndex(ArgosPackageIndex(path))
    }

    val translators = ConcurrentHashMap<String, Translator>()

    fun load() {
        packageIndex.load()
    }

    private suspend fun createTranslator(code: String): Translator {
        val split = code.split("_")

        val translator = if (split[0] == split[1])
            // Just passthrough if they are the same
            DummyTranslator(this, code)
        else
            ModelBasedTranslator(this, code)

        translator.load()

        return translator
    }

    suspend fun getTranslator(fromCode: String, toCode: String): Translator {
        return getTranslator("${fromCode}_${toCode}")
    }

    suspend fun getTranslator(code: String): Translator {
        if (!translators.containsKey(code))
            translators[code] = createTranslator(code)

        return translators[code]!!
    }

    @ApiStatus.Internal
    external fun loadModel(modelPath: String, spModelPath: String?, bpeModelPath: String?, useCuda: Boolean): Long

    @ApiStatus.Internal
    external fun batchTranslate(modelPtr: Long, textArray: Array<String>): Array<String>

    companion object {
        val logger: Logger = LoggerFactory.getLogger(UnityTranslateLib::class.java)

        init {
            System.load(Path("UnityTranslateLibRsAgain.dll").toAbsolutePath().absolutePathString())
        }
    }
}