plugins {
    kotlin("jvm") version "2.1.0"
    kotlin("plugin.serialization") version "2.1.0"
}

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.1")
    api("org.jetbrains.kotlinx:kotlinx-coroutines-core-jvm:1.10.1")
    api("org.jetbrains.kotlinx:kotlinx-coroutines-jdk8:1.10.1")
    api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.0-RC")
    api("org.slf4j:slf4j-api:2.0.16")

    testRuntimeOnly("org.slf4j:slf4j-simple:2.0.16")
}

publishing {
    publications {
        register("maven", MavenPublication::class) {
            groupId = "xyz.bluspring.unitytranslate"
            artifactId = "UnityTranslateLib"
            version = "${rootProject.property("unitytranslate_version")}"
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
    jvmToolchain(8)
}