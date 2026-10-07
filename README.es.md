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

**Compara dos ejecuciones de aprendizaje automático y observa qué tan seguro es el resultado de la comparación.** ScalarScope abre dos trazas de inferencia, dos historiales de entrenamiento o dos exportaciones de geometría ASPIRE. Indica en qué se diferencia la ejecución B de la ejecución A, asigna a cada diferencia un intervalo o su razón, y omite lo que los datos no pueden respaldar.

ScalarScope 3.x es una reescritura en Rust. La versión **3.1.1** es la aplicación para Windows en Microsoft Store, con el identificador [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK), paquete 3.1.1.0, una actualización de la misma entrada. El nombre del paquete `mcp-tool-shop.ScalarScope` y el editor `CN=5305D976-6952-4F00-9C21-3A5DB090359F` no cambian, por lo que los archivos, las reseñas guardadas y la configuración de la versión 2.0 se conservan.

## Modelo de confianza

- **Qué lee.** Los archivos que abres. El paquete de la tienda también lee y escribe `comparison-log.json`, `preferences.json` y `workbench.json` en su propia carpeta LocalState, la misma carpeta que usaba la versión 2.0. Una versión no empaquetada no escribe ninguno de ellos.
- **Qué escribe.** Un paquete (bundle), una imagen o un registro de sesión, solo en una ruta que elijas.
- **Red.** No hay ninguna cuenta, ni telemetría ni análisis. La única conexión que la aplicación puede establecer es desde la pestaña Workbench, y solo cuando presionas **Preguntar**: a un Ollama local en `127.0.0.1` en este equipo. Los modelos en la nube de Ollama se rechazan. Nada sale de la máquina.
- **Paquetes (bundles).** Un paquete contiene las ejecuciones que compara: compártalo solo si compartiría esas ejecuciones. Una coincidencia de SHA-256 es una verificación de contenido, no una firma. Indica que los bytes están intactos. No indica quién escribió el archivo.
- **Complementos.** Los complementos que quedaron en la carpeta de la versión 2.0 no se cargan.

---

## Qué hace

**Inferencia: dos ejecuciones de latencia por paso.**
- **El titular** es B/A en p50, p90 y p99. Cada relación tiene un intervalo de bootstrap de bloques móviles del 95%.
- **Se pueden abrir varias ejecuciones por lado.** El intervalo luego vuelve a muestrear también las ejecuciones completas, y se denomina indicativo cuando hay menos de tres ejecuciones por lado.
- **Un percentil sin suficientes muestras** no se imprime; p99 necesita 368.
- **Tres deltas,** cada uno con una ficha que indica si se activó, permaneció inactivo o se omitió, y por qué:
- **ΔF (nuevas anomalías)** cuenta las muestras estables más allá de 5 desviaciones robustas. Se activa solo cuando el exceso de B está más allá de lo que se esperaría por azar.
- **ΔO (variabilidad)** se activa cuando el intervalo de la dispersión relativa de B sobre A excluye 1.
- **ΔTc (estabilización)** asigna una forma a cada ejecución. Se activa solo cuando ambas se estabilizan y sus rangos de estabilización no se superponen.
- **Seis vistas:** Serie, Calentamiento, Distribución con un umbral que puedes arrastrar, Diferencia por percentil, Espectro y Mapa de calor. Ninguna se anima.

**Entrenamiento.** Una retropropagación `run_history.json` se dibuja como pérdida de entrenamiento, con la pérdida retenida, la perplejidad y las métricas de la tarea a su lado. Los deltas de inferencia no se calculan sobre ella.

**Geometría: dos ejecuciones de entrenamiento ASPIRE.**
- **Qué lee:** la exportación de geometría que [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) escribe, con su trayectoria, puntuaciones del evaluador, espectro propio y fallos.
- **Cinco deltas:** ΔF, ΔTc, ΔTd, ΔĀ (concentración del espectro) y ΔO. Se portan de la versión 2.0 y se verifican con los resultados de la versión 2.0, con las correcciones registradas en [la especificación](docs/parity-and-beyond.spec.md).
- **Contrato de exportación (esquema 1.1).** Una exportación puede indicar que sus pasos son puntos de control × elementos en lugar de tiempo, o que sus puntuaciones se reprodujeron o son fijas por elemento. Cuando lo hace, los deltas que dependen del orden de los pasos o de las caídas de puntuación se omiten con esa razón, y ΔĀ se compara punto de control por punto de control. Cuando las puntuaciones son fijas por elemento, ΔĀ indica que compara los evaluadores, no las ejecuciones. El titular solo se basa en los deltas que se muestran. Las exportaciones más antiguas se leen como antes.

**Workbench.**
- **Abre varias ejecuciones** que difieren en una configuración, como el tamaño del lote o la precisión.
- **Preguntar:** un modelo local que puede llamar a herramientas mide las ejecuciones con fórmulas y propone qué hace cada configuración.
- **El programa establece cada número y veredicto.** Una hipótesis se prueba mediante una prueba de rango exacta en cada conjunto de ejecuciones. Los veredictos entre conjuntos solo se obtienen en los puntos de control, mediante e-BH con una tasa de descubrimiento falso del 5%.
- **Tú decides primero.** Puedes escribir tu propia consulta antes de preguntar. La nota del modelo se muestra debajo de los veredictos, etiquetada como sus palabras.

**Historial.** En el paquete de la tienda, las comparaciones que completas se agrupan por el conjunto de datos y el modelo del lado B. Cada medida se dibuja en esas reseñas, con los puntos donde su nivel cambió. Se nombra un cambio de código o entorno junto a un cambio y se denomina coincidencia, no causa. El registro conserva las últimas 40 reseñas.

**Paquetes (bundles) y estado guardado.**
- **Save bundle** escribe la reseña como un `.scbundle`. **Open bundle** verifica su hash, lo rechaza si hay una discrepancia y lo muestra exactamente como se guardó, en modo de reseña. Si las reglas de hoy leen una reseña de geometría guardada de manera diferente, esa lectura se muestra debajo, etiquetada.
- **Los paquetes de la versión 2.0** se abren: paquetes de comparación con sus deltas almacenados y reseñas de inferencia marcadas como no verificadas.
- **La configuración de la versión 2.0** se conserva: tema, paletas de visión de color, alto contraste, escala de texto, el límite de archivos recientes y la regla de anomalías.
- **Exporta** la vista actual como SVG o la ventana como PNG.

---

## Comienzo rápido

### Desde la Microsoft Store

1. Instala **ScalarScope** desde [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK). Necesita Windows 10 versión 1809 (compilación 17763) o posterior, x64.
2. En **Bienvenido**, haz clic en **Prueba la comparación de ejemplo**. O haz clic en **Comparar dos ejecuciones** y abre la ruta A y la ruta B.
3. Lee el titular y luego haz clic en una ficha para ver **Por qué** y **Muéstrame**.

Los archivos de ejemplo para probar, un par de inferencia, un par de geometría y una carpeta de ejecuciones para Workbench, están en [`samples/`](samples/). [TESTING.md](TESTING.md) indica cuáles abrir y dónde.

### Desde el código fuente

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` abre un par al iniciarse.

### Qué puedes abrir

| Tipo | Archivos |
|---|---|
| Inferencia | Un archivo CSV de latencia, un archivo JSON de referencia, un registro de perfilador de Chrome o PyTorch (`.json` o `.json.gz`), un registro de tiempo de ejecución, un archivo JSON de ScalarScope RunTrace o una carpeta de ejecución. |
| Entrenamiento | Una retropropagación `run_history.json` |
| Geometría | Una exportación de geometría ASPIRE (aspire-si, esquema 1.x) |
| Revisión | Un `.scbundle` de 3.x o 2.0 |

La sección "Cómo empezar" del [manual](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/) contiene un ejemplo breve de cada archivo.

---

## Atajos de teclado

| Atajo | Acción |
|---|---|
| `F1` | Guía |
| `Ctrl+,` | Configuración |
| `Ctrl+H` | Bienvenido |
| `1`–`6` | En "Comparar": Series, Calentamiento, Distribución, Diferencia, Espectro, Mapa de calor |
| `Esc` | Cerrar el panel "Por qué" |

---

## Pruebas

```bash
# The review: 229 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (135 tests)
dotnet test tests/ScalarScope.FixtureTests
```

El proyecto .NET en `src/ScalarScope` es la aplicación 2.0. Permanece en el repositorio como la referencia con la que se compara 3.x. El paquete Store se crea a partir de `rust/` por `packaging/pack.ps1`, y el proyecto MAUI se niega a publicarse con esa carga.

---

## Estructura del proyecto

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

## Relacionado

- [Manual](https://mcp-tool-shop-org.github.io/scalarscope/handbook/): la guía para la revisión
- [Paridad y más allá](docs/parity-and-beyond.spec.md): qué conserva 3.x de 2.0, qué cambia y por qué, con fuentes
- [CHANGELOG.md](CHANGELOG.md): historial de versiones
- [PRIVACY.md](PRIVACY.md): política de privacidad
- [TESTING.md](TESTING.md): cómo probar esta versión
- [El entorno de trabajo](https://github.com/mcp-tool-shop-org/runforge): compartido con RunForge

---

## Licencia

[MIT](LICENSE). Copyright (c) 2025-2026 Proyecto ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
