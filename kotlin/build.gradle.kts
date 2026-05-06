plugins {
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.shadow)
}

base {
    archivesName.set("UnityTranslateLib")
}

dependencies {
    api(libs.bundles.kotlin)
    api(libs.slf4j.api)
    testRuntimeOnly(libs.slf4j.simple)
}

publishing {
    publications {
        register("maven", MavenPublication::class) {
            groupId = "xyz.bluspring.unitytranslate"
            artifactId = "UnityTranslateLib"
            version = "${rootProject.version}"
            from(components.getByName("java"))
        }
    }
}

tasks.test {
    useJUnitPlatform()
}

java {
    withSourcesJar()
}

kotlin {
    jvmToolchain(17)
}