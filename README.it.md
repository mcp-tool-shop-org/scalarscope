<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.md">English</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> Parte di [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Una recensione di due esecuzioni di machine learning.** L'app che si crea da questo repository è il programma Rust in `rust/`. Il processo di rilascio impacchetta quel programma come MSIX 3.0.0.0 non firmato. Il nome del pacchetto e l'editore rimangono gli stessi. Quel file non viene caricato. La copia sullo Store è ancora il precedente pacchetto .NET fino a quel caricamento.

Versione del pacchetto **3.0.0.0**. Gli aggiornamenti sullo Store di [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) mantengono il nome `mcp-tool-shop.ScalarScope` e l'editore `CN=5305D976-6952-4F00-9C21-3A5DB090359F`.

## Modello di affidabilità

La recensione legge i due file che si aprono. Un'esecuzione impacchettata legge e scrive anche `comparison-log.json` e `preferences.json` nella cartella LocalState di quel pacchetto. Un pacchetto viene scritto solo nel percorso che si seleziona.

Non invia questi file da nessuna parte. Non ci sono account, telemetria o analisi. I plugin in quella cartella rimangono al loro posto e non vengono caricati. L'hash di un pacchetto verifica i byte archiviati. Si tratta di un controllo del contenuto, non di una firma, e non indica chi ha scritto il file.

Il programma necessita del permesso di leggere i file che si selezionano e di scrivere il pacchetto che si salva.

---

## Perché ScalarScope?

La maggior parte dei team di ML esamina i log. ScalarScope sostituisce questo con un confronto strutturato e riproducibile.

I punti seguenti descrivono il pacchetto .NET pubblicato. La recensione Rust elabora una serie di inferenze o una curva di perdita di addestramento retropropagante. Su una coppia di inferenze, segnala ΔF e ΔO e ΔTc solo quando entrambi i lati hanno una pietra miliare di stato stazionario. Quando entrambe le pietre miliari esistono, la serie disegna una linea verticale lì. Scrive un `.scbundle` nel layout di Fase 7.2. Riaprendo quel file, si visualizza la recensione salvata. L'hash è SHA-256 dei byte del file archiviato e `integrity.json` è il sigillo. Un hash corrispondente è un controllo del contenuto, non una firma. Un'esecuzione impacchettata mantiene `comparison-log.json` e `preferences.json` nella cartella LocalState di quel pacchetto, gli stessi file su cui ha scritto l'app .NET. I colori delle serie seguono una modalità di visione dei colori salvata. I file recenti e le viste salvate da quella cartella si riaprono qui. I plugin in quella cartella rimangono al loro posto e non vengono caricati. Un'esecuzione non impacchettata non scrive quella cartella.

- **Confronto diretto** — Carica due tracce di inferenza affiancate e osserva esattamente cosa è cambiato
- **Analisi delta canonica** — Cinque tipi di delta (ΔTc, ΔO, ΔF, ΔĀ, ΔTd) si attivano solo quando le differenze sono statisticamente significative
- **Preset di runtime** — Il preset TFRT sopprime automaticamente le metriche irrilevanti in modo che tu possa concentrarti su ciò che conta per i carichi di lavoro TensorFlow-TRT
- **Pacchetti riproducibili** — Esporta archivi `.scbundle` con integrità SHA-256, delta congelati e metadati completi di provenienza
- **Modalità di recensione** — Apri un pacchetto senza ricalcolare. Un SHA-256 corrispondente è un controllo del contenuto, non una firma.
- **Privacy al primo posto** — Nessuna telemetria, nessuna analisi, tutti i dati rimangono locali a meno che tu non li esporti esplicitamente

---

## VortexKit

VortexKit è la libreria di visualizzazione in `src/VortexKit`. Fa parte di questo repository. Non è pubblicato su NuGet.

Copre la riproduzione sincronizzata nel tempo, le animazioni di canvas SkiaSharp, le viste di confronto, le sovrapposizioni di annotazione, l'esportazione SVG e PNG e un sistema di colori semantico.

---

## Avvio rapido

### Recensione Rust

Da questo repository:

```
cargo run --manifest-path rust/Cargo.toml
```

Apri due file di inferenza o due file `run_history.json` di retropropagazione. Un file di inferenza è un CSV di latenza, un JSON di benchmark o una traccia di Chrome. Un `ProfilerStep` completo in quella traccia è una singola inferenza e le operazioni al suo interno non sono campioni aggiuntivi. Una traccia senza passaggi utilizza comunque eventi i cui nomi contengono TensorRT o inferenza. Un file di addestramento viene disegnato come perdita di addestramento. La perdita trattenuta, la perplessità e le metriche delle attività sono visualizzate insieme a quella curva. `final_loss` viene visualizzato come un numero a sé stante. Una coppia di inferenze segnala ΔF e ΔO dalla serie di latenza. ΔTc viene segnalato solo quando entrambi i file hanno una pietra miliare di stato stazionario. Senza quella pietra miliare, l'ultimo passaggio non viene definito come tempo di stabilizzazione e la serie non disegna una linea di stato stazionario. ΔTd e ΔĀ non vengono visualizzati nella pagina di inferenza. I delta di inferenza non vengono calcolati su una cronologia di addestramento. Il salvataggio del pacchetto scrive la recensione sulla pagina. L'apertura del pacchetto mostra nuovamente quella recensione salvata. L'hash corrisponde al controllo .NET di Fase 7.2. Una corrispondenza significa che i byte sono intatti. Non è una firma.

`packaging/pack.ps1` crea il `ScalarScope_3.0.0.0_x64.msix` non firmato dal binario di rilascio. Il nome del pacchetto è `mcp-tool-shop.ScalarScope`, l'editore è `CN=5305D976-6952-4F00-9C21-3A5DB090359F` e l'architettura è x64. Non viene caricato. La copia sullo Store è ancora il precedente pacchetto .NET.

### Dal Microsoft Store

1. Installa **ScalarScope** dal [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (ID dello Store: `9P3HT1PHBKQK`)
2. Fai clic su **Confronta due esecuzioni**
3. Carica una traccia di riferimento: un CSV di latenza, un JSON di benchmark o un profiler `trace.json`
4. Carica la traccia ottimizzata nello stesso tipo di file
5. Esamina i delta nella scheda **Confronta**
6. Esporta un `.scbundle` per una condivisione riproducibile

### Utilizzo di VortexKit

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

## Funzionalità

### Analisi delta: cinque tipi di delta canonici

Ogni confronto produce un insieme di delta canonici. Ogni delta si attiva solo quando la differenza è statisticamente significativa; i delta irrilevanti vengono soppressi automaticamente.

| Delta | Nome completo | Cosa misura | Si attiva quando |
|-------|-----------|------------------|------------|
| **ΔTc** | Tempo di convergenza | Passaggi per raggiungere una latenza stabile | Lo stato stazionario viene raggiunto in passaggi diversi (separazione di 3+ passaggi) |
| **ΔO** | Variabilità dell'output | Oscillazione / instabilità in fase di esecuzione | Il punteggio dell'area sopra la soglia differisce oltre la soglia di rumore |
| **ΔF** | Tasso di errore | Frequenza delle anomalie | La frequenza o il tipo di errore differiscono tra le esecuzioni |
| **ΔĀ** | Latenza media | Valore medio della metrica | La differenza media è significativa (soppressa nella configurazione predefinita TFRT) |
| **ΔTd** | Durata totale | Tempo di esecuzione / emergenza strutturale | La durata o l'inizio della dominanza differiscono (soppresso nella configurazione predefinita TFRT) |

### Configurazioni predefinite per l'esecuzione — TFRT

La configurazione predefinita integrata **TensorFlow-TRT** (`tensorflowrt-runtime-v1`) mappa i segnali specifici per l'inferenza (latenza, velocità di trasmissione, memoria, carico della CPU/GPU) e sopprime le differenze valide solo per l'addestramento (ΔĀ, ΔTd) che non hanno significato per il confronto dell'inferenza. I meccanismi di sicurezza avvisano quando il periodo di riscaldamento supera il 50% dell'esecuzione o quando sono disponibili solo statistiche aggregate.

### Pacchetti riproducibili

Esporta i risultati come archivi `.scbundle` (ComparisonBundle v1.0.0):

- **`manifest.json`** — metadati del pacchetto, versione dell'app, etichette di confronto, modalità di allineamento
- **`repro/repro.json`** — impronte digitali degli input, hash della configurazione predefinita, seme di determinismo, informazioni sull'ambiente
- **`findings/deltas.json`** — differenze canoniche con punteggi di confidenza, punti di riferimento e tipi di trigger
- **`findings/why.json`** — spiegazioni leggibili dall'utente, meccanismi di sicurezza, elementi di parametro
- **`findings/summary.md`** — riepilogo Markdown generato automaticamente
- **Integrità** — ogni file ha un hash SHA-256. L'hash del pacchetto è un controllo del contenuto, non una firma.

### Modalità di revisione

Apri qualsiasi `.scbundle` senza ricalcolare. Un hash SHA-256 corrispondente è un controllo del contenuto, non una firma. Le differenze memorizzate vengono visualizzate come memorizzate.

### Framework di visualizzazione VortexKit

VortexKit è la libreria di visualizzazione in `src/VortexKit`. Viene fornita con questo repository. Non è un pacchetto NuGet.

| Componente | Cosa fa |
|-----------|-------------|
| `PlaybackController` | Timeline condivisa 0→1 con riproduzione/pausa/passo/ciclo, configurazioni predefinite per la velocità (0,25x—4x), aggiornamento a ~60 fps |
| `AnimatedCanvas` | Base astratta `SKCanvasView` con invalidazione sincronizzata con il tempo, disegno della griglia, eventi di tocco/trascinamento, helper per le coordinate |
| `ITimeSeries<T>` / `TimeSeries<T>` | Serie temporale generica con mappatura indice↔tempo ed enumerazione del percorso |
| `ExportService` | PNG a fotogramma singolo, sequenze di fotogrammi (con suggerimenti ffmpeg) ed esportazione di confronto affiancato |
| `SvgExportService` | Esportazione SVG completamente vettoriale con livelli Inkscape, spline Catmull-Rom, mappe di calore, campi vettoriali e quattro tavolozze di colori (Predefinita, Chiara, Contrasto elevato, Pubblicazione) |
| `IAnnotation` | Annotazioni tipizzate (Fase, Avviso, Approfondimento, Errore, Personalizzata) con base teorica e priorità |
| `VortexColors` | Tavolozza di colori semantica: livelli di sfondo, semantica di accento, codifica della gravità, tavolozza degli autovalori, helper per lerp/gradiente |

---

## Installazione

### Microsoft Store (consigliato)

**ID Store:** `9P3HT1PHBKQK`

[Scaricalo dal Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

Richiede Windows 10 (versione 17763) o successiva.

### Da sorgente

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

## Struttura del progetto

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

## Test

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

## Scorciatoie da tastiera

| Scorciatoia | Azione |
|----------|--------|
| `Space` | Riproduci / Pausa |
| `Left` / `Right` | Passo indietro / avanti (1%) |
| `Shift+Left` / `Shift+Right` | Passo fine (0,1%) |
| `Home` / `End` | Vai all'inizio / alla fine |
| `Up` / `+` | Aumenta la velocità di riproduzione |
| `Down` / `-` | Diminuisci la velocità di riproduzione |
| `0` | Ripristina la velocità a 1x |
| `S` o `Ctrl+S` | Tenta di scrivere un PNG nella cartella di esportazione dalle impostazioni o in Documents/ScalarScope Exports quando non è impostata alcuna cartella. Non viene visualizzata alcuna notifica. Ctrl+E non è mappato. |
| `1`–`6` | Richiede una panoramica delle route, della traiettoria, degli scalari, della geometria, del confronto e degli errori. Non Home, Confronta, Guida o Impostazioni. Premendo 1 non si apre Home. |
| `?` | Apri la guida |

---

## Correlati

- [Handbook](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — La guida per la revisione
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — Risultati sperimentali completi
- [CHANGELOG.md](CHANGELOG.md) — Cronologia delle versioni
- [PRIVACY.md](PRIVACY.md) — Informativa sulla privacy
- [ROADMAP.md](ROADMAP.md) — Un piano più vecchio non verificato, non la revisione corrente

---

## Licenza

[MIT](LICENSE) — Copyright (c) 2025-2026 ScalarScope Project (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
