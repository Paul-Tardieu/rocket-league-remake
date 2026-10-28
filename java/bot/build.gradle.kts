// Bots only see the public sim API: GameState in, CarInput out.
plugins {
    id("carball.java-conventions")
    `java-library`
}

dependencies {
    api(project(":sim"))
}
