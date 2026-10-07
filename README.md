<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> Part of [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Compare two machine-learning runs, and see how sure the comparison is.** ScalarScope opens two inference traces, two training histories, or two ASPIRE geometry exports. It says how run B differs from run A, gives each difference an interval or its reason, and holds back what the data cannot support.

Version **3.1.0** is a rewrite in Rust. It is the Windows app on the Microsoft Store as [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK), as package 3.1.0.0, an update of the same listing. The package name `mcp-tool-shop.ScalarScope` and the publisher `CN=5305D976-6952-4F00-9C21-3A5DB090359F` are unchanged, so files, saved reviews and settings from 2.0 carry over.

## Trust model

- **What it reads.** The files you open. The Store package also reads and writes `comparison-log.json`, `preferences.json` and `workbench.json` in its own LocalState folder, the folder 2.0 used. An unpackaged build writes none of them.
- **What it writes.** A bundle, a picture or a session record, only to a path you pick.
- **Network.** There is no account, no telemetry and no analytics. The one connection the app can make is from the Workbench tab, and only when you press **Ask**: to a local Ollama at `127.0.0.1` on this computer. Ollama's cloud models are refused. Nothing leaves the machine.
- **Bundles.** A matching SHA-256 is a content check, not a signature. It says the bytes are intact. It does not say who wrote the file.
- **Plugins.** Plugins left in the 2.0 folder are not loaded.

---

## What it does

**Inference: two runs of latency per step.**
- **The headline** is B/A at p50, p90 and p99. Each ratio carries a 95% moving-block bootstrap interval.
- **Several runs per side** can be opened. The interval then resamples whole runs too, and it is called indicative below three runs per side.
- **A percentile without enough samples** behind it is not printed; p99 needs 368.
- **Three deltas,** each with a tile that says whether it fired, stayed quiet or was withheld, and why:
  - **ΔF (new anomalies)** counts steady samples beyond 5 robust deviations. It fires only when B's excess is beyond chance.
  - **ΔO (variability)** fires when the interval on B's relative spread over A's excludes 1.
  - **ΔTc (stabilization)** gives each run a shape. It fires only when both settle and their settle ranges do not overlap.
- **Six views:** Series, Warmup, Distribution with a threshold you drag, Difference by percentile, Spectrum and Heat map. None animates.

**Training.** A backpropagate `run_history.json` is drawn as training loss, with held-out loss, perplexity and task metrics beside it. Inference deltas are not computed on it.

**Geometry: two ASPIRE training runs.**
- **What it reads:** the geometry export that [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) writes, with its trajectory, evaluator scores, eigen spectrum and failures.
- **Five deltas:** ΔF, ΔTc, ΔTd, ΔĀ (spectrum concentration) and ΔO. They are ported from 2.0 and checked against 2.0's own results, with the corrections recorded in [the spec](docs/parity-and-beyond.spec.md).
- **Export contract (schema 1.1).** An export can state that its steps are checkpoints × items rather than time, or that its scores were replayed. When it does, the deltas that read time or score dips are withheld with that reason, and the headline speaks only from the deltas that stand. Older exports read as before.

**Workbench.**
- **Open many runs** that differ in a setting, such as batch size or precision.
- **Ask:** a local model that can call tools measures the runs with formulas and proposes what each setting does.
- **The program sets every number and verdict.** A hypothesis is tested by an exact rank test on each set of runs. Verdicts across sets come only at checkpoints, by e-BH at a 5% false discovery rate.
- **Your call first.** You can write your own call before asking. The model's note is shown under the verdicts, labelled as its words.

**History.** In the Store package, the comparisons you finish are grouped by side B's dataset and model. Each measure is drawn across those reviews, with the points where its level shifted. A code or environment change beside a shift is named, and called a coincidence, not a cause. The log keeps the last 40 reviews.

**Bundles and saved state.**
- **Save bundle** writes the review as a `.scbundle`. **Open bundle** shows it exactly as saved, in review mode.
- **2.0 bundles** open: comparison bundles with their stored deltas, and inference reviews marked unverified.
- **2.0's settings** carry over: theme, color-vision palettes, high contrast, text scale, the recent-file limit and the anomaly rule.
- **Export** the current view as SVG, or the window as PNG.

---

## Quick start

### From the Microsoft Store

1. Install **ScalarScope** from the [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK). It needs Windows 10 version 1809 (build 17763) or later, x64.
2. On **Welcome**, click **Try the sample comparison**. Or click **Compare two runs** and open path A and path B.
3. Read the headline, then click a tile for **Why** and **Show me**.

Sample files to try, an inference pair, a geometry pair and a folder of runs for the Workbench, are in [`samples/`](samples/). [TESTING.md](TESTING.md) says which to open where.

### From source

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` opens a pair at launch.

### What you can open

| Kind | Files |
|---|---|
| Inference | A latency CSV, a benchmark JSON, a Chrome or PyTorch profiler trace (`.json` or `.json.gz`), a runtime log, a ScalarScope RunTrace JSON, or a run folder |
| Training | A backpropagate `run_history.json` |
| Geometry | An ASPIRE geometry export (aspire-si, schema 1.x) |
| Review | A `.scbundle` from 3.x or 2.0 |

Getting Started in the [handbook](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/) has a short example of each file.

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `F1` | Guide |
| `Ctrl+,` | Settings |
| `Ctrl+H` | Welcome |
| `1`–`6` | On Compare: Series, Warmup, Distribution, Difference, Spectrum, Heat map |
| `Esc` | Close the Why panel |

---

## Testing

```bash
# The review: 217 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (134 tests)
dotnet test tests/ScalarScope.FixtureTests
```

The .NET project in `src/ScalarScope` is the 2.0 app. It stays in the repo as the reference that 3.x is checked against. The Store package is built from `rust/` by `packaging/pack.ps1`, and the MAUI project refuses to be published as that upload.

---

## Project structure

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

## Related

- [Handbook](https://mcp-tool-shop-org.github.io/scalarscope/handbook/): the guide to the review
- [Parity and beyond](docs/parity-and-beyond.spec.md): what 3.x keeps from 2.0, what it changes and why, with sources
- [CHANGELOG.md](CHANGELOG.md): release history
- [PRIVACY.md](PRIVACY.md): privacy policy
- [TESTING.md](TESTING.md): how to test this release
- [The workbench](https://github.com/mcp-tool-shop-org/runforge): shared with RunForge

---

## License

[MIT](LICENSE). Copyright (c) 2025-2026 ScalarScope Project (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
