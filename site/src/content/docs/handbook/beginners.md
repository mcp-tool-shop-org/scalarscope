---
title: Beginners
description: Step-by-step guide for new users of ScalarScope.
sidebar:
  order: 99
---

New to ScalarScope? This page walks you through the core ideas, starting from zero.

## What does this tool do?

ScalarScope compares two machine-learning runs and tells you how run B differs from run A. It also tells you how sure that answer is. Instead of scrolling through two logs, you get:
- a headline ratio with an interval;
- a tile for each kind of difference, saying whether it fired and why;
- views that show the evidence.

What the data cannot support is held back, with the reason.

## Installation

### Microsoft Store

1. Open the [Microsoft Store listing](https://apps.microsoft.com/detail/9P3HT1PHBKQK).
2. Click **Install**. ScalarScope needs Windows 10 version 1809 (build 17763) or later, x64.

### From source

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --manifest-path rust/Cargo.toml
```

## Your first comparison

1. ScalarScope opens on **Welcome**. Click **Try the sample comparison**. Two built-in runs of one model, before and after an optimization, open on **Compare**.
2. **Read the headline.** "B/A p50 0.65 (0.65–0.66)" means B's median latency is 65% of A's, and the 95% interval runs from 0.65 to 0.66. With one run per side the headline calls itself indicative: run-to-run variation is not measured.
3. **Read the tiles.** ΔF, ΔTc and ΔO each say fired, quiet or withheld. Click one for **Why**, and **Show me** to move to the evidence.
4. **Switch views** with `1` to `6`: Series, Warmup, Distribution, Difference, Spectrum, Heat map.
5. **Keep the review** with **Save bundle**. **Open bundle** shows it again exactly as saved.

To compare your own runs, use **Open path A** and **Open path B**. ScalarScope opens:
- a latency CSV;
- a benchmark JSON;
- a Chrome profiler trace;
- a runtime log or a RunTrace JSON;
- a backpropagate `run_history.json`;
- an ASPIRE geometry export.

[Getting Started](/scalarscope/handbook/getting-started/) has a short example of each.

## Key ideas

### Intervals, not just numbers

A ratio of 0.65 could be noise. The interval says how much the ratio would move if the run were repeated. ScalarScope estimates it by resampling the run in blocks, because neighbouring samples are not independent. Open several runs per side, three or more, and the interval covers run-to-run variation too.

### Deltas

A delta is one kind of difference: new anomalies (ΔF), a different settle point (ΔTc), or different variability (ΔO). Each fires only on its stated rule. [Delta Analysis](/scalarscope/handbook/delta-analysis/) gives the rules.

### Withheld

When the data cannot carry a reading, ScalarScope says so instead of guessing. For example:
- a percentile with too few samples behind it;
- a geometry delta that would read time on steps that are not time.

### Bundles

A `.scbundle` is a self-contained archive of a review. A matching SHA-256 is a content check, not a signature: it says the bytes are intact, not who wrote them.

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `F1` | Guide |
| `Ctrl+,` | Settings |
| `Ctrl+H` | Welcome |
| `1`–`6` | On Compare: Series, Warmup, Distribution, Difference, Spectrum, Heat map |
| `Esc` | Close the Why panel |

## Settings

The **Settings** tab has the theme (Follow Windows, Light or Dark), color-vision palettes, high contrast, text scale, the recent-files limit and the anomaly rule. 2.0's choices carry over.

Accessibility in the Rust review: it applies color vision, high contrast and text scale. It does not offer screen reader descriptions or a larger pointer; 2.0's settings for those are kept in the file but not applied. If the preferences file cannot be read, ScalarScope leaves that file unchanged and Settings says so.

## FAQ

**Q: Do I need a GPU?**
No. ScalarScope reads the files and computes on the CPU. The Workbench's Ask uses a local model through Ollama, which may use your GPU, but nothing else in the app needs it.

**Q: What file formats does ScalarScope accept?**
A latency CSV, a benchmark JSON, a Chrome profiler trace, a runtime log, a RunTrace JSON, a backpropagate `run_history.json`, an ASPIRE geometry export, and a `.scbundle` from 3.x or 2.0. [Getting Started](/scalarscope/handbook/getting-started/) has one short example of each.

**Q: Does ScalarScope send any data?**
No. There is no telemetry, no analytics and no account. The only connection the app can make is the Workbench's Ask, to a local Ollama at `127.0.0.1` on this computer. Cloud models are refused. See the [Privacy Policy](https://github.com/mcp-tool-shop-org/scalarscope/blob/main/PRIVACY.md).

**Q: I used ScalarScope 2.0. What carries over?**
Your settings, your recent files and your saved bundles. 2.0 comparison bundles open with their stored deltas. 2.0 inference reviews open marked as unverified, because 2.0 saved them without a content check.

**Q: What happens if a bundle has been changed?**
The SHA-256 no longer matches, and ScalarScope does not open it: the page says the bundle hash does not match these bytes. A 2.0 inference review, which 2.0 saved without a content check, is the one exception: it opens marked as unverified.

**Q: How do I report a bug?**
Open an issue at [github.com/mcp-tool-shop-org/scalarscope/issues](https://github.com/mcp-tool-shop-org/scalarscope/issues/new), or use **Report an issue** in Settings.
