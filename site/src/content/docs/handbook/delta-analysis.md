---
title: Delta Analysis
description: The five canonical delta types, detector configuration, and runtime presets.
sidebar:
  order: 2
---

Every comparison produces a set of canonical deltas. Each delta fires only when the difference is statistically meaningful — no noise, no false signals, no manual threshold tuning.

## Five canonical delta types

Deltas are ordered by causal salience, not mathematical complexity:

| Delta | Full Name | Category | What It Measures | Fires When |
|-------|-----------|----------|------------------|------------|
| **ΔF** | Failure Rate | Event | Anomaly frequency | Failure frequency or kind differs between runs |
| **ΔTc** | Convergence Time | Timing | Steps to reach stable latency | Steady-state reached at different steps (3+ step separation) |
| **ΔTd** | Total Duration | Timing | Wall-clock time / structural emergence | Dominance onset differs (suppressed in TFRT preset) |
| **ΔĀ** | Average Latency | Behavior | Mean metric value | Mean differs meaningfully (suppressed in TFRT preset) |
| **ΔO** | Output Variability | Behavior | Oscillation / runtime instability | Area-above-threshold score differs beyond noise floor |

Each delta has three possible statuses:

- **Present** — the difference is real and meaningful
- **Suppressed** — the difference is below threshold or irrelevant for the active preset
- **Indeterminate** — cannot determine (insufficient data)

## How deltas are computed

Each delta type has its own detector with configurable thresholds. Every delta includes:

- **Confidence score** (0.0 to 1.0) — how certain the difference is meaningful
- **Anchors** — specific data points and view targets that triggered the delta
- **Trigger type** — what kind of signal caused the detection (e.g., sustained, recurrence, area episode, persistence-weighted)
- **Human-readable explanation** — auto-generated text describing the finding (target: 12 words or fewer)
- **Summary sentence** — neutral, descriptive sentence for export summaries

### Convergence detection (ΔTc)

The convergence detector looks for when a signal stays within an epsilon band for a sustained window:

- **Window**: number of consecutive stable steps required (default: 5, minimum: 3)
- **Epsilon**: base tolerance band; effective epsilon = max(base epsilon, 0.5 * robust sigma)
- **Resolution**: minimum 3-step separation between runs to count as meaningful
- Confidence is a heuristic based on tail stability and noise level — it affects visual intensity but never suppresses the delta

### Stability detection (ΔO)

The stability detector uses area-above-threshold scoring with an adaptive threshold:

- Threshold adapts based on both median and sigma of curvature magnitudes
- Episodes must sustain for at least 4 steps to avoid flicker false positives
- Between-run suppression applies a delta floor of 0.05
- Within-run noise floor of 0.1 filters out trivial episodes

### Failure detection (ΔF)

Detects anomalies using a persistence window (default: 3 steps). Can trigger on norm violations, loss explosions, or other failure kinds. Reports which run failed, at what step, and what kind of failure occurred.

### Emergence detection (ΔTd)

Detects structural emergence through eigenvalue dominance. Fires when one eigenvalue exceeds k times the next (default k = 1.5) for a sustained window, or through a recurrence rule (repeated dominance segments within a rolling window).

## Runtime presets

### TFRT (TensorFlow-TRT)

The built-in TensorFlow-TRT preset (`tensorflowrt-runtime-v1`) is designed for inference comparison:

- Maps inference-specific signals: latency, throughput, memory, CPU/GPU load
- **Suppresses ΔĀ and ΔTd** — these have no meaning for inference workloads
- Active deltas use latency as the primary signal (ΔTc for stabilization time, ΔO for oscillation, ΔF for outliers)

#### TFRT guardrails

The preset raises warnings when:

- No steady-state milestone is found in the trace
- Warmup exceeds 50% of the run duration
- Only aggregated stats are available (disables time-based deltas entirely)

These guardrails appear as inline warnings in the Compare tab so you know when results may be limited.
