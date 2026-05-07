plugins {
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    `maven-publish`
}

group = "xyz.bluspring.unitytranslate"
version = "${rootProject.property("unitytranslate_version")}"

base {
    archivesName.set("unitytranslate-library")
}

repositories {
    mavenCentral()
}

dependencies {
    api(libs.bundles.kotlin)
    api(libs.slf4j.api)
    testRuntimeOnly(libs.slf4j.simple)
}

java {
    withSourcesJar()
}

kotlin {
    jvmToolchain(17)
}

val xmake = natives {
    path = projectDir.toPath().resolve("jni")

    platform("windows", "x64")

    // TODO: fix for every other platform
//    platform("windows", "arm64")
//    platform("windows", "arm64ec")

//    platform("linux", "x86_64")
//    platform("linux", "armv7")
//    platform("linux", "armv7s")
//    platform("linux", "arm64")

//    platform("mac", "x86_64")
//    platform("mac", "arm64")
}

tasks {
    test {
        useJUnitPlatform()
    }
}

publishing {
    repositories {
        maven("https://mvn.devos.one/releases") {
            credentials {
                username = System.getenv()["MAVEN_USER"]
                password = System.getenv()["MAVEN_PASS"]
            }
        }
    }

    publications {
        register("maven", MavenPublication::class) {
            groupId = "xyz.bluspring.unitytranslate"
            artifactId = "unitytranslate-library"
            version = "${rootProject.version}"
            from(components.getByName("java"))

            for (platform in xmake.platformTaskNames) {
                artifact(tasks.getByName("nativesJar$platform"))
            }
        }
    }
}