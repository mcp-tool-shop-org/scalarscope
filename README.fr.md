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

**Analyse de deux exécutions d’apprentissage automatique.** L’application que vous créez à partir de ce dépôt est le programme Rust dans `rust/`. Le processus de publication crée un package MSIX 3.0.0.0 non signé. Le nom du package et l’éditeur restent les mêmes. Ce fichier n’est pas téléchargé. La copie dans le Store est toujours le package .NET précédent jusqu’à ce que ce fichier soit téléchargé.

Version du package : **3.0.0.0**. Les mises à jour du Store pour [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) conservent le nom `mcp-tool-shop.ScalarScope` et l’éditeur `CN=5305D976-6952-4F00-9C21-3A5DB090359F`.

## Modèle de confiance

L’analyse lit les deux fichiers que vous ouvrez. Une exécution empaquetée lit et écrit également `comparison-log.json` et `preferences.json` dans le dossier LocalState de ce package. Un ensemble de fichiers est écrit uniquement dans le chemin que vous choisissez.

Ces fichiers ne sont envoyés nulle part. Il n’y a pas de compte, pas de télémétrie et pas d’analyse. Les plug-ins dans ce dossier sont laissés en place et ne sont pas chargés. Le hachage d’un ensemble de fichiers vérifie les octets archivés. Il s’agit d’une vérification du contenu, et non d’une signature, et il n’indique pas qui a écrit le fichier.

Le programme a besoin de l’autorisation de lire les fichiers que vous choisissez et d’écrire l’ensemble de fichiers que vous enregistrez.

---

## Pourquoi ScalarScope ?

La plupart des équipes d’apprentissage automatique examinent les journaux. ScalarScope remplace cette pratique par une comparaison structurée et reproductible.

Les puces ci-dessous décrivent le package .NET publié. L’analyse Rust effectue une série d’inférences ou trace une courbe d’apprentissage par rétropropagation. Pour une paire d’inférences, elle affiche ΔF et ΔO, et ΔTc uniquement lorsque les deux côtés présentent une étape stable. Lorsque les deux étapes existent, la série trace une ligne verticale à cet endroit. Elle écrit un `.scbundle` dans la mise en page de la phase 7.2. La réouverture de ce fichier affiche l’analyse enregistrée. Le hachage est SHA-256 des octets du fichier archivé, et `integrity.json` est le sceau. Un hachage correspondant est une vérification du contenu, et non une signature. Une exécution empaquetée conserve `comparison-log.json` et `preferences.json` dans le dossier LocalState de ce package, les mêmes fichiers que l’application .NET a écrits. Les couleurs des séries suivent un mode de vision des couleurs enregistré. Les fichiers récents et les vues enregistrées de ce dossier se réouvrent ici. Les plug-ins dans ce dossier sont laissés en place et ne sont pas chargés. Une exécution non empaquetée n’écrit pas ce dossier.

- **Comparaison directe** — Chargez deux séries d’inférences côte à côte et voyez exactement ce qui a changé.
- **Analyse delta canonique** — Cinq types de delta (ΔTc, ΔO, ΔF, ΔĀ, ΔTd) ne sont déclenchés que lorsque les différences sont statistiquement significatives.
- **Préréglages d’exécution** — Le préréglage TFRT supprime automatiquement les métriques non pertinentes afin que vous puissiez vous concentrer sur ce qui compte pour les charges de travail TensorFlow-TRT.
- **Ensembles de fichiers reproductibles** — Exportez des archives `.scbundle` avec une intégrité SHA-256, des deltas figés et des métadonnées de provenance complètes.
- **Mode d’analyse** — Ouvrez un ensemble de fichiers sans recalculer. Un hachage SHA-256 correspondant est une vérification du contenu, et non une signature.
- **Priorité à la confidentialité** — Zéro télémétrie, zéro analyse, toutes les données restent locales, sauf si vous les exportez explicitement.

---

## VortexKit

VortexKit est la bibliothèque de visualisation dans `src/VortexKit`. Elle fait partie de ce dépôt. Elle n’est pas publiée sur NuGet.

Elle couvre la lecture synchronisée dans le temps, les canevas SkiaSharp animés, les vues de comparaison, les superpositions d’annotations, l’exportation SVG et PNG, et un système de couleurs sémantique.

---

## Démarrage rapide

### Analyse Rust

À partir de ce dépôt :

```
cargo run --manifest-path rust/Cargo.toml
```

Ouvrez deux fichiers d’inférence ou deux fichiers `run_history.json` de rétropropagation. Un fichier d’inférence est un fichier CSV de latence, un fichier JSON de référence ou une trace Chrome. Un `ProfilerStep` complet dans cette trace est une inférence, et les opérations qu’il contient ne sont pas des échantillons supplémentaires. Une trace sans étape utilise toujours des événements dont les noms contiennent TensorRT ou inférence. Un fichier d’apprentissage est affiché sous forme de perte d’apprentissage. La perte retenue, la perplexité et les métriques de tâche sont affichées avec cette courbe. `final_loss` est affiché comme un nombre distinct. Une paire d’inférences affiche ΔF et ΔO à partir de la série de latence. ΔTc est affiché uniquement lorsque les deux fichiers ont une étape stable. Sans cette étape, la dernière étape n’est pas appelée temps de stabilisation, et la série ne trace pas de ligne d’état stable. ΔTd et ΔĀ ne sont pas affichés sur la page d’inférence. Les deltas d’inférence ne sont pas calculés sur un historique d’apprentissage. L’enregistrement de l’ensemble de fichiers écrit l’analyse sur la page. L’ouverture de l’ensemble de fichiers affiche à nouveau cette analyse enregistrée. Le hachage correspond à la vérification de la phase 7.2 du .NET. Une correspondance signifie que les octets sont intacts. Il ne s’agit pas d’une signature.

`packaging/pack.ps1` crée le `ScalarScope_3.0.0.0_x64.msix` non signé à partir du fichier binaire de publication. Le nom du package est `mcp-tool-shop.ScalarScope`, l’éditeur est `CN=5305D976-6952-4F00-9C21-3A5DB090359F` et l’architecture est x64. Il n’est pas téléchargé. La copie dans le Store est toujours le package .NET précédent.

### À partir du Microsoft Store

1. Installez **ScalarScope** à partir du [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (ID du Store : `9P3HT1PHBKQK`)
2. Cliquez sur **Comparer deux exécutions**
3. Chargez une trace de référence : un fichier CSV de latence, un fichier JSON de référence ou un fichier de profilage `trace.json`
4. Chargez la trace optimisée dans le même type de fichier
5. Examinez les deltas dans l’onglet **Comparer**
6. Exportez un `.scbundle` pour un partage reproductible

### Utilisation de VortexKit

```csharp
using VortexKit.Core;

// 1. Create a shared playback controller (0.0 -> 1.0 timeline)
var player = new PlaybackController { Duration = 10.0, Loop = true };

// 2. Bind multiple animated canvases to the same controller
player.TimeChanged += () =>
{
    trajectoryCanvas.CurrentTime = player.Time;
    eigenCanvas.CurrentTime      = player.Time;
    scalarsCanvas.CurrentTime    = player.Time;
};

// 3. Subclass AnimatedCanvas for custom rendering
public class MyTrajectoryCanvas : AnimatedCanvas
{
    protected override void OnRender(SKCanvas canvas, SKImageInfo info, double time)
    {
        // Your SkiaSharp rendering at the current time position
    }
}

// 4. Export a side-by-side comparison as PNG
var exporter = new ExportService();
await exporter.ExportComparisonAsync(
    leftRender, rightRender, time: 0.5,
    outputPath: "comparison.png",
    new ComparisonExportOptions
    {
        Width = 1920, Height = 1080,
        LeftLabel = "Baseline", RightLabel = "Optimized",
        ShowLabels = true
    });

// 5. Export as layered SVG (Inkscape-compatible)
var svgExporter = new SvgExportService();
await svgExporter.ExportSvgAsync(svgData, "trajectory.svg",
    new SvgExportOptions
    {
        Palette = SvgColorPalette.Publication,
        UseCatmullRomSplines = true,
        EnableGlow = false
    });
```

---

## Fonctionnalités

### Analyse delta — Cinq types de delta canoniques

Chaque comparaison produit un ensemble de deltas canoniques. Chaque delta ne se déclenche que lorsque la différence est statistiquement significative ; les deltas non pertinents sont automatiquement supprimés.

| Delta | Nom complet | Ce qu’il mesure | Se déclenche lorsque |
|-------|-----------|------------------|------------|
| **ΔTc** | Temps de convergence | Étapes nécessaires pour atteindre une latence stable | L’état stable est atteint à des étapes différentes (séparation de 3 étapes ou plus) |
| **ΔO** | Variabilité de la sortie | Oscillation / instabilité pendant l’exécution | Le score au-dessus du seuil diffère au-delà du seuil de bruit |
| **ΔF** | Taux d’échec | Fréquence des anomalies | La fréquence ou le type des échecs diffèrent entre les exécutions |
| **ΔĀ** | Latence moyenne | Valeur moyenne de la métrique | La différence moyenne est significative (supprimée dans la configuration par défaut TFRT) |
| **ΔTd** | Durée totale | Temps écoulé / émergence structurelle | La durée ou le début de la dominance diffère (supprimé dans la configuration par défaut TFRT) |

### Configurations par défaut d’exécution — TFRT

La configuration intégrée **TensorFlow-TRT** (`tensorflowrt-runtime-v1`) mappe les signaux spécifiques à l’inférence (latence, débit, mémoire, charge du CPU/GPU) et supprime les différences (ΔĀ, ΔTd) qui ne sont pertinentes que pour l’entraînement et qui n’ont aucun sens pour la comparaison de l’inférence. Des mécanismes de sécurité avertissent lorsque la période de préchauffage dépasse 50 % de l’exécution ou lorsque seules des statistiques agrégées sont disponibles.

### Ensembles reproductibles

Exporter les résultats sous forme d’archives `.scbundle` (ComparisonBundle v1.0.0) :

- **`manifest.json`** — métadonnées de l’ensemble, version de l’application, étiquettes de comparaison, mode d’alignement
- **`repro/repro.json`** — empreintes d’entrée, hachage de la configuration, valeur d’initialisation du déterminisme, informations sur l’environnement
- **`findings/deltas.json`** — différences canoniques avec des scores de confiance, des points d’ancrage et des types de déclencheurs
- **`findings/why.json`** — explications lisibles par l’homme, mécanismes de sécurité, éléments de paramètre
- **`findings/summary.md`** — résumé généré automatiquement au format Markdown
- **Intégrité** — chaque fichier est haché avec SHA-256. Le hachage de l’ensemble est une vérification du contenu, et non une signature.

### Mode d’examen

Ouvrir n’importe quel `.scbundle` sans recalculer. Un hachage SHA-256 correspondant est une vérification du contenu, et non une signature. Les différences stockées sont affichées telles qu’elles ont été stockées.

### Framework de visualisation VortexKit

VortexKit est la bibliothèque de visualisation dans `src/VortexKit`. Elle est fournie avec ce dépôt. Il ne s’agit pas d’un package NuGet.

| Composant | Ce qu’il fait |
|-----------|-------------|
| `PlaybackController` | Chronologie partagée 0→1 avec lecture/pause/étape/boucle, configurations de vitesse (0,25x à 4x), taux de rafraîchissement d’environ 60 ips |
| `AnimatedCanvas` | Base abstraite `SKCanvasView` avec invalidation synchronisée avec le temps, dessin de grille, événements tactiles/de glissement, assistants de coordonnées |
| `ITimeSeries<T>` / `TimeSeries<T>` | Série chronologique générique avec mappage index↔temps et énumération de la trace |
| `ExportService` | PNG à image unique, séquences d’images (avec indications ffmpeg) et exportation de comparaison côte à côte |
| `SvgExportService` | Exportation SVG entièrement vectorielle avec calques Inkscape, splines de Catmull-Rom, cartes thermiques, champs vectoriels et quatre palettes de couleurs (Par défaut, Clair, Contraste élevé, Publication) |
| `IAnnotation` | Annotations typées (Phase, Avertissement, Information, Échec, Personnalisé) avec base théorique et priorité |
| `VortexColors` | Palette de couleurs sémantique — calques d’arrière-plan, sémantique d’accentuation, codage de la gravité, palette de valeurs propres, assistants de lissage/de dégradé |

---

## Installation

### Microsoft Store (recommandé)

**ID du Store :** `9P3HT1PHBKQK`

[Téléchargez-le sur le Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

Nécessite Windows 10 (version 17763) ou ultérieure.

### À partir du code source

```bash
# Prerequisites:
#   .NET 9.0 SDK (global.json pins 9.0.100)
#   Visual Studio 2022 with MAUI workload, or:
#     dotnet workload install maui-windows

git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet restore
dotnet build

# Run the desktop app
dotnet run --project src/ScalarScope
```

---

## Structure du projet

```
scalarscope/
├── src/
│   ├── ScalarScope/                    # .NET MAUI desktop app
│   │   ├── Models/                     # GeometryRun, InsightEvent
│   │   ├── ViewModels/                 # Welcome, Comparison, Export, Settings, TrajectoryPlayer, VortexSession
│   │   ├── Views/                      # XAML pages + 19 custom controls
│   │   │   ├── WelcomePage.xaml        # First-60-seconds onboarding (Home tab)
│   │   │   ├── ComparisonPage.xaml     # Side-by-side delta comparison (Compare tab)
│   │   │   ├── HelpPage.xaml           # Interpretation guide (Guide tab)
│   │   │   ├── SettingsPage.xaml       # Preferences and about (Settings tab)
│   │   │   └── Controls/              # DeltaZone, BundleExportPanel, PlaybackControl, etc.
│   │   ├── Services/
│   │   │   ├── Connectors/            # RunTraceComparer, TfrtRuntimePreset, validation
│   │   │   ├── Bundles/               # BundleBuilder, BundleExporter, integrity, schemas
│   │   │   ├── Evidence/              # Comparison evidence reports, detector diagnostics
│   │   │   ├── Plugins/               # PluginManager
│   │   │   ├── CanonicalDeltaService.cs
│   │   │   ├── DeltaTypes.cs          # 5 canonical deltas + detector configs
│   │   │   ├── DeterminismService.cs  # Reproducible seed management
│   │   │   ├── FlowFieldService.cs    # Vector field computation
│   │   │   └── ...                    # 70+ service files
│   │   └── Resources/
│   │       ├── Styles/DesignSystem.xaml # Unified visual grammar
│   │       └── Raw/Samples/            # Built-in example traces
│   │
│   └── VortexKit/                      # Visualization library in this repo
│       ├── Core/
│       │   ├── AnimatedCanvas.cs       # Time-synced SkiaSharp canvas base
│       │   ├── PlaybackController.cs   # Shared playback timeline
│       │   ├── ITimeSeries.cs          # Generic time-series interface
│       │   ├── ExportService.cs        # PNG frame/sequence export
│       │   └── SvgExportService.cs     # Layered SVG export
│       ├── Annotations/
│       │   └── IAnnotation.cs          # Typed annotation system
│       └── Theme/
│           └── VortexColors.cs         # Semantic color palette
│
├── tests/
│   ├── ScalarScope.FixtureTests/       # Golden-file fixture tests
│   ├── ScalarScope.DeterminismTests/   # Reproducibility verification
│   ├── ScalarScope.SoakTests/          # Long-running stability tests
│   └── Fixtures/                       # Shared test data
│
├── docs/                               # Design docs, results, limitations
├── .github/workflows/
│   ├── build.yml                       # CI: restore, build, format check, pack, artifacts
│   ├── publish.yml                     # NuGet publish
│   └── release.yml                     # GitHub Release + Store submission
├── global.json                         # .NET SDK 9.0.100
├── ScalarScope.sln                     # Solution file
├── CHANGELOG.md                        # Keep-a-Changelog format
├── PRIVACY.md                          # Privacy policy (no telemetry)
├── SECURITY.md                         # Security policy
└── STORE_LISTING.md                    # Microsoft Store listing copy
```

---

## Tests

```bash
# Run all tests
dotnet test

# Fixture smoke tests only
dotnet test --filter Category=FixtureSmoke

# Determinism tests (verifies reproducible deltas)
dotnet test --filter Category=Determinism

# With coverage
dotnet test --collect:"XPlat Code Coverage"

# Rust review. Line coverage has to stay above 90%.
cd rust
cargo llvm-cov --offline --locked --all-targets --fail-under-lines 90
```

---

## Raccourcis clavier

| Raccourci | Action |
|----------|--------|
| `Space` | Lecture / Pause |
| `Left` / `Right` | Avance / recule d’une étape (1 %) |
| `Shift+Left` / `Shift+Right` | Avance d’une petite étape (0,1 %) |
| `Home` / `End` | Aller au début / à la fin |
| `Up` / `+` | Augmenter la vitesse de lecture |
| `Down` / `-` | Diminuer la vitesse de lecture |
| `0` | Réinitialiser la vitesse à 1x |
| `S` ou `Ctrl+S` | Tente d’écrire un PNG dans le dossier d’exportation des paramètres ou dans Documents/ScalarScope Exports lorsqu’aucun n’est défini. Aucune notification n’est affichée. Ctrl+E n’est pas mappé. |
| `1`–`6` | Demande une vue d’ensemble des itinéraires, de la trajectoire, des scalaires, de la géométrie, de la comparaison et des échecs. Ne correspond pas à Accueil, Comparaison, Guide ou Paramètres. Appuyer sur 1 n’ouvre pas Accueil. |
| `?` | Ouvrir l’aide / le guide |

---

## Articles connexes

- [Manuel](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — Le guide pour l’examen
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — Résultats expérimentaux complets
- [CHANGELOG.md](CHANGELOG.md) — Historique des versions
- [PRIVACY.md](PRIVACY.md) — Politique de confidentialité
- [ROADMAP.md](ROADMAP.md) — Un ancien plan non vérifié, et non l’examen actuel

---

## Licence

[MIT](LICENSE) — Copyright (c) 2025-2026 Projet ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
