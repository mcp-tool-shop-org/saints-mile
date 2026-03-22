<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.md">English</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

Un JRPG de type "frontier" pour les adultes qui ont aimé ces jeux.

Saint's Mile est un RPG au tour par tour, avec un groupe de personnages, se déroulant dans le bassin de Cinder, un territoire de frontière en pleine transformation grâce au chemin de fer, à l'eau et à la loi. Vous incarnez Galen Rook, un homme dont le nom arrive en ville avant lui, au cours de quatre décennies d'une vie vécue sous le signe d'un avis de recherche rédigé par quelqu'un d'autre.

Développé en Rust pour le terminal. Pas de surcharge graphique. Accent mis sur les mécaniques déterministes, les combats en groupe et une histoire qui fait confiance à son public.

## Ce qu'est ce jeu

- Un **JRPG de style années 90** avec un groupe de 4 personnages, des rôles distincts, des techniques en duo et des combats au tour par tour.
- Un **western de frontière** où la réputation est un réseau, la distance influence les décisions et le chemin est la prison.
- Un **jeu pour adultes** : thèmes de regret, de devoir, de compromis, de vieillesse, de loyauté et de recommencement.
- Une **expérience native du terminal** : fonctionne dans n'importe quel terminal grâce à [ratatui](https://ratatui.rs/).

## L'histoire

Le jeu se déroule sur presque quatre décennies : d'un jeune adjoint de 19 ans qui pense encore que la loi et la vérité sont liées, à un jeune pistolero portant la faute de quelqu'un d'autre, en passant par un hors-la-loi accompli traversant un bassin mourant avec un groupe de spécialistes brisés, jusqu'à un homme plus âgé contraint de décider si une vie peut être rachetée par des actes, par la vérité, ou non.

Le conflit principal est lié au chemin de fer, à l'eau et à la terre. Le conflit plus profond est de savoir qui a le droit d'écrire l'histoire de ce qui s'est passé à Saint's Mile.

## Combats

Une tension de confrontation précède chaque combat important : les mains hésitent, les nerfs sont mis à l'épreuve, l'initiative est gagnée. Ensuite, un système de combat JRPG basé sur le groupe prend le relais : quatre membres actifs parmi une liste de six, chacun avec des ensembles de commandes uniques, des lignes de compétences qui se développent au fil de l'histoire et des liens, et des techniques en duo qui récompensent l'investissement du groupe.

La couche "western" modifie les mécaniques, pas seulement l'esthétique : munitions au lieu de points de magie (PM), nerfs au lieu du moral, ténacité au lieu des bonus de défense, et des blessures qui persistent entre les combats.

## Le groupe

| Personnage | Rôle | Identité au combat |
|-----------|------|----------------|
| **Galen Rook** | Tireur | Précision, tirs ciblés, commandement sur le terrain. Évolue avec l'âge. |
| **Eli Winter** | Escroc | Attaques de nerfs, perturbation, tours. La loyauté se débloque plus tard. |
| **Dr. Ada Mercer** | Médecin | Soins, gestion des blessures, révélation des faiblesses. |
| **Rosa Varela** | Ouvrier de ranch | Contrôle de foule avec la lasso, tanking en première ligne, pression positionnelle. |
| **Rev. Miriam Slate** | Prédicateur | Buffs canalisés, soutien des nerfs, gestion de foule. |
| **Lucien "Fuse" Marr** | Dynamiteur | Dommages de zone retardés, destruction de l'environnement, modification du terrain. |

## Statut

**Phase 1 — Noyau de production.** Campagne complète conçue (Prologue + 15 chapitres). Construction de la constitution et des contrats d'exécution verrouillés. Implémentation de l'arc d'ouverture suivante.

## Modèle de menace

Saint's Mile est un jeu solo et hors ligne. Il ne :
- Se connecte à Internet
- Collecte de données télémétriques ou d'analyses
- Accède à des fichiers en dehors de son propre répertoire de sauvegarde
- Nécessite des autorisations autres que les entrées/sorties du terminal

Les fichiers de sauvegarde sont stockés au format RON dans un répertoire accessible par l'utilisateur.

## Prérequis

- Rust 1.75+ (édition 2021)
- Tout terminal prenant en charge les couleurs 256

## Licence

MIT

---

Développé par <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a
