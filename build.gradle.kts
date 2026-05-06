plugins {
    `maven-publish`
}

allprojects {
    group = "xyz.bluspring.unitytranslate"
    version = "${rootProject.property("unitytranslate_version")}"

    repositories {
        mavenCentral()
    }
}

subprojects {
    apply(plugin = "java")
    apply(plugin = "maven-publish")

    publishing {
        repositories {
            maven("https://mvn.devos.one/snapshots") {
                credentials {
                    username = System.getenv()["MAVEN_USER"]
                    password = System.getenv()["MAVEN_PASS"]
                }
            }
        }
    }
}