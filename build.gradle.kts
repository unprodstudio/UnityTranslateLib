plugins {
    id("maven-publish")
}

subprojects {
    group = "xyz.bluspring"
    version = "0.2.0"

    repositories {
        mavenCentral()
    }
}