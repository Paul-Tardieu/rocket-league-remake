plugins {
    id("carball.java-conventions")
    application
}

dependencies {
    implementation(project(":bot"))
}

application {
    mainClass = "io.github.paultrde.carball.render.Main"
}
