import java.net.URI
import java.util.zip.ZipFile

plugins {
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    `maven-publish`
    id("rust-setup")
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
    testImplementation(kotlin("test"))

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

val rust = natives("unitytranslatelib") {
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
        doFirst {
            val esModel = URI.create("https://argos-net.com/v1/translate-en_es-1_0.argosmodel").toURL()
            val daModel = URI.create("https://argos-net.com/v1/translate-en_da-1_9.argosmodel").toURL()
            val svModel = URI.create("https://argos-net.com/v1/translate-en_sv-1_5.argosmodel").toURL()

            val models = listOf(esModel, daModel, svModel)

            for (modelUrl in models) {
                val name = modelUrl.file
                val filePath = project.layout.buildDirectory.asFile.get().resolve(("models/$name"))
                val dirPath = project.layout.buildDirectory.asFile.get().resolve(("models/${name.removeSuffix(".argosmodel")}"))

                if (dirPath.exists())
                    continue

                if (!filePath.exists() && !dirPath.exists()) {
                    modelUrl.openStream().use {
                        filePath.outputStream().use { out ->
                            it.transferTo(out)
                        }
                    }
                }

                filePath.parentFile.mkdirs()
                filePath.createNewFile()

                dirPath.mkdirs()

                val zipFile = ZipFile(filePath)
                for (entry in zipFile.stream()) {
                    val file = dirPath.resolve(entry.name)
                    if (file.exists())
                        continue

                    if (entry.isDirectory)
                        continue

                    file.parentFile.mkdirs()
                    file.createNewFile()

                    val inputStream = zipFile.getInputStream(entry)
                    inputStream.use {
                        it.transferTo(file.outputStream())
                    }
                }
            }
        }

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

            for (platform in rust.platformTaskNames) {
                artifact(tasks.getByName("nativesJar$platform"))
            }
        }
    }
}