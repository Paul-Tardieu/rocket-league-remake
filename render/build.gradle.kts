plugins {
    id("carball.java-conventions")
    application
}

dependencies {
    implementation(project(":sim"))
}

application {
    mainClass = "io.github.paultrde.carball.render.Main"
}
