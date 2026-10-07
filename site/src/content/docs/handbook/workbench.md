---
title: Workbench
description: Weigh a setting across many runs with a local model, while the program sets every verdict.
sidebar:
  order: 5
---

The **Workbench** tab looks at many inference runs that differ in a setting, such as batch size, precision, TensorRT, threads, CUDA graphs or input shape. It asks what each setting does.

## Runs and knobs

Click **Open runs…** and pick run files, or use the runs being compared on Compare. A run's settings, its knobs, are read from the first place that has each:
1. a `knobs` object in the RunTrace `metadata`;
2. a `knobs.json` beside the run;
3. `key=value` pairs in the folder name, such as `batch=8_precision=fp16`.

The page says which knobs vary and where each was read. A knob every run shares cannot be weighed on these runs, and the page says so.

## Measures

Formulas read the latency measures:
- `p50`, `p90` and `p99` of the steady samples. A percentile without enough samples is refused, as on the headline.
- `p99_over_p50` and `steady_mean`.
- `warmup_cost`: latency above the steady median before the steady state, in ms × steps.
- `anomaly_rate`, under the anomaly rule chosen in Settings.
- `throughput_p50` and `memory_peak`.
- Windows over steps: `quantile_between(a, b, q)` and `mean_between(a, b)`.

Type one into **Try a formula** to see its value on every run. A measure a run cannot supply is refused with its reason, never shown as zero.

## Ask

**Ask** runs a session with a local model that can call tools, through Ollama at `127.0.0.1` on this computer. Cloud models are refused. If no local model can call tools, the page says so.

**What the model does.** It measures, builds formula tools, and proposes hypotheses: a knob, a formula and a direction, with its reason in words.

**What the program does.** It computes every number and sets every state:
- **On one set of runs,** a hypothesis is not testable, confounded, inconclusive, passes its test, or goes the other way. The test is an exact rank test.
- **Across sets of runs,** evidence gathers as e-values. Verdicts come only at checkpoints, one every five new sets of runs, by e-BH at a 5% false discovery rate.

**The order of the page.**
1. The program's verdicts come first, with how many hypotheses were tried and how many hold.
2. Then a box for your own call, written before you ask.
3. Then the model's note, labelled as the model's words, a proposal and not a measurement.
4. Then the measures it never looked at, and each of its calls with the program's answer.

**Save session record** writes the model, its digest, every call and answer, and no file paths.

## What is kept

The Store package keeps the learned tools, the hypotheses and the checkpoints in `workbench.json` in its own app folder. The file holds no paths. An unpackaged build keeps nothing after it closes.

The workbench is shared with [RunForge](https://github.com/mcp-tool-shop-org/runforge).
