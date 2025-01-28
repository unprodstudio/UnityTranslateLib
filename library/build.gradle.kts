plugins {
    kotlin("jvm") version "2.1.0"
    kotlin("plugin.serialization") version "2.1.0"
    id("com.gradleup.shadow") version "8.3.5"
}

group = "xyz.bluspring"
version = "0.2.0"

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

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(17)
}