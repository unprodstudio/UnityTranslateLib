import org.gradle.kotlin.dsl.support.uppercaseFirstChar

plugins {
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.shadow)
    `maven-publish`
}

group = "xyz.bluspring.unitytranslate"
version = "${rootProject.property("unitytranslate_version")}"

base {
    archivesName.set("UnityTranslateLib")
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

val platforms = listOf(
    "windows-x64",
//    "windows-arm64",
//    "windows-arm64ec",
    "linux-x86_64",
    "linux-armv7",
    "linux-armv7s",
    "linux-arm64",
    "mac-x86_64",
    "mac-arm64"
)

tasks {
    test {
        useJUnitPlatform()
    }

    register<XmakeCompileTask>("compileCppWindowsX64") {
        workingDir("${project.projectDir}/jni")
        platform = "windows"
        arch = "x64"
    }

    // not supported by sentencepiece (gperftools)
//    register<XmakeCompileTask>("compileCppWindowsArm64") {
//        workingDir("${project.projectDir}/jni")
//        platform = "windows"
//        arch = "arm64"
//    }
//
//    register<XmakeCompileTask>("compileCppWindowsArm64ec") {
//        workingDir("${project.projectDir}/jni")
//        platform = "windows"
//        arch = "arm64ec"
//    }

    register<XmakeCompileTask>("compileCppLinuxX86_64") {
        workingDir("${project.projectDir}/jni")
        platform = "linux"
        arch = "x86_64"
    }

    register<XmakeCompileTask>("compileCppLinuxArmv7") {
        workingDir("${project.projectDir}/jni")
        platform = "linux"
        arch = "armv7"
    }

    register<XmakeCompileTask>("compileCppLinuxArmv7s") {
        workingDir("${project.projectDir}/jni")
        platform = "linux"
        arch = "armv7s"
    }

    register<XmakeCompileTask>("compileCppLinuxArm64") {
        workingDir("${project.projectDir}/jni")
        platform = "linux"
        arch = "arm64"
    }

    register<XmakeCompileTask>("compileCppMacX86_64") {
        workingDir("${project.projectDir}/jni")
        platform = "macosx"
        arch = "x86_64"
    }

    register<XmakeCompileTask>("compileCppMacArm64") {
        workingDir("${project.projectDir}/jni")
        platform = "macosx"
        arch = "arm64"
    }

    register("compileAllCpp") {
        dependsOn(platforms.map {
            "compileCpp" + it.split("-").joinToString("") { b -> b.uppercaseFirstChar() }
        }.toTypedArray())
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
            artifactId = "UnityTranslateLib"
            version = "${rootProject.version}"
            from(components.getByName("java"))
        }
    }
}