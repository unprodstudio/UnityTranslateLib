package xyz.bluspring.unitytranslate.library.util

import java.io.File
import java.net.URLConnection
import java.util.*

object DownloadHelper {
    val runningDownloads: MutableList<DownloadInfo> = Collections.synchronizedList(mutableListOf<DownloadInfo>())

    fun getDownloadInfo(id: String): DownloadInfo? {
        synchronized(runningDownloads) {
            return runningDownloads.firstOrNull { it.id == id }
        }
    }

    fun getAllDownloadsById(id: String): List<DownloadInfo> {
        synchronized(runningDownloads) {
            return runningDownloads.filter { it.id == id }
        }
    }

    fun download(id: String, connection: URLConnection, output: File) {
        val tempOutput = output.parentFile.resolve("${output.name}.tmp")
        val info = DownloadInfo(id, connection, tempOutput, output)

        if (tempOutput.exists())
            tempOutput.delete()

        runningDownloads.add(info)
        try {
            connection.inputStream.use {
                tempOutput.outputStream().use { o ->
                    it.copyTo(o)
                }
            }

            if (output.exists())
                output.delete()

            tempOutput.renameTo(output)
        } finally {
            runningDownloads.remove(info)
        }
    }

    data class DownloadInfo(val id: String, private val connection: URLConnection, private val tempOutput: File, private val output: File) {
        val totalLength: Long
            get() {
                return connection.contentLengthLong
            }

        val currentLength: Long
            get() {
                return if (tempOutput.exists())
                    tempOutput.length()
                else
                    output.length()
            }
    }
}