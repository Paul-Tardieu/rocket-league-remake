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

Pour voir le bot jouer :

```bash
uv run python replay.py --seconds 180   # enregistre 3 min avec la dernière sauvegarde
```

Ça écrit `data/replay.html`, une page à ouvrir dans un navigateur : replay vu de dessus, nombre de touches
comparé à un bot qui joue au hasard, et courbe d'apprentissage (lue dans `data/train.log` si tu as lancé
l'entraînement avec `script -qfc "uv run python train.py" data/train.log`).

Récompenses (`rewards.py`) : vitesse vers la balle, vitesse de la balle vers le but adverse, petit bonus
en l'air, touche de balle et but. Les buts seuls sont trop rares pour qu'un bot débutant apprenne quoi que ce soit.

Les sauvegardes du modèle vont dans `data/checkpoints/` (les 5 plus récentes sont gardées). Il faut lancer `train.py` dans un vrai terminal :
RLGym-PPO lit le clavier pendant l'entraînement.

Mesuré sur la machine de dev (16 cœurs, GPU NVIDIA) : environ 19 000 pas d'agent par seconde avec 24 processus
(~150 000 ticks de physique par seconde, ~1200× le temps réel), processeur à ~50 %. Nombre de processus
réglable avec `CARBALL_N_PROC`.

## Suivi en direct sur projects.tardieu.ovh

La page [projects.tardieu.ovh/rocket-league/](https://projects.tardieu.ovh/rocket-league/) montre l'entraînement
en cours : statut et machine, courbes (récompense, entropie, divergence KL, pas par seconde), touches et buts de
la dernière sauvegarde comparés au jeu au hasard, et son replay vu de dessus.

Pendant l'entraînement, dans un deuxième terminal :

```bash
uv run python publish.py              # publie toutes les 5 min (Ctrl+C pour arrêter)
uv run python publish.py --dry-run    # fabrique data/snapshot.json sans rien envoyer
uv run python publish.py --site       # envoie la page elle-même (site/rocket-league/) après l'avoir modifiée
```

Comment ça marche : `publish.py` lit les rapports d'itération de `data/train.log`, enregistre 60 s de replay à
chaque nouvelle sauvegarde, et écrit tout dans **un seul fichier JSON** qu'il fabrique lui-même. Il le pousse
comme unique commit de la branche `data` du repo GitHub `Paul-Tardieu/projects` (la branche est écrasée à chaque
fois, donc le repo ne grossit pas). La page, servie par GitHub Pages depuis la branche `main` du même repo, relit
ce fichier toutes les minutes. Les PC d'entraînement n'ouvrent aucun port : ils envoient, personne ne s'y
connecte. Aucun checkpoint ni maillage de collision n'est jamais envoyé.

Le statut « en cours » veut dire que `data/train.log` a bougé dans les 3 dernières minutes : lancer
l'entraînement avec `script -qfc "uv run python train.py" data/train.log`. Une seule machine publie à la fois ;
son nom vient de `CARBALL_MACHINE` (sinon le nom de l'ordinateur).

Sur une autre machine (le PC fixe) : il lui faut un droit d'écriture sur `Paul-Tardieu/projects`. Le plus sûr
est une clé de déploiement, qui ne donne accès qu'à ce repo :

```bash
ssh-keygen -t ed25519 -f ~/.ssh/carball_publish -N ""
gh repo deploy-key add ~/.ssh/carball_publish.pub --repo Paul-Tardieu/projects --allow-write --title "PC fixe"
CARBALL_PUBLISH_REPO="git@github.com:Paul-Tardieu/projects.git" GIT_SSH_COMMAND="ssh -i ~/.ssh/carball_publish" \
  CARBALL_MACHINE="pc-fixe" uv run python publish.py
```

## Jouer contre le bot dans le vrai jeu

Le bot entraîné se branche sur Rocket League avec [RLBot](https://rlbot.org), uniquement en matchs
**hors ligne ou privés**. Un bot en partie en ligne, c'est de la triche.
