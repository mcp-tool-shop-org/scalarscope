---
title: Rust review
description: The program this repo builds, and the files it opens.
sidebar:
  order: 2
---

The app you build from this repo is the Rust program in `rust/`. The release workflow packs that program as the unsigned 3.0.0.0 MSIX. The Store copy is still the previous .NET package until that upload.

## Run it

From this repo:

```bash
cargo run --manifest-path rust/Cargo.toml
```

The window has four buttons: **Open path A**, **Open path B**, **Open bundle**, and **Save bundle**.

Open path A and Open path B take one file each. An inference file is a latency CSV, a benchmark JSON, or a Chrome trace. A training file is a backpropagate `run_history.json`, and it is drawn as training loss. Loss stays loss. Inference deltas are not computed on a training history.

Save bundle writes the review on the page. Open bundle shows that stored review again. The hash is SHA-256 of the archived file bytes. A matching hash is a content check, not a signature.

Short examples of the four files are on [Getting Started](/scalarscope/handbook/getting-started/).

## What an inference pair reports

An inference pair reports ΔF and ΔO from the latency series. ΔTc is reported only when both files have a steady-state milestone. Without that milestone the last step is not called a stabilization time. ΔTd and ΔĀ stay off the inference page.

A complete `ProfilerStep` in a Chrome trace is one inference. The ops inside that step are not extra samples. A trace with no step still uses events whose names contain TensorRT or inference.

## The package

`packaging/pack.ps1` builds the unsigned `ScalarScope_3.0.0.0_x64.msix`, the `ScalarScope_3.0.0.0_Store.msixupload`, and `checksums.txt`. The package name is `mcp-tool-shop.ScalarScope`, the publisher is `CN=5305D976-6952-4F00-9C21-3A5DB090359F`, and the architecture is x64. It is not uploaded. The copy on the Store is still the previous .NET package. That package is the four-tab shell: Home, Compare, Guide, and Settings.
