package xyz.bluspring.unitytranslate.library.test

import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import xyz.bluspring.unitytranslate.library.UnityTranslateLibInstance
import xyz.bluspring.unitytranslate.library.util.LangPair
import xyz.bluspring.unitytranslate.library.util.TokenizerType
import java.util.*
import java.util.concurrent.atomic.AtomicInteger
import kotlin.io.path.*
import kotlin.test.Test
import kotlin.test.assertEquals

class UnityTranslateLibTests {
    val library = UnityTranslateLib()
    val texts = listOf(
        "Hello, welcome to Unity Multiplayer, where everyone is absolutely bloody deranged and we love it.",
        "I'm just having fun here :D"
    )
    val languages = listOf("es", "sv", "da")
    val langToInstances: MutableMap<String, UnityTranslateLibInstance> = Collections.synchronizedMap(mutableMapOf())

    init {
        UnityTranslateLib.autoLoad()
        var startTime = System.currentTimeMillis()
        val modelsPath = Path("build/models/v1").toAbsolutePath()
        val toLangMap = modelsPath.listDirectoryEntries().filter { it.isDirectory() }
            .associate {
                val toLang = it.name.removePrefix("translate-en_").replace(Regex("-\\d+_\\d+"), "")
                toLang to it.resolve("en_$toLang")
            }

        runBlocking {
            for (lang in languages) {
                val file = toLangMap[lang]!!
                val tokenizerType = if (file.resolve("sentencepiece.model").exists())
                    TokenizerType.SENTENCEPIECE
                else
                    TokenizerType.BPE

                launch {
                    langToInstances[lang] = library.createInstance(LangPair("en", lang), tokenizerType, file.resolve("${tokenizerType.name.lowercase()}.model"), file.resolve("model"), false)
                }
            }
        }

        println("took ${System.currentTimeMillis() - startTime}ms to initialize translators")
    }

    val expectedTranslations = mapOf(
        "es" to listOf(
            "Hola, bienvenido a Unity Multiplayer, donde todo el mundo está absolutamente maldito y nos encanta.",
            "Me estoy divirtiendo aquí :D"
        ),
        "sv" to listOf(
            "Hej, välkommen till Unity Multiplayer, där alla är helt blodiga derangerade och vi älskar det.",
            "Jag har bara kul här :D"
        ),
        "de" to listOf(
            "Hallo, willkommen bei Unity Multiplayer, wo jeder absolut verdammt verwirrt ist und wir lieben es.",
            "Ich habe nur Spaß hier :D"
        ),
        "da" to listOf(
            "Velkommen til Unity Multiplayer, hvor alle er fuldstændig sindssyge, og vi elsker det.",
            "Jeg har det bare sjovt her: D"
        )
    )

    @Test
    fun testTranslate() {
        val totalTranslated = AtomicInteger(0)

        runBlocking {
            for (lang in languages) {
                launch {
                    val startTime = System.currentTimeMillis()

                    val translator = langToInstances[lang]!!
                    val translated = translator.batchTranslate(texts)

                    for ((index, string) in translated.withIndex()) {
                        assertEquals(expectedTranslations[lang]!![index], string)
                    }

                    println("$lang - took ${System.currentTimeMillis() - startTime}ms")

                    totalTranslated.incrementAndGet()
                }
            }
        }

        assertEquals(languages.size, totalTranslated.get())
    }
}
