<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.md">English</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

Parte de [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Una revisión de dos ejecuciones de aprendizaje automático.** La aplicación que se crea a partir de este repositorio es el programa Rust en `rust/`. El flujo de trabajo de lanzamiento empaqueta ese programa como el MSIX 3.0.0.0 sin firmar. El nombre del paquete y el editor permanecen iguales. Ese archivo no se carga. La copia en la tienda sigue siendo el paquete .NET anterior hasta que se realice esa carga.

Versión del paquete **3.0.0.0**. Las actualizaciones de la tienda de [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) mantienen el nombre `mcp-tool-shop.ScalarScope` y el editor `CN=5305D976-6952-4F00-9C21-3A5DB090359F`.

## Modelo de confianza

La revisión lee los dos archivos que se abren. Una ejecución empaquetada también lee y escribe `comparison-log.json` y `preferences.json` en la carpeta LocalState de ese paquete. Un paquete se escribe solo en la ruta que se elige.

No envía esos archivos a ninguna parte. No hay ninguna cuenta, ni telemetría, ni análisis. Los complementos de esa carpeta se dejan en su lugar y no se cargan. El hash de un paquete verifica los bytes archivados. Es una verificación de contenido, no una firma, y no indica quién escribió el archivo.

El programa necesita permiso para leer los archivos que se eligen y para escribir el paquete que se guarda.

---

## ¿Por qué ScalarScope?

La mayoría de los equipos de aprendizaje automático examinan los registros. ScalarScope reemplaza eso con una comparación estructurada y reproducible.

Los puntos que se indican a continuación describen el paquete .NET publicado. La revisión de Rust realiza una serie de inferencias o una curva de pérdida de entrenamiento de retropropagación. En un par de inferencias, informa ΔF y ΔO, y ΔTc solo cuando ambos lados tienen un hito de estado estacionario. Cuando ambos hitos existen, la serie dibuja una línea vertical allí. Escribe un `.scbundle` en el diseño de la fase 7.2. Volver a abrir ese archivo muestra la revisión almacenada. El hash es SHA-256 de los bytes del archivo archivado, y `integrity.json` es el sello. Un hash coincidente es una verificación de contenido, no una firma. Una ejecución empaquetada mantiene `comparison-log.json` y `preferences.json` en la carpeta LocalState de ese paquete, los mismos archivos que escribió la aplicación .NET. Los colores de las series siguen un modo de visión de color guardado. Los archivos recientes y las vistas guardadas de esa carpeta se vuelven a abrir aquí. Los complementos de esa carpeta se dejan en su lugar y no se cargan. Una ejecución no empaquetada no escribe esa carpeta.

- **Comparación directa** — Carga dos rastros de inferencia uno al lado del otro y observa exactamente qué cambió
- **Análisis delta canónico** — Cinco tipos de delta (ΔTc, ΔO, ΔF, ΔĀ, ΔTd) se activan solo cuando las diferencias son estadísticamente significativas
- **Preajustes de tiempo de ejecución** — El preajuste TFRT suprime automáticamente las métricas irrelevantes para que te centres en lo que importa para las cargas de trabajo de TensorFlow-TRT
- **Paquetes reproducibles** — Exporta archivos `.scbundle` con integridad SHA-256, deltas congelados y metadatos completos de procedencia
- **Modo de revisión** — Abre un paquete sin volver a calcularlo. Una coincidencia de SHA-256 es una verificación de contenido, no una firma.
- **Privacidad primero** — Cero telemetría, cero análisis, todos los datos permanecen locales a menos que los exportes explícitamente

---

## VortexKit

VortexKit es la biblioteca de visualización en `src/VortexKit`. Es parte de este repositorio. No se publica en NuGet.

Abarca la reproducción sincronizada en el tiempo, lienzos animados de SkiaSharp, vistas de comparación, superposiciones de anotaciones, exportación de SVG y PNG, y un sistema de color semántico.

---

## Comenzar

### Revisión de Rust

Desde este repositorio:

```
cargo run --manifest-path rust/Cargo.toml
```

Abre dos archivos de inferencia o dos archivos de retropropagación `run_history.json`. Un archivo de inferencia es un CSV de latencia, un JSON de referencia o un rastro de Chrome. Un `ProfilerStep` completo en ese rastro es una inferencia, y las operaciones que contiene no son muestras adicionales. Un rastro sin paso aún utiliza eventos cuyos nombres contienen TensorRT o inferencia. Un archivo de entrenamiento se dibuja como pérdida de entrenamiento. La pérdida retenida, la perplejidad y las métricas de la tarea se muestran junto a esa curva. `final_loss` se muestra como su propio número. Un par de inferencias informa ΔF y ΔO de la serie de latencia. ΔTc se informa solo cuando ambos archivos tienen un hito de estado estacionario. Sin ese hito, el último paso no se denomina tiempo de estabilización y la serie no dibuja una línea de estado estacionario. ΔTd y ΔĀ no se muestran en la página de inferencia. Los deltas de inferencia no se calculan en un historial de entrenamiento. Guardar el paquete escribe la revisión en la página. Abrir el paquete muestra esa revisión almacenada nuevamente. El hash coincide con la verificación de la fase 7.2 de .NET. Una coincidencia significa que los bytes están intactos. No es una firma.

`packaging/pack.ps1` crea el `ScalarScope_3.0.0.0_x64.msix` sin firmar a partir del binario de lanzamiento. El nombre del paquete es `mcp-tool-shop.ScalarScope`, el editor es `CN=5305D976-6952-4F00-9C21-3A5DB090359F` y la arquitectura es x64. No se carga. La copia de la tienda sigue siendo el paquete .NET anterior.

### Desde la Microsoft Store

1. Instala **ScalarScope** desde la [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (ID de la tienda: `9P3HT1PHBKQK`)
2. Haz clic en **Comparar dos ejecuciones**
3. Carga un rastro de referencia: un CSV de latencia, un JSON de referencia o un perfilador `trace.json`
4. Carga el rastro optimizado en el mismo tipo de archivo
5. Revisa los deltas en la pestaña **Comparar**
6. Exporta un `.scbundle` para compartirlo de forma reproducible

### Uso de VortexKit

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

## Características

### Análisis de delta: cinco tipos de delta canónicos

Cada comparación produce un conjunto de deltas canónicos. Cada delta se activa solo cuando la diferencia es estadísticamente significativa; los deltas irrelevantes se suprimen automáticamente.

| Delta | Nombre completo | Qué mide | Se activa cuando |
|-------|-----------|------------------|------------|
| **ΔTc** | Tiempo de convergencia | Pasos para alcanzar una latencia estable | Se alcanza el estado estacionario en pasos diferentes (separación de 3+ pasos) |
| **ΔO** | Variabilidad de la salida | Oscilación / inestabilidad en tiempo de ejecución | La puntuación del área por encima del umbral difiere más allá del ruido de fondo |
| **ΔF** | Tasa de fallos | Frecuencia de anomalías | La frecuencia o el tipo de fallos difieren entre las ejecuciones |
| **ΔĀ** | Latencia promedio | Valor métrico medio | La diferencia media es significativa (suprimida en la configuración preestablecida TFRT) |
| **ΔTd** | Duración total | Tiempo transcurrido / emergencia estructural | La duración o el inicio de la dominancia difieren (suprimido en la configuración preestablecida TFRT) |

### Configuraciones preestablecidas de tiempo de ejecución: TFRT

La configuración preestablecida integrada de **TensorFlow-TRT** (`tensorflowrt-runtime-v1`) asigna señales específicas de la inferencia (latencia, rendimiento, memoria, carga de CPU/GPU) y suprime las diferencias que solo se aplican al entrenamiento (ΔĀ, ΔTd) que no tienen sentido para la comparación de la inferencia. Los mecanismos de seguridad advierten cuando el período de calentamiento supera el 50 % de la ejecución o cuando solo están disponibles estadísticas agregadas.

### Paquetes reproducibles

Exportar los resultados como archivos `.scbundle` (ComparisonBundle v1.0.0):

- **`manifest.json`** — metadatos del paquete, versión de la aplicación, etiquetas de comparación, modo de alineación
- **`repro/repro.json`** — huellas digitales de entrada, hash de la configuración preestablecida, semilla de determinismo, información del entorno
- **`findings/deltas.json`** — diferencias canónicas con puntuaciones de confianza, anclas y tipos de desencadenantes
- **`findings/why.json`** — explicaciones legibles por humanos, mecanismos de seguridad, fragmentos de parámetros
- **`findings/summary.md`** — resumen generado automáticamente en Markdown
- **Integridad:** cada archivo tiene un hash SHA-256. El hash del paquete es una verificación del contenido, no una firma.

### Modo de revisión

Abrir cualquier `.scbundle` sin volver a calcular. Un SHA-256 coincidente es una verificación del contenido, no una firma. Las diferencias almacenadas se muestran como almacenadas.

### Marco de visualización de VortexKit

VortexKit es la biblioteca de visualización en `src/VortexKit`. Se incluye con este repositorio. No es un paquete NuGet.

| Componente | Qué hace |
|-----------|-------------|
| `PlaybackController` | Línea de tiempo compartida de 0 a 1 con reproducción/pausa/avance/bucle, configuraciones preestablecidas de velocidad (0,25x a 4x), ~60 fotogramas por segundo |
| `AnimatedCanvas` | Base abstracta `SKCanvasView` con invalidación sincronizada con el tiempo, dibujo de cuadrícula, eventos de toque/arrastre, auxiliares de coordenadas |
| `ITimeSeries<T>` / `TimeSeries<T>` | Serie temporal genérica con mapeo de índice ↔ tiempo y enumeración de rastros |
| `ExportService` | PNG de un solo fotograma, secuencias de fotogramas (con sugerencias de ffmpeg) y exportación de comparación lado a lado |
| `SvgExportService` | Exportación SVG vectorial completa con capas de Inkscape, curvas de Catmull-Rom, mapas de calor, campos vectoriales y cuatro paletas de colores (Predeterminada, Clara, Alto contraste, Publicación) |
| `IAnnotation` | Anotaciones tipadas (Fase, Advertencia, Información, Fallo, Personalizada) con base teórica y prioridad |
| `VortexColors` | Paleta de colores semántica: capas de fondo, semántica de acento, codificación de gravedad, paleta de valores propios, auxiliares de interpolación/degradado |

---

## Instalación

### Microsoft Store (recomendado)

**ID de la tienda:** `9P3HT1PHBKQK`

[Obtenga la aplicación de Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

Requiere Windows 10 (versión 17763) o posterior.

### Desde el código fuente

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

## Estructura del proyecto

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

## Pruebas

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

## Atajos de teclado

| Atajo | Acción |
|----------|--------|
| `Space` | Reproducir / Pausar |
| `Left` / `Right` | Avanzar / retroceder (1 %) |
| `Shift+Left` / `Shift+Right` | Avanzar / retroceder (0,1 %) |
| `Home` / `End` | Ir al inicio / final |
| `Up` / `+` | Aumentar la velocidad de reproducción |
| `Down` / `-` | Disminuir la velocidad de reproducción |
| `0` | Restablecer la velocidad a 1x |
| `S` o `Ctrl+S` | Intenta escribir un PNG en la carpeta de exportación de la configuración o en Documents/ScalarScope Exports cuando no se ha definido ninguna. No se muestra ninguna notificación. Ctrl+E no está asignado. |
| `1`–`6` | Solicita una vista general de las rutas, la trayectoria, los escalares, la geometría, la comparación y los fallos. No es Inicio, Comparar, Guía ni Configuración. Al presionar 1 no se abre Inicio. |
| `?` | Abrir ayuda / guía |

---

## Relacionado

- [Manual](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — La guía para la revisión
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — Resultados experimentales completos
- [CHANGELOG.md](CHANGELOG.md) — Historial de versiones
- [PRIVACY.md](PRIVACY.md) — Política de privacidad
- [ROADMAP.md](ROADMAP.md) — Un plan más antiguo no verificado, no la revisión actual

---

## Licencia

[MIT](LICENSE) — Copyright (c) 2025-2026 Proyecto ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
