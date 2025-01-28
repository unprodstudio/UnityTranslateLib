plugins {
    id("maven-publish")
}

subprojects {
    apply(plugin = "java")
    apply(plugin = "maven-publish")

    group = "xyz.bluspring"
    version = "${rootProject.property("unitytranslate_version")}"

    repositories {
        mavenCentral()
    }

    publishing {
        repositories {
            maven("https://mvn.devos.one/snapshots") {
                credentials {
                    username = System.getenv()["MAVEN_USER"]
                    password = System.getenv()["MAVEN_PASS"]
                }
            }
        }
        publications {
            register("maven", MavenPublication::class) {
                groupId = "xyz.bluspring"
                artifactId = "UnityTranslateLib"
                version = "${rootProject.property("unitytranslate_version")}"
                from(components.getByName("java"))
            }
        }
    }
}