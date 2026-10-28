# Entraînement de bots Rocket League

Entraînement par renforcement (PPO) dans **RocketSim**, une simulation de la physique de Rocket League
qui tourne bien plus vite que le temps réel, via **RLGym v2** et **RLGym-PPO**.
Bases : [quickstart RLGym](https://rlgym.org/Getting%20Started/quickstart/) et
[Training an Agent](https://rlgym.org/Rocket%20League/training_an_agent/).

## Installation (une seule fois)

1. Installer [uv](https://docs.astral.sh/uv/) : `curl -LsSf https://astral.sh/uv/install.sh | sh`
2. Depuis ce dossier : `uv sync --python 3.12` (installe PyTorch avec CUDA, RLGym, RocketSim et RLGym-PPO dans `.venv/`)
3. **Maillages de collision** : ils viennent de *ta propre* installation de Rocket League (le jeu est gratuit) :
   - télécharger le `.exe` de [RLArenaCollisionDumper](https://github.com/ZealanL/RLArenaCollisionDumper/releases) sur Windows ;
   - lancer Rocket League en **Free Play**, puis le `.exe` ;
   - copier le dossier créé à côté du `.exe` ici, sous le nom `training/collision_meshes/` (il doit contenir `soccar/mesh_*.cmf`).

   Ce sont des fichiers du jeu : ils sont exclus de Git et ne doivent jamais être publiés.

## Utilisation

```bash
uv run python smoke_test.py   # vérifie l'arène et la physique, mesure la vitesse (~2 s)
uv run python train.py        # lance l'entraînement 1v1 (Ctrl+C pour arrêter)
```

Les sauvegardes du modèle vont dans `data/checkpoints/`. Il faut lancer `train.py` dans un vrai terminal :
RLGym-PPO lit le clavier pendant l'entraînement.

Mesuré sur la machine de dev (16 cœurs, GPU NVIDIA) : environ 15 000 pas d'agent par seconde, soit
~120 000 ticks de physique par seconde (~1000× le temps réel).

## Jouer contre le bot dans le vrai jeu

Le bot entraîné se branche sur Rocket League avec [RLBot](https://rlbot.org), uniquement en matchs
**hors ligne ou privés**. Un bot en partie en ligne, c'est de la triche.
