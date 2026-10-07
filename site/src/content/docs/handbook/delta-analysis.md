---
title: Delta Analysis
description: The deltas ScalarScope reports, the rule behind each, and when a delta is withheld.
sidebar:
  order: 3
---

A delta names one way run B differs from run A. Each tile says whether its delta **fired**, stayed **quiet** or was **withheld**, and **Why** gives the rule, this pair's numbers and the parameters.

## Inference: three deltas

These are 3.x's rules. Where they answer differently from 2.0, the [spec](https://github.com/mcp-tool-shop-org/scalarscope/blob/main/docs/parity-and-beyond.spec.md) records the change and the evidence for it.

| Delta | What it asks | Fires when |
|---|---|---|
| **ΔF** | Did B gain runtime anomalies? | B has more steady samples beyond 5 robust deviations (1.4826 × MAD) from the median than chance allows: one-sided exact binomial test, p < 0.05 |
| **ΔO** | Did the variability change? | The 95% block-bootstrap interval on B's relative spread, (p90 − p10) / p50 of the steady samples, over A's excludes 1 |
| **ΔTc** | Did B settle at a different point? | Both runs settle, and their settle ranges do not overlap |

**Notes on the rules.**
- **Warmup.** Warmup samples are startup cost, so ΔF counts steady samples only.
- **The 3-sigma rule.** 2.0's 3-sigma anomaly rule is a choice in Settings, and the caption names the rule a review used.
- **Sample counts.** ΔF and ΔO need 20 steady samples per side. So do the ratios in the headline.
- **ΔTc and run shape.** Each run gets a shape: flat, warmup, slowdown or no steady state. Only a flat run or one that warms up has a settle point.
  - Where a run settles is a range, from the change points found at three penalties.
  - When a RunTrace states its steady-state step, that step is used as written.

**Kept off the inference page.** ΔTd and ΔĀ describe training dynamics, so they do not appear for inference.

**Warnings.** The page warns when warmup takes more than half the run, when no steady state is found, and when a file holds only aggregated statistics.

## Geometry: five deltas

For two ASPIRE training runs. These are 2.0's rules with 2.0's defaults, ported and checked against 2.0's results, with the corrections ruled on 2026-10-06. On exports whose steps are checkpoints × items, the rules ruled on 2026-10-07 apply instead (see Withheld, below, and [Geometry](/scalarscope/handbook/geometry/)).

| Delta | What it asks | Defaults |
|---|---|---|
| **ΔF** | Did a run fail persistently? | A failure that persists for 3 recorded failures or steps. The time and the kind come from the same failure. |
| **ΔTc** | Did the runs converge at different steps? | Velocity stays inside an epsilon band for 5 steps; the effective epsilon is max(0.02, 0.5 × robust sigma). Fires at 3 steps apart or more. |
| **ΔTd** | Did a dominant direction emerge at a different time? | The first eigenvalue exceeds 1.5 × the next, sustained or recurring |
| **ΔĀ** | Is one run's evaluator spectrum more concentrated? | λ1 / Σλ per step, compared over the longest sustained stretch; on checkpoint × item exports, per checkpoint, with the same sign at every one. 2.0 called it evaluator agreement. |
| **ΔO** | Did one run show sustained instability? | Curvature above an adaptive threshold for 4 steps or more; floor 0.05 between runs, 0.1 within a run |

## Withheld

A delta is **withheld** when the data cannot carry it. The tile then says why.
- **On a geometry export that states its layout** (the [export contract](/scalarscope/handbook/geometry/)):
  - ΔTc, ΔTd, ΔO and ΔF are withheld when the steps are checkpoints × items, not time.
  - ΔF is withheld when the scores were replayed or are fixed per item.
  - ΔĀ is withheld when the two runs' checkpoints do not pair, such as three against four.
- **The headline speaks only from the deltas that stand.**
