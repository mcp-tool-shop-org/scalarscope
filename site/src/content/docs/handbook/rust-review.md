---
title: The review
description: Compare in ScalarScope 3.1.0, the files it opens, and how it is built.
sidebar:
  order: 2
---

ScalarScope 3.1.0 is the Rust program in `rust/`. It is the Microsoft Store package `mcp-tool-shop.ScalarScope`, version 3.1.0.0, an update of listing `9P3HT1PHBKQK`.

## Run it

Install it from the Store. Or, from this repo:

```bash
cargo run --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` opens a pair at launch.

**Compare** has four buttons: **Open path A**, **Open path B**, **Open bundle** and **Save bundle**.
- **Folder A** and **Folder B** open a run folder, or a folder of runs that are repeats of one configuration.
- **An inference file** is a latency CSV, a benchmark JSON, a Chrome or PyTorch profiler trace, a runtime log or a RunTrace JSON.
- **A training file** is a backpropagate `run_history.json`. It is drawn as training loss. Loss stays loss: inference deltas are not computed on a training history.
- **A geometry file** is an ASPIRE export from aspire-si; see [Geometry](/scalarscope/handbook/geometry/).

Short examples of each file are on [Getting Started](/scalarscope/handbook/getting-started/).

## What an inference pair reports

- **The headline** is B/A at p50, p90 and p99, each with a 95% moving-block bootstrap interval.
  - With one run per side, it says the interval covers variation within that run only, and calls itself indicative.
  - A percentile without enough samples behind it is named and not printed; p99 needs 368.
  - A ratio needs 20 steady samples in every run.
- **Three tiles,** ΔF, ΔTc and ΔO, each say whether the delta fired, stayed quiet or was withheld. **Why** gives the rule, this pair's numbers and the parameters. **Show me** moves to the view the delta is about. The rules are on [Delta Analysis](/scalarscope/handbook/delta-analysis/).
- **Six views,** chosen with `1` to `6`:
  - **Series:** latency from the steady state, with the p10–p90 band and the anomaly marks.
  - **Warmup:** each run from its first sample, with where it settles shaded.
  - **Distribution:** the CDF with a threshold you drag, and 20 quantile dots.
  - **Difference:** B − A by percentile, with intervals.
  - **Spectrum:** a log tail axis.
  - **Heat map:** step × latency on a shared scale.

A complete `ProfilerStep` in a Chrome trace is one inference. The ops inside that step are not extra samples.

## Saving and exporting

- **Save bundle** writes the review as a `.scbundle`. The hash is SHA-256 of the archived file bytes. A matching hash is a content check, not a signature.
- **Export SVG** writes the current view as a vector drawing. **Export PNG** saves a picture of the window.

## The package

`packaging/pack.ps1` builds `ScalarScope_3.1.0.0_x64.msix`, `ScalarScope_3.1.0.0_Store.msixupload` and `checksums.txt` from the release binary. It reads the version from `packaging/AppxManifest.xml`. Partner Center signs the upload. The package name is `mcp-tool-shop.ScalarScope`, the publisher is `CN=5305D976-6952-4F00-9C21-3A5DB090359F`, and the architecture is x64.
