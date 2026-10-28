# Carball

Jeu de foot en voiture minimaliste et original, conçu pour entraîner des bots (apprentissage par renforcement).

- `rust/` : **version active**, avec vue 3D, boost, vitesse max, sauts, double saut et flips.
- `java/` : premier prototype (vue 2D de dessus), conservé pour référence.
- `training/` : entraînement de bots pour le vrai Rocket League (RLGym v2 + RocketSim), voir [training/README.md](training/README.md).

## Lancer le jeu

Prérequis : Rust ([rustup](https://rustup.rs)) et un compilateur C (`sudo apt install build-essential` sur Ubuntu).

```bash
cd rust
cargo run --release -p game                     # toi (bleu) contre un bot (orange)
cargo run --release -p game -- --watch          # bot contre bot
cargo run --release -p game -- --watch --speed=4
```

Options : `--watch`, `--speed=N`, `--camera=ball|car|broadcast`,
`--screenshot=fichier.png --after=3` (enregistre une image après 3 s de jeu simulé, puis quitte).

### Commandes

| Action | Touches |
|---|---|
| Conduire / contrôle en l'air | ZQSD, WASD ou flèches |
| Sauter (2e appui = flip dans la direction tenue) | J ou clic droit |
| Boost | K, Maj gauche ou clic gauche |
| Roulis en l'air | E / Alt gauche |
| Pause | Espace |
| Changer de caméra (balle, voiture, tribune) | C |
| Recommencer le match | R |
| Quitter | Échap |

## Pour les développeurs

```bash
cd rust
cargo test                                            # tests de la physique et du bot
cargo clippy --all-targets -- -D warnings
cargo run --release -p carball-bot --example bench    # taux de buts du bot de référence
cargo run --release -p carball-bot --example versus   # variantes du bot les unes contre les autres
```

La simulation (`rust/crates/sim`) est headless (sans affichage) et déterministe : mêmes entrées, même match.
Elle tourne à 120 ticks par seconde, sans jamais lire l'horloge. Elle est donc prête à être pilotée
à pleine vitesse pour l'entraînement de bots.
