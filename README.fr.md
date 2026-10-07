<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.md">English</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> Fait partie de [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Comparez deux exécutions d’apprentissage automatique et voyez à quel point la comparaison est fiable.** ScalarScope ouvre deux séries d’inférences, deux historiques d’entraînement ou deux exportations de géométrie ASPIRE. Il indique en quoi l’exécution B diffère de l’exécution A, attribue à chaque différence un intervalle ou sa raison, et retient les informations que les données ne peuvent pas étayer.

La version **3.1.0** est une réécriture en Rust. Il s’agit de l’application Windows disponible sur le Microsoft Store sous le nom [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK), en tant que package 3.1.0.0, une mise à jour de la même version. Le nom du package `mcp-tool-shop.ScalarScope` et l’éditeur `CN=5305D976-6952-4F00-9C21-3A5DB090359F` restent inchangés, de sorte que les fichiers, les évaluations et les paramètres enregistrés de la version 2.0 sont conservés.

## Modèle de confiance

- **Ce qu’il lit.** Les fichiers que vous ouvrez. Le package du Store lit et écrit également `comparison-log.json`, `preferences.json` et `workbench.json` dans son propre dossier LocalState, le dossier utilisé par la version 2.0. Une version non empaquetée n’en écrit aucun.
- **Ce qu’il écrit.** Un ensemble de données, une image ou un enregistrement de session, uniquement dans un chemin que vous choisissez.
- **Réseau.** Il n’y a pas de compte, pas de télémétrie et pas d’analyse. La seule connexion que l’application peut établir provient de l’onglet Workbench, et uniquement lorsque vous appuyez sur **Demander** : vers un Ollama local à l’emplacement `127.0.0.1` sur cet ordinateur. Les modèles cloud d’Ollama sont refusés. Rien ne quitte la machine.
- **Paquets (bundles).** Un hachage SHA-256 correspondant est une vérification du contenu, et non une signature. Cela indique que les octets sont intacts. Cela n’indique pas qui a écrit le fichier.
- **Modules complémentaires.** Les modules complémentaires laissés dans le dossier 2.0 ne sont pas chargés.

---

## Ce qu’il fait

**Inférence : deux exécutions de latence par étape.**
- **Le titre** est B/A aux valeurs p50, p90 et p99. Chaque rapport est assorti d’un intervalle de bootstrap à bloc mobile de 95 %.
- **Plusieurs exécutions de chaque côté** peuvent être ouvertes. L’intervalle rééchantillonne alors également l’ensemble des exécutions, et il est qualifié d’indicatif lorsque le nombre d’exécutions de chaque côté est inférieur à trois.
- **Un percentile sans suffisamment d’échantillons** n’est pas affiché ; p99 nécessite 368.
- **Trois deltas,** chacun avec une vignette indiquant s’il a été déclenché, s’il est resté inactif ou s’il a été retenu, et pourquoi :
- **ΔF (nouvelles anomalies)** compte les échantillons stables au-delà de 5 écarts robustes. Il ne se déclenche que lorsque l’excès de B est supérieur à ce qui pourrait être dû au hasard.
- **ΔO (variabilité)** se déclenche lorsque l’intervalle de la dispersion relative de B par rapport à A exclut 1.
- **ΔTc (stabilisation)** attribue une forme à chaque exécution. Il ne se déclenche que lorsque les deux se stabilisent et que leurs plages de stabilisation ne se chevauchent pas.
- **Six vues :** Série, Échauffement, Distribution avec un seuil que vous faites glisser, Différence par percentile, Spectre et Carte thermique. Aucune ne s’anime.

**Entraînement.** Une rétropropagation `run_history.json` est affichée sous forme de perte d’entraînement, avec la perte retenue, la perplexité et les mesures de tâche à côté. Les deltas d’inférence ne sont pas calculés sur celle-ci.

**Géométrie : deux exécutions d’entraînement ASPIRE.**
- **Ce qu’il lit :** l’exportation de géométrie que [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) écrit, avec sa trajectoire, les scores de l’évaluateur, le spectre propre et les échecs.
- **Cinq deltas :** ΔF, ΔTc, ΔTd, ΔĀ (concentration du spectre) et ΔO. Ils sont importés de la version 2.0 et vérifiés par rapport aux résultats de la version 2.0, les corrections étant enregistrées dans [la spécification](docs/parity-and-beyond.spec.md).
- **Contrat d’exportation (schéma 1.1).** Une exportation peut indiquer que ses étapes sont des points de contrôle × éléments plutôt que du temps, ou que ses scores ont été rejoués. Dans ce cas, les deltas qui lisent le temps ou les baisses de score sont retenus avec cette raison, et le titre ne se base que sur les deltas qui sont pris en compte. Les anciennes exportations sont lues comme auparavant.

**Workbench.**
- **Ouvrez de nombreuses exécutions** qui diffèrent par un paramètre, tel que la taille du lot ou la précision.
- **Demander :** un modèle local capable d’appeler des outils mesure les exécutions à l’aide de formules et propose ce que chaque paramètre fait.
- **Le programme définit chaque nombre et chaque verdict.** Une hypothèse est testée par un test de rang exact sur chaque ensemble d’exécutions. Les verdicts entre les ensembles ne sont donnés qu’aux points de contrôle, par e-BH avec un taux de faux positifs de 5 %.
- **C’est à vous de décider en premier.** Vous pouvez écrire votre propre demande avant de demander. La note du modèle est affichée sous les verdicts, et est étiquetée comme étant de son cru.

**Historique.** Dans le package du Store, les comparaisons que vous effectuez sont regroupées par l’ensemble de données et le modèle du côté B. Chaque mesure est affichée pour ces évaluations, avec les points où son niveau a changé. Un changement de code ou d’environnement à côté d’un changement est nommé et qualifié de coïncidence, et non de cause. Le journal conserve les 40 dernières évaluations.

**Ensembles de données et état enregistré.**
- **Enregistrer l’ensemble de données** enregistre l’évaluation sous forme de `.scbundle`. **Ouvrir l’ensemble de données** l’affiche exactement comme elle a été enregistrée, en mode évaluation.
- **Les ensembles de données de la version 2.0** s’ouvrent : les ensembles de données de comparaison avec leurs deltas stockés, et les évaluations d’inférence marquées comme non vérifiées.
- **Les paramètres de la version 2.0** sont conservés : thème, palettes de couleurs, contraste élevé, échelle du texte, limite des fichiers récents et règle d’anomalie.
- **Exporter** la vue actuelle au format SVG, ou la fenêtre au format PNG.

---

## Démarrage rapide

### Depuis le Microsoft Store

1. Installez **ScalarScope** depuis le [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK). Il nécessite Windows 10 version 1809 (build 17763) ou ultérieure, x64.
2. Dans **Bienvenue**, cliquez sur **Essayez la comparaison d’exemple**. Ou cliquez sur **Comparer deux exécutions** et ouvrez le chemin A et le chemin B.
3. Lisez le titre, puis cliquez sur une vignette pour afficher **Pourquoi** et **Montrez-moi**.

Les fichiers d’exemple à essayer, une paire d’inférence, une paire de géométrie et un dossier d’exécutions pour le Workbench, se trouvent dans [`samples/`](samples/). [TESTING.md](TESTING.md) indique lesquels ouvrir et où.

### Depuis la source

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` ouvre une paire au lancement.

### Ce que vous pouvez ouvrir

| Type | Fichiers |
|---|---|
| Inférence | Un fichier CSV de latence, un fichier JSON de référence, une trace de profilage Chrome ou PyTorch (`.json` ou `.json.gz`), un journal d’exécution, un fichier JSON ScalarScope RunTrace ou un dossier d’exécution |
| Entraînement | Une rétropropagation `run_history.json` |
| Géométrie | Une exportation de géométrie ASPIRE (aspire-si, schéma 1.x) |
| Relecture | Un `.scbundle` de 3.x ou 2.0 |

La section « Premiers pas » du [manuel](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/) contient un court exemple pour chaque fichier.

---

## Raccourcis clavier

| Raccourci | Action |
|---|---|
| `F1` | Guide |
| `Ctrl+,` | Paramètres |
| `Ctrl+H` | Bienvenue |
| `1`–`6` | Dans « Comparer » : séries, phase de test, distribution, différence, spectre, carte thermique |
| `Esc` | Fermer le panneau « Pourquoi » |

---

## Tests

```bash
# The review: 215 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (134 tests)
dotnet test tests/ScalarScope.FixtureTests
```

Le projet .NET dans `src/ScalarScope` est l’application 2.0. Il reste dans le dépôt en tant que référence par rapport à laquelle la version 3.x est vérifiée. Le package Store est créé à partir de `rust/` par `packaging/pack.ps1`, et le projet MAUI refuse d’être publié sous cette forme.

---

## Structure du projet

```
scalarscope/
├── rust/                 # The app: ScalarScope 3.x (egui)
│   ├── src/              # review, geometry, workbench, history, bundles, settings, UI
│   └── tests/            # Rust tests and fixtures (real and simulated aspire-si exports, the knob folder)
├── samples/              # Files a first-time user or tester can open
├── packaging/            # AppxManifest.xml and pack.ps1 (the Store package)
├── src/ScalarScope/      # The 2.0 .NET app, kept as the reference
├── src/VortexKit/        # 2.0's visualization library
├── tests/                # .NET fixture tests and the 2.0 fixtures
├── site/                 # Landing page and handbook
└── docs/                 # The 3.x spec, receipts and release notes
```

---

## Éléments connexes

- [Manuel](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) : le guide pour la relecture
- [Parité et au-delà](docs/parity-and-beyond.spec.md) : ce que la version 3.x conserve de la version 2.0, ce qu’elle modifie et pourquoi, avec les sources
- [CHANGELOG.md](CHANGELOG.md) : historique des versions
- [PRIVACY.md](PRIVACY.md) : politique de confidentialité
- [TESTING.md](TESTING.md) : comment tester cette version
- [L’environnement de travail](https://github.com/mcp-tool-shop-org/runforge) : partagé avec RunForge

---

## Licence

[MIT](LICENSE). Copyright (c) 2025-2026 Projet ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
