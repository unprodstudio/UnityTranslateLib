object OperatingSystem {
    enum class Type(val formatted: String) {
        WINDOWS("Windows"),
        MAC("macOS"),
        LINUX("Linux"),
        OTHER("(unknown)");
    }

    val type: Type = System.getProperty("os.name", "generic").lowercase().run {
        if (indexOf("mac") >= 0 || indexOf("darwin") >= 0)
            Type.MAC
        else if (indexOf("win") >= 0)
            Type.WINDOWS
        else if (indexOf("nux") >= 0)
            Type.LINUX
        else
            Type.OTHER
    }
}
