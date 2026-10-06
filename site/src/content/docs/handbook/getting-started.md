---
title: Getting Started
description: Install ScalarScope and compare your first two runs.
sidebar:
  order: 1
---

## The review this repo builds

The program you build here is the Rust review. From this repo:

```bash
cargo run --manifest-path rust/Cargo.toml
```

The window has four buttons: **Open path A**, **Open path B**, **Open bundle**, and **Save bundle**. An inference file is a latency CSV, a benchmark JSON, or a Chrome trace. A training file is a backpropagate `run_history.json`. The [Rust review](/scalarscope/handbook/rust-review/) page is the short guide. The Store copy is still the previous .NET package.

## Files the review opens

A latency CSV needs a latency column. `latency_ms`, `latency`, and `time_ms` are the names it reads:

```csv
step,latency_ms
0,12.5
1,11.8
```

A benchmark JSON puts samples in `results`, `iterations`, or `benchmarks`. Each sample needs `latency_ms` or `latency`:

```json
{
  "results": [
    { "latency_ms": 12.5 },
    { "latency_ms": 11.8 }
  ]
}
```

A Chrome trace uses `traceEvents`. A complete `ProfilerStep` is one inference. `dur` is microseconds:

```json
{
  "traceEvents": [
    { "name": "ProfilerStep#1", "ph": "X", "dur": 12500 }
  ]
}
```

A training file is a backpropagate `run_history.json`. The review draws `loss_history`. `final_loss` stays its own number. It is not another sample on that curve:

```json
[
  {
    "run_id": "example",
    "status": "completed",
    "loss_history": [1.2, 0.8],
    "final_loss": 0.8
  }
]
```

## Install from the Microsoft Store

The listing is still the previous .NET package. The easiest way to get that package:

1. Visit the [Microsoft Store listing](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (Store ID: `9P3HT1PHBKQK`)
2. Click Install
3. Requires Windows 10 (build 17763) or later

## Your first comparison on the Store package

These steps are the four-tab shell on the Store. They are not the Rust review buttons.

1. Open ScalarScope — the **Home** tab shows your workspace status
2. Click **Compare Two Runs** to open the Compare tab
3. Load your baseline TFRT trace (before optimization)
4. Load your optimized TFRT trace (after optimization)
5. Review deltas in the **Compare** tab — only statistically meaningful differences appear
6. Click **Export Bundle** to save a `.scbundle` for reproducible sharing

### Try the built-in example

If you do not have traces yet, click **Try Example** on the Home tab. This loads a built-in demo comparison so you can explore the interface without your own data. The demo walks through the delta analysis workflow with pre-loaded traces.

## Navigation

ScalarScope uses four tabs:

| Tab | Purpose |
|-----|---------|
| **Home** | Workspace status, recent comparisons, quick actions |
| **Compare** | Side-by-side delta comparison — the core workflow |
| **Guide** | Interpretation help and onboarding |
| **Settings** | Theme, playback, export, and accessibility preferences |

Switch tabs by clicking the tab bar. Number keys `1`–`6` request the routes overview, trajectory, scalars, geometry, compare, and failures. They do not open Home, Compare, Guide, or Settings. Pressing 1 does not open Home.

## Build from source

The review:

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --manifest-path rust/Cargo.toml
```

`dotnet run --project src/ScalarScope` builds the previous .NET shell. It is not the program the release packs.

```bash
# Prerequisites for that shell:
#   .NET 9.0 SDK (global.json pins 9.0.100)
#   dotnet workload install maui-windows

dotnet restore
dotnet build
dotnet run --project src/ScalarScope
```

## VortexKit

VortexKit is the project at `src/VortexKit`. It is not published to NuGet.

See the [VortexKit page](/scalarscope/handbook/vortexkit/) for the types.

## Running tests

```bash
# All tests
dotnet test

# Fixture smoke tests only
dotnet test --filter Category=FixtureSmoke

# Determinism tests (verifies reproducible deltas)
dotnet test --filter Category=Determinism

# With coverage
dotnet test --collect:"XPlat Code Coverage"
```

## Settings overview

Open the **Settings** tab to configure:

- **Theme**: System, Light, or Dark
- **Playback**: default speed (0.25x to 4x), auto-play on load
- **Export**: default resolution (width/height), export folder
- **Accessibility**: high contrast mode, color vision modes (deuteranopia, protanopia, tritanopia, monochrome), screen reader support, text scale (75% to 200%), large pointer
- **Session**: auto-load last session, recent files limit

The Settings tab writes these controls, and the four-tab shell reads them when it starts. The Rust review applies color vision, high contrast, and text scale. It does not offer screen reader descriptions or a larger pointer.

All preferences are stored locally and persist between sessions. If the preferences file cannot be read, ScalarScope leaves that file unchanged and Settings says so.
