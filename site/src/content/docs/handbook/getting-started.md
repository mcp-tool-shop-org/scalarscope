---
title: Getting Started
description: Install ScalarScope and compare your first two runs.
sidebar:
  order: 1
---

## Install

1. Visit the [Microsoft Store listing](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (Store ID: `9P3HT1PHBKQK`).
2. Click Install. ScalarScope needs Windows 10 version 1809 (build 17763) or later, x64.

Or build it from this repo:

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --manifest-path rust/Cargo.toml
```

## Your first comparison

1. ScalarScope opens on **Welcome**. Click **Try the sample comparison** to load two built-in runs, or **Compare two runs** to open your own.
2. On **Compare**, click **Open path A** and **Open path B**. Each takes a file of the kinds below. **Folder A** and **Folder B** take a run folder.
3. Read the headline: how B compares with A at p50, p90 and p99, each with an interval.
4. Click a tile for **Why**, then **Show me** to move to the evidence. Press `1` to `6` to switch views.
5. **Save bundle** keeps the review as a `.scbundle`, and **Open bundle** shows it again exactly as saved.

The repo's `samples/` folder has an inference pair, a geometry pair and a folder of runs for the Workbench. `TESTING.md` says which to open where. [The review](/scalarscope/handbook/rust-review/) explains each part of Compare.

## The tabs

| Tab | Purpose |
|-----|---------|
| **Welcome** | The sample comparison, recent reviews and what the review gives |
| **Compare** | Two runs side by side: the headline, the tiles and the views |
| **Workbench** | Many runs that differ in a setting, weighed with a local model ([Workbench](/scalarscope/handbook/workbench/)) |
| **History** | A project's measures across its logged reviews ([History](/scalarscope/handbook/history/)) |
| **Guide** | What each part means, with search |
| **Settings** | Theme, color vision, high contrast, text scale, recent files and the anomaly rule |

`F1` opens Guide, `Ctrl+,` opens Settings and `Ctrl+H` opens Welcome.

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

## Settings

The **Settings** tab writes the 2.0 `preferences.json` keys, so a 2.0 user's choices carry over. Every key it does not use is kept. It covers:
- theme (Follow Windows, Light or Dark);
- series colors for color-vision modes;
- high contrast and text scale;
- the recent-files limit and Clear recent files;
- the anomaly rule: robust deviations, or 2.0's 3-sigma rule.

The review's caption names the rule a review used. About gives the version, the privacy statement, and links to report an issue and to the source.

Accessibility in the Rust review: it applies color vision, high contrast and text scale. It does not offer screen reader descriptions or a larger pointer; 2.0's settings for those are kept in the file but not applied. If the preferences file cannot be read, ScalarScope leaves that file unchanged and Settings says so.

## Running tests

```bash
cd rust
cargo test

# The 2.0 oracles and the version surfaces
dotnet test tests/ScalarScope.FixtureTests
```

The .NET app in `src/ScalarScope` is 2.0. It stays in the repo as the reference that 3.x is checked against.
