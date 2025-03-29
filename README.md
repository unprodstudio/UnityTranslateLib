# UnityTranslateLib

A Java/Kotlin library using [CTranslate2](https://github.com/OpenNMT/CTranslate2) to handle text translation natively
within [UnityTranslate](https://github.com/UnityMultiplayer/UnityTranslate).

Based on [Argos Translate](https://github.com/argosopentech/argos-translate/) and its models.

## Installation

**Gradle**
```groovy
repositories {
    maven {
        url = uri("https://mvn.devos.one/snapshots")
    }
}

dependencies {
    // You can check the gradle.properties for the current version.
    def utVersion = "<VERSION>" 
    
    implementation("xyz.bluspring.unitytranslate:UnityTranslateLib:$utVersion")
    implementation("xyz.bluspring.unitytranslate:UnityTranslateLib-natives-windows-amd64:$utVersion")
    implementation("xyz.bluspring.unitytranslate:UnityTranslateLib-natives-linux-amd64:$utVersion")
}
```