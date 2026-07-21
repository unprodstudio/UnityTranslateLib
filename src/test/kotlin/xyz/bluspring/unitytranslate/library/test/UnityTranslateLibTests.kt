package xyz.bluspring.unitytranslate.library.test

import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import xyz.bluspring.unitytranslate.library.UnityTranslateLib
import java.util.concurrent.atomic.AtomicInteger
import kotlin.io.path.Path
import kotlin.test.Test

class UnityTranslateLibTests {
    val library = UnityTranslateLib(Path(".").toAbsolutePath())
    val texts = listOf(
        "Hello, welcome to Unity Multiplayer, where everyone is absolutely bloody deranged and we love it.",
        "I'm just having fun here :D"
    )
    val languages = listOf("es", "sv", "de")

    init {
        UnityTranslateLib.autoLoad()
        var startTime = System.currentTimeMillis()
        runBlocking {
            for (lang in languages) {
                launch {
//                    library.getTranslator("en", lang)
                }
            }
        }

        println("took ${System.currentTimeMillis() - startTime}ms to initialize translators")
    }

    val expectedTranslations = mapOf(
        "en" to texts,
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
        )
    )

    @Test
    fun testTranslate() {
        val totalTranslated = AtomicInteger(0)

//        runBlocking {
//            for (lang in languages) {
//                launch {
//                    val startTime = System.currentTimeMillis()
//
//                    val translator = library.getTranslator("en", lang)
//                    val translated = translator.batchTranslate(texts)
//
//                    for ((index, string) in translated.withIndex()) {
//                        assertEquals(expectedTranslations[lang]!![index], string)
//                    }
//
//                    println("$lang - took ${System.currentTimeMillis() - startTime}ms")
//
//                    totalTranslated.incrementAndGet()
//                }
//            }
//        }
//
//        assertEquals(languages.size, totalTranslated.get())
    }
}
