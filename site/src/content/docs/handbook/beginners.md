---
title: Beginners
description: Step-by-step guide for new users of ScalarScope.
sidebar:
  order: 99
---

New to ScalarScope? This page walks you through the core concepts and workflows, starting from zero.

## What does this tool do?

ScalarScope compares two ML inference runs side by side and tells you exactly what changed. Instead of manually scrolling through logs looking for differences, an inference comparison shows three deltas, and only when the difference is statistically meaningful. You get a clear answer: "these things changed, and here's the evidence."

The tool is built for TensorFlow-TRT inference workloads but the underlying delta analysis works on any pair of time-series traces. Results are exportable as `.scbundle` archives. A matching SHA-256 is a content check, not a signature.

## Installation

### Microsoft Store (recommended)

1. Open the [Microsoft Store listing](https://apps.microsoft.com/detail/9P3HT1PHBKQK)
2. Click **Install**
3. Requires Windows 10 (build 17763) or later

### From source

```bash
# Install .NET 9.0 SDK, then:
dotnet workload install maui-windows

git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet restore
dotnet build
dotnet run --project src/ScalarScope
```

## Your first comparison

Follow these steps to compare two inference traces:

1. **Launch ScalarScope** — the Home tab shows your workspace status
2. **Click "Compare Two Runs"** — this opens the Compare tab
3. **Load the baseline trace** — select the TFRT trace from before your optimization
4. **Load the optimized trace** — select the TFRT trace from after your optimization
5. **Read the deltas** — the Delta Zone shows which differences are meaningful, with confidence scores and explanations
6. **Click "Show Me"** on any delta to jump to the relevant view and see the evidence highlighted
7. **Export a bundle** — click Export Bundle to create a `.scbundle` file you can share with your team

If you do not have your own traces yet, click **Try Example** on the Home tab. This loads a built-in demo comparison so you can explore the full interface.

## Key concepts

### Deltas

A delta represents a statistically meaningful difference between two runs. On an inference comparison this page shows three:

- **ΔTc (Convergence Time)** — steps to stable latency
- **ΔO (Output Variability)** — runtime instability
- **ΔF (Failure Rate)** — failure rate

ΔĀ (average latency) and ΔTd (total duration) stay off the inference page.

Each delta includes a confidence score (0 to 1) and a human-readable explanation. Deltas that do not reach statistical significance are automatically suppressed — you never see noise.

### Runtime presets

The TFRT preset is designed for TensorFlow-TRT inference workloads. It maps inference signals (latency, throughput, memory, CPU/GPU load) and suppresses training-only deltas (ΔĀ and ΔTd) that are irrelevant for inference comparison.

### Bundles

A `.scbundle` is a self-contained archive of your comparison results. Every file inside is hashed with SHA-256. A matching bundle hash is a content check, not a signature. When a colleague opens your bundle, Review Mode activates automatically — they see the stored results without recomputing anything.

### Alignment

When two runs have different lengths, ScalarScope aligns them before comparison. Three alignment modes are available: step-based (default for same-length runs), convergence-onset (aligns when signals stabilize), and first-instability (aligns at the first oscillation event).

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Play / Pause playback |
| `Left` / `Right` | Step backward / forward (1%) |
| `Shift+Left` / `Shift+Right` | Fine step (0.1%) |
| `Home` / `End` | Jump to start / end |
| `Up` / `+` | Increase playback speed |
| `Down` / `-` | Decrease playback speed |
| `0` | Reset speed to 1x |
| `S` or `Ctrl+S` | Tries to write a PNG to the export folder from Settings, or Documents/ScalarScope Exports when none is set. No notification is shown. Ctrl+E is not mapped. |
| `1`–`6` | Request routes overview, trajectory, scalars, geometry, compare, and failures. Not Home, Compare, Guide, or Settings. Pressing 1 does not open Home. |
| `?` | Open Guide tab |

## Configuration

Open the **Settings** tab to adjust:

- **Theme** — System, Light, or Dark
- **Playback** — default speed, auto-play on load
- **Export** — default output resolution and export folder
- **Accessibility** — high contrast mode, color vision simulation (deuteranopia, protanopia, tritanopia, monochrome), screen reader support, adjustable text scale (75% to 200%), large pointer mode
- **Session** — auto-load last session, recent files limit (5, 10, or 20)

The Settings tab writes these controls, and the four-tab shell reads them when it starts. The Rust review applies color vision, high contrast, and text scale. It does not offer screen reader descriptions or a larger pointer.

All settings are stored locally in your app data directory and persist between sessions. If the preferences file cannot be read, ScalarScope leaves that file unchanged and Settings says so. Click **Reset All Settings** to return everything to defaults.

## FAQ

**Q: Do I need a GPU to run ScalarScope?**
No. ScalarScope is a visualization and analysis tool. It reads inference traces and computes deltas on the CPU. No GPU is required.

**Q: What file formats does ScalarScope accept?**
ScalarScope uses the RunTrace format (JSON-based, schema version 1.0.0). The TFRT connector reads TensorFlow-TRT log directories. You can also open `.scbundle` review bundles or load the built-in demo.

**Q: Does ScalarScope send any data externally?**
No. ScalarScope has zero telemetry and zero analytics. All data stays local unless you explicitly export a bundle and share it yourself. See the [Privacy Policy](https://github.com/mcp-tool-shop-org/scalarscope/blob/main/PRIVACY.md) for details.

**Q: Can I use VortexKit without ScalarScope?**
VortexKit lives in `src/VortexKit` in this repo. It is not a NuGet package. See the [VortexKit page](/scalarscope/handbook/vortexkit/) for the types.

**Q: What happens if a bundle has been tampered with?**
Review Mode verifies every file against its embedded SHA-256 hash. If any file has been modified, the integrity check fails and a warning is displayed. The bundle contents are still shown, but the verification status is flagged.

**Q: How do I report a bug?**
Open an issue at [github.com/mcp-tool-shop-org/scalarscope/issues](https://github.com/mcp-tool-shop-org/scalarscope/issues/new).
