# ScalarScope 3: parity and beyond

Status: decisions D1–D4 taken by the director on 2026-10-06. Nothing here is built yet.

ScalarScope 3 is the Rust review in `rust/`. It replaces the .NET 2.0 app on the Store listing 9P3HT1PHBKQK, so every current user gets it as an update. This spec says what 3 must do before that update ships, and what it should do beyond 2.0. Three things set the bar:

1. **Nothing a 2.0 user relies on goes missing.** Their files open, their saved reviews open, and their settings carry over.
2. **The numbers are sound.** A comparison says how sure it is, and the program withholds what the data cannot support.
3. **A workbench sidecar**, like RunForge's: a local model builds measures and proposes what each knob does, and the program gathers the evidence.

## Decisions (director, 2026-10-06)

| # | Question | Ruling |
|---|---|---|
| D1 | Port the ASPIRE geometry half: training-dynamics JSON, trajectories, eigen spectrum, evaluator alignment, failures. | **Yes, with `aspire-ai` as its source** (`prototypes/packages/aspire-ai`). The exporter that 2.0 documents (`aspire.export.geometry_export` in `aspire-engine`) no longer exists, and `aspire-ai` records only per-epoch losses and teacher scores. Phase 4 therefore has two parts. First, a geometry exporter in `aspire-ai` writes the 2.0 `GeometryRun` format. Second, ScalarScope imports it and draws static views, with playback optional (V7). |
| D2 | Share RunForge's workbench code through a new crate instead of copying it. | **Yes.** The formula language, hypotheses, e-value evidence and Ollama loop move to a shared crate behind a small `Run` interface (named sample series plus a recipe of knobs). The change goes to runforge by pull request, with its tests carried over. |
| D3 | ΔTc in 3.0 has a 3-step floor (PR #1). The 2.0 inference comparer fires on any nonzero difference; the floor came from 2.0's geometry path. | **Keep the floor until phase 2**, then replace it with the interval rule in S5: ΔTc fires only when the two milestone ranges do not overlap (W1–W3). |
| D4 | Which local model drives the sidecar. | **The tool-capable model RunForge uses,** pinned per session and recorded in the session file. |

## Research grounding

Five research questions were run on 2026-10-06. Sources were retrieved by search and fetch. Most were read at abstract level, and the ones marked *fetched* were read in full. Where a design choice rests on a number, the number is quoted from the source. Velez 2021 and Kaltenecker could not be retrieved and are not cited.

Because these are statistical claims that a design depends on, rule 3 allows one targeted outside check. **Not run.** If Mike wants it, check the statistics block S1–S5 below by hand in Grok or Gemini and record the result here.

### Comparing two runs

- **C1. Variance sits at several levels (build, process, iteration). Results should be an effect size with a confidence interval, with repetition placed where the variance is.** Kalibera & Jones 2013, *Rigorous benchmarking in reasonable time*, ISMM, doi:10.1145/2464157.2464160 (fetched). → Within-run spread and run-to-run spread are reported separately. With one run per side, every verdict is labelled indicative.
- **C2. A performance change is best stated as a ratio with a confidence interval.** Kalibera & Jones 2020, *Quantifying performance changes with effect size confidence intervals*, arXiv:2007.10899. → The headline of an inference review is "B/A at p50 and p99, with interval", not a fired flag.
- **C3. Separate startup from steady-state performance.** Georges, Buytaert & Eeckhout 2007, *Statistically rigorous Java performance evaluation*, OOPSLA, doi:10.1145/1297027.1297033. → Steady-state statistics are computed after the warmup cut, and ΔTc reports the startup part.
- **C4. Use rank statistics and nonparametric confidence intervals. These still assume independent samples.** Hoefler & Belli 2015, *Scientific benchmarking of parallel computing systems*, SC, doi:10.1145/2807591.2807644. → Medians and quantiles with intervals. The interval is flagged when the samples are autocorrelated.
- **C5. Timing noise is non-normal, so robust statistics beat mean and standard deviation.** Chen & Revels 2016, *Robust benchmarking in noisy environments*, arXiv:1608.04295. → The spread band and the anomaly rule move from mean ± sd and 3-sigma to quantiles and MAD.
- **C6. Run-to-run variability ranges from 0.03% to over 100% CV, and interleaving A and B detects small slowdowns.** Laaber, Scheuner & Leitner 2019, *Software microbenchmarking in the cloud. How bad is it really?*, ASE. → A comparison accepts several runs per side.
- **C7. The sample count a percentile needs depends on the percentile. MLPerf uses a binomial bound at confidence 0.99.** MLCommons, *MLPerf Inference rules*, github.com/mlcommons/inference_policies (fetched). → Each pNN shows whether there are enough samples behind it.
- **C8. Coordinated omission: a closed-loop load generator drops the worst latencies.** Tene, *How NOT to measure latency* (talk; secondary summaries only). → A note when the timing source cannot rule it out. This is not a claim that it happened.

### Warmup and steady state

- **W1. Only about 43–57% of (VM, benchmark) pairs show classic warmup. Others are flat, slow down, or never settle.** Barrett, Bolz-Tereick, Killick, Mount & Tratt 2017, *Virtual machine warmup blows hot and cold*, OOPSLA, doi:10.1145/3133876. → Steady state is a tested outcome. Each run is classified as flat, warmup, slowdown or no steady state, and ΔTc is withheld unless both runs are flat or warmup.
- **W2. PELT finds changepoints exactly at linear cost. The penalty is the main setting.** Killick, Fearnhead & Eckley 2012, JASA, arXiv:1101.1438. → PELT on log-latency replaces the window/CV heuristic. Its penalty is fixed and recorded in the review and the bundle.
- **W3. Bayesian online changepoint detection gives a probability over where the change sits.** Adams & MacKay 2007, arXiv:0710.3742. → The milestone is an interval, not a single step (with PELT stability across penalties as the cheaper alternative).
- **W4. Static warmup settings are often wrong. Dynamic detection helps but still needs a sanity check.** Traini, Cortellessa, Di Pompeo & Tucci 2022, *Towards effective assessment of steady state performance in Java software*, Empirical Software Engineering. → The 2.0 heuristic stays as a cross-check. A disagreement lowers confidence and is not settled silently.
- **W5. MLPerf excludes warmup by protocol (untimed setup) instead of detecting it.** Reddi et al. 2020, *MLPerf Inference Benchmark*, arXiv:1911.02549. → "No warmup visible" is a valid run shape, not an error.

### Views

- **V1. Showing uncertainty about the mean makes readers overrate an effect, compared with showing outcome variability.** Hofman, Goldstein & Hullman 2020, CHI, doi:10.1145/3313831.3376454. → Keep the per-step spread visible next to any interval on the difference, and label which is which.
- **V2. Uncertainty displays are rarely evaluated on decisions, and often leave unclear what the band means.** Hullman et al. 2019, *In pursuit of error*, IEEE TVCG, doi:10.1109/TVCG.2018.2864889. → Every band and dot states what it is. The 3.0 caption already does this for the spread band.
- **V3. Quantile dotplots improve probability estimates.** Kay, Kola, Hullman & Munson 2016, *When (ish) is my bus?*, CHI. → A 20-dot quantile strip per run.
- **V4. Quantile dotplots and CDFs both improve decisions under risk.** Fernandes et al. 2018, CHI, doi:10.1145/3173574.3173718. → Keep the CDF and add a threshold the user drags, reading "P(latency > X): A 3%, B 1%".
- **V5. Bars with error bars lead to decisions that disagree with the statistics; gradient and violin plots do better.** Correll & Gleicher 2014, *Error bars considered harmful*, IEEE TVCG. → The band is shaded by quantile and not drawn as a hard edge.
- **V6. Comparison views are built from juxtaposition, superposition and explicit difference.** Gleicher et al. 2011, *Visual comparison for information visualization*, Information Visualization. → Add a difference plot (B−A per percentile, with interval) beside the overlay.
- **V7. Animation was the least accurate form for analysis, and small multiples were more accurate.** Robertson, Fernandez, Fisher, Lee & Stasko 2008, IEEE TVCG. → No animation in a view used to judge. Playback, when ported, is optional and sits beside a static view.
- **V8. Latency heat maps (time × latency) show patterns that summaries hide.** Gregg 2010, *Visualizing system latency*, ACM Queue, doi:10.1145/1794514.1809426. → A heat-map view per run on a shared scale.

### Knobs and evidence over time

- **K1. Performance-influence models give each option, and each option interaction, its own term.** Siegmund, Grebhahn, Apel & Kästner 2015, ESEC/FSE. → Main-effect hypotheses and interaction hypotheses are separate kinds.
- **K2. No learner or sampling design is best everywhere.** Grebhahn, Siegmund & Apel 2019, arXiv:1911.12643. → Each comparison records the design that produced it.
- **K3. Per-method profiling attributes performance to options.** Weber, Apel & Siegmund 2021, ICSE, arXiv:2102.06395. → When a profiler trace carries per-kernel times, a hypothesis can name the kernel.
- **K4. Change-point detection over CI history found regressions sooner, with fewer false alarms.** Daly et al. 2020, ICPE, arXiv:2003.00584. → A history view segments each measure over time and flags shifts that coincide with an environment change (driver, library).
- **K5. Hunter replaced a permutation test with a deterministic test so results repeat.** Fleming et al. 2023, ICPE, arXiv:2301.03034. → Seeds and tests are recorded with every checkpointed verdict.
- **K6. Always-valid inference survives continuous monitoring.** Johari, Pekelis & Walsh 2022, Operations Research, arXiv:1512.04922. **E-processes stay valid at any stopping time.** Ramdas, Grünwald, Vovk & Shafer 2023, arXiv:2210.01948. → The e-value evidence from RunForge carries over unchanged.

### The model in the loop

- **M1. The best open model trailed GPT-4 by about 19 points on closed-form data-analysis questions. An LLM judge agreed with experts 67% of the time.** Hu et al. 2024, *InfiAgent-DABench*, ICML, arXiv:2401.05507. → The program owns every verdict, and the model never judges.
- **M2. The best agent solved 34% of the analysis tasks.** Jing et al. 2025, *DSBench*, ICLR, arXiv:2409.07703. **The best system scored about 25% on real discovery tasks.** Majumder et al. 2025, *DiscoveryBench*, ICLR, arXiv:2407.01725. → The model proposes small, tool-checked steps, not conclusions.
- **M3. LLM hypotheses come in volume and hallucinate. Sequential falsification with e-values controls Type I error.** Huang et al. 2025, *POPPER*, ICML, arXiv:2502.09858. → Every proposed hypothesis is counted, and the e-BH correction runs over all of them. The pane shows "N tried, M hold".
- **M4. Agents doing root-cause analysis misread data and stop exploring early, across models; prompting did not fix it.** Kim, Park, Yun & Lee 2026, arXiv:2602.09937. → Numbers in the model's note are program-rendered, and the pane lists measures the model never looked at.
- **M5. Multi-turn tool calling breaks down in small models.** Patil et al. 2025, *BFCL*, ICML (multi-turn figures are from secondary summaries). → Calls are schema-checked, errors come back structured, and rounds and calls are capped. RunForge's loop already does all three.
- **M6. Explanations raised acceptance of AI advice whether or not the advice was right. Asking the person to commit first reduced over-reliance.** Bansal et al. 2021, CHI, arXiv:2006.14779; Buçinca, Malaya & Gajos 2021, CSCW, arXiv:2102.09692. → The program's verdict sits above the model's prose, which is labelled a proposal. An optional "your call first" box records the user's guess before the model runs.

## Parity map

Each row gives the 2.0 behaviour, the 3.0 status and the phase that closes it. "Oracle" names the 2.0 test or fixture that becomes the 3.0 acceptance test.

### Inputs

| 2.0 input | 3.0 today | Phase | Oracle |
|---|---|---|---|
| Stored RunTrace JSON (`schemaVersion`, `scalars.series`, `timeline`, `milestones.list`, fingerprints) | Refused | 1 | `tests/Fixtures/InferenceOptimization/*_tfrt_runtrace.json` with `expected_assertions*.json` |
| RunTrace validation (`RT_TIMELINE_*`, `RT_SCALAR_*`, `RT_MILESTONE_*`) that blocks the comparison | Missing | 1 | `expected_assertions_broken.json`: one `RT_TIMELINE_NON_MONOTONIC` at index 5, no deltas, no export |
| Fingerprint comparison (dataset, code and environment must match; model may differ) | Missing | 1 | `expected_assertions.json` fingerprint block |
| Profiler trace `.json.gz` | `.json` only | 1 | `TraceReviewTests.cs` |
| Memory, CPU and GPU series (`memory_bytes`, `memory_mb`, `peak_memory`, `gpu_memory_bytes`, `cpu_*`, `gpu_*`) | Latency and throughput only | 1 | fixture memory series |
| Runtime `.log` (`ParseRuntimeLog`) | Missing | 1 | new fixtures, because 2.0 has no `.log` test (audit F-ec12f8b8) |
| A folder of sources (`DetectSourcesAsync` priorities, `config.json` `WarmupSteps`) | Single files only | 1 | new fixture folder |
| ASPIRE geometry JSON (training dynamics) and geometry CSV | Refused | 4 (D1) | 2.0 demo pair and `GeometryRun` tests |
| backpropagate `run_history.json` | 3.0 only | done | `rust/tests/history_tests.rs` |

### Analysis

| 2.0 analysis | 3.0 today | Phase |
|---|---|---|
| ΔO, ΔF on inference | Same rule | done; re-based in phase 2 (S3) |
| ΔTc on inference (nonzero difference) | 3-step floor (D3) | 2 |
| ΔTd, ΔĀ suppressed for the TFRT preset | Same in effect | done |
| Confidence `min(1, aligned/100) × (both steady ? 1 : 0.7)` | Missing | replaced by intervals in phase 2 |
| Guardrails: warmup over 50% of the run, no steady state, only aggregated stats | Missing | 1 |
| TFRT preset shown to the user | Not shown | 1 |
| Alignment by wall clock | Missing (milestone alignment is the same) | 2 |
| Geometry deltas: failure presence, convergence, emergence ΔTd, alignment ΔĀ, stability ΔO, with their 2.0 defaults | Missing | 4 (D1) |

### Bundles and saved state

| 2.0 behaviour | 3.0 today | Phase |
|---|---|---|
| Phase 7.2 bundle: same files, hash and verify codes | Same, plus `review/review.json` | done |
| A 2.0 training or compare bundle opens | Findings text only, no chart (2.0 bundles hold no series) | 3: render the stored deltas as tiles with their explanations (see Views) |
| A 2.0 inference review (`review/review.json` only, no `integrity.json`) | Fails, and fails in 2.0 too (F-ed13f6e4) | 3: open it as unverified, show the review, and say there is no content check |
| A 3.0 bundle in 2.0 | Probably opens with the chart lost (not run) | 3: test it on the 2.0 build and record the result |
| `comparison-log.json`, recent files, the recent-file limit | Same | done |
| `preferences.json`: theme, reduce animations, screen-reader mode, large pointer, annotation density, export defaults, first run, dismissed hints | Read only for palette and text scale; other keys preserved | 3 |
| Gallery views | Titles listed, trace and history sources reopen | 3 (geometry views in 4) |
| Plugins | Not loaded in 2.0 or 3.0 | no change |

### Views and controls

| 2.0 view or control | 3.0 today | Phase |
|---|---|---|
| Overlay with band, marks, steady line, throughput, CDF | Same | done |
| Delta tiles, "Why did this fire?", "Show me" | Verdict sentence only; `review::Finding` exists but is not drawn | 3 |
| Welcome page, sample workflow, comparison log | Recent list only | 3 |
| Guide: delta glossary, search, shortcuts | Missing | 3 |
| Settings: theme, accessibility, export, about, privacy, report an issue | Missing; dark only | 3 |
| Keyboard shortcuts | Missing | 3 |
| Review-mode banner, load disabled while a bundle is open, copy hash | Partial | 3 |
| PNG export, side-by-side export, SVG export (four palettes) | Missing | 3 (PNG, SVG), 4 (frame sequences) |
| Trajectory canvas, eigen spectrum, scalar rings, failures timeline, annotations, insight feed, minimap | Missing | 4 (D1) |
| Synced playback (0.1–10×) | Missing | 4, optional beside static views (V7) |

## Beyond parity

### S. Statistics (phase 2)

- **S1. Ratio with interval as the headline.** B/A at p50, p90 and p99, each with a moving-block bootstrap interval (block length at least n^(1/3)). Labelled "within one run" when a side has one run (C1, C2, C4).
- **S2. Several runs per side.** Open a folder per side. The bootstrap resamples runs first, then steps within each run, and a verdict needs at least three runs per side to drop "indicative" (C1, C6).
- **S3. Robust spread and anomalies.** The band becomes p10–p90 of a centred window, shaded by quantile. Anomalies are points beyond the median plus 5 MAD, and ΔF counts them. The 2.0 3-sigma rule stays available as a setting that bundles record (C5, V5).
- **S4. Sample sufficiency.** Each pNN shows the binomial order-statistic interval. A percentile without enough samples behind it is shown as "not enough samples" and is not printed as if exact (C7).
- **S5. Run shape and an interval ΔTc.** PELT on log-latency, with the penalty recorded. Each run gets a shape label: flat, warmup, slowdown or no steady state. The milestone is the range of change points across three penalties. ΔTc fires only when both runs are flat or warmup and the two milestone ranges do not overlap, and the 2.0 heuristic is a cross-check (W1–W4, D3).
- **S6. Notes, not verdicts,** for coordinated omission (C8) and for a run with no visible warmup (W5).

Every one of these settings is written into the bundle's `repro.json`, so a stored review says how it was made.

### V. Views (phases 2–3)

- **Difference plot.** B−A by percentile from p50 to p99.9, with its interval and a zero line, beside the overlay (V6, V1).
- **Percentile spectrum.** Both runs on a log-tail axis (C8, V4).
- **CDF threshold.** Drag a threshold to read "P(latency > X)" for each side (V4).
- **Quantile strip.** 20 dots per run (V3).
- **Heat map.** Step × latency per run, on a shared colour scale (V8).
- **Segments.** The PELT segments and the shape label drawn on the series (W1).
- **Delta tiles.** Each tile shows what fired, why, the interval, and "show me", which moves the plot to the anchor.

### H. History (phase 5)

- A history of reviews per project, with change points per measure over time (K4). An environment change on a run (driver, library, GPU) is shown beside any shift that coincides with it.

## The workbench sidecar (phase 5)

It works like RunForge's (`docs/sidecar-workbench.md` in runforge), on the shared crate (D2).

- **Runs and knobs.** A run carries its knobs: batch size, precision, TensorRT on or off, threads, CUDA graphs, input shape. They come from the RunTrace metadata, a sidecar `knobs.json` beside a run, or the folder name. A comparison counts as evidence for a knob only when that knob differs and the recorded others match (K1, K2).
- **Measures.** The built-in set is p50, p90, p99, p99/p50, steady mean, warmup cost (ms × steps), anomaly rate, throughput at p50 and memory peak. The model builds more in the formula language. Kernel-level measures are available when a profiler trace has them (K3).
- **Hypotheses.** Main effects ("when batch goes up, p99/p50 goes up") and interactions ("fp16 × TensorRT") are separate kinds. Each test is fixed when it is proposed. Evidence is a permutation e-value per comparison, multiplied across comparisons, with checkpointed e-BH verdicts over every hypothesis ever proposed (K6, M3).
- **The envelope.** The model calls tools and proposes. The program computes every number and verdict and renders the numbers in the note. The pane shows "N tried, M hold", measures not yet looked at, the pinned model and the session record (M1–M5).
- **The person first.** An optional box records the user's own call before Ask. The verdict and its interval sit above the model's note, which is labelled a proposal (M6).
- **Memory.** The workbench file holds tools, hypotheses, checkpoints and notes. It holds no paths and stays local.

### Phase 5 plan (2026-10-06)

The section above says what phase 5 does, but not how the code moves. This plan fills that gap. It was written after reading RunForge at `c0c5042`: `crates/runforge-core/src/{expr,bench,session}.rs` and `crates/runforge/src/sidecar.rs`.

**What is shared today, and what is not.** RunForge's workbench is generic in its method but not in its types. `expr` evaluates a formula against a `Series`, and its measures (`low`, `median_between`, `slope_between`) are loss-over-epoch readings. `bench` and `session` take a `Board`, and they read `Series.recipe`, `Series.seed` and the loss samples (for `run_fingerprint`). The statistics (`exact_p`, `permutation_e`, `lambda_for`, Holm, e-BH, `Book` checkpoints), the wording fence, the phases, the caps and the Ollama loop never touch a loss value. They are what moves.

**5a. The shared crate, by pull request to runforge** (opened as mcp-tool-shop-org/runforge#7: 149 RunForge tests before the move, 154 after, all passing).

- It lives at `crates/workbench` in the runforge workspace (crate name `workbench`, `publish = false`). ScalarScope depends on it with `git = "https://github.com/mcp-tool-shop-org/runforge"`, pinned to a `rev`. A pin moves by pull request, as the Atlas pin does. It does not get a repo of its own: D2 sends the change to runforge, and one home keeps one history for the evidence code.
- **The interface** is a concrete `Run` plus one trait the host implements:
  - `Run { name, seed: Option<i64>, knobs: Map<String, Value>, identity: String }`. `identity` is computed by the host. RunForge keeps "seed + hash of the first 32 samples". ScalarScope uses seed + a hash of the first 32 latency samples.
  - `trait Measures { fn catalogue(&self) -> &[Measure]; fn measure(&self, run: usize, name: &str, args: &[Arg]) -> Result<f64, String>; fn format(&self, name: &str, value: f64) -> String; fn method(&self) -> String; }`. `method` is the board key that hypotheses are filed under. RunForge's is the training method. ScalarScope's is the TFRT preset or the framework.
  - The crate keeps `knob('name')` and the math functions (`abs`, `sqrt`, `ln`, `exp`, `min`, `max`). It adds them to every host's catalogue, so a host cannot shadow them.
- **What moves:** the formula grammar, its caps and canonical form (`expr`, without `RunView`); everything in `bench` except `run_fingerprint`; `session`; and from `sidecar.rs` the Ollama loop, `choose_tool_model`, `candidates`, `can_call_tools` and the HTTP code. RunForge keeps `LossMeasures` (today's `RunView`), its `Board`, its pane and its file locations.
- **D4, pinned and recorded.** RunForge's sidecar chooses a model by preference order and does not save which one it used (its own doc defers this). The shared loop records the model name and its Ollama digest in the session record, so both apps reach PIN_PER_STEP 3. The preference order stays RunForge's. On this rig `qwen3:14b` is not installed (2026-10-06), so the loop falls to `qwen2.5:14b`, the second in that order.
- **Tests carry over.** `tests/evidence.rs` and the unit tests in `expr`, `bench`, `session` and `sidecar.rs` (including the fake-Ollama end-to-end test) move into the crate and run against a test `Measures`. RunForge's full suite stays green, with the same test count across both crates. This is the first half of the exit test.

**5b. The workbench in ScalarScope.**

- **Knobs.** They are read in this order, and the first source that has a knob wins: a `knobs` object in the RunTrace `metadata`; a `knobs.json` beside the run; then `key=value` pairs in the run's folder name, split on `_` or `,` (`batch=8_precision=fp16`). The pane shows where each knob came from. The 2.0 fixtures carry no knobs, so on the golden pair every hypothesis is "not testable". That is a program-set state, and it is the honest answer for those files.
- **Measures.** The catalogue is the built-in set the spec names: `p50`, `p90`, `p99`, `p99_over_p50`, `steady_mean`, `warmup_cost` (ms × warmup steps, from the S5 shape), `anomaly_rate` (under the rule the review uses), `throughput_p50` and `memory_peak`. It adds `quantile_between(a, b, q)` and `mean_between(a, b)` over steps, so the model can build windows. A measure that a run cannot supply (no memory series, no milestone) is a refusal with a reason, never a zero. Kernel measures (K3) wait for a profiler-trace fixture that carries per-kernel times. Until then they are listed as deferred, not offered.
- **The pane.** It is a "Workbench" tab beside the comparison. It shows the program's verdicts and intervals first. Below them, an optional "Your call" box is saved before Ask (M6). Then the model's note, labelled "the model's words, not a measurement", then "N tried, M hold", the measures not yet used (M4), the pinned model and digest, and the step log. Ask is disabled while a bundle is open in review mode.
- **Memory.** `workbench.json` sits in the preferences directory (the 2.0 `LocalState` path when packaged). It holds tools, hypotheses, the checkpoint book and notes. It holds no paths: a run is named by its identity hash. Its compensators are RunForge's: delete an entry, or delete the file.
- **Exit fixture.** A knob-varying folder is added as `rust/tests/fixtures/workbench/`, beside the aspire-si fixtures, so the .NET fixture tests do not walk it: batch 1, 4 and 8, three seeds each, as nine RunTrace files with `metadata.knobs`. These are **simulated**, with latency drawn from a stated model in a committed generator script, and the folder's README says so. Real inference timings would be better. They need GPU time and a model to serve, so they are a director's call (question P2).

**5c. History (H, K4).**

- A **project** is the pair (dataset fingerprint, model fingerprint) of side B. Code and environment are left out of the key, because their changes are what the history is for. A 2.0 log entry without fingerprints goes to "Unsorted".
- Each comparison-log entry gains an optional `measures` object: side B's built-in measures, as the review computed them. 2.0 entries lack it, and they are kept and shown without a series.
- For each measure, the project's entries in date order go through PELT with a fixed penalty, which is recorded on the view. With fewer than six entries the view says "too few reviews for change points" and draws only the points.
  - *As built:* `shape::pelt`'s mean-and-variance cost is unstable on segments of a few reviews. Three near-equal values have almost no variance, and their cost runs to minus infinity. History therefore uses `trends::mean_shift_pelt` instead:
    - the squared error around each segment's mean, on the logarithm;
    - one noise scale for the whole series, 1.4826 × MAD of first differences / √2, so the shift itself does not inflate it;
    - a penalty of 2 · ln n (BIC: a change adds a location and a level);
    - segments of at least three reviews.
- An entry whose code or environment fingerprint differs from the one before it is marked on the axis. A shift that falls within one entry of such a mark names the mark: "p99 shifted at 2026-10-09, with an environment change". The program writes that sentence, and it claims coincidence, not cause.

**Exit test, restated.** (1) RunForge's workbench tests pass on the shared crate, with RunForge's suite green and its count unchanged. (2) A live session with the local tool model on the golden pair and on `rust/tests/fixtures/workbench/` ends with every hypothesis in a program-set state. The session record, with no paths, is committed under `docs/receipts/` as the receipt. (3) The history view draws a project of at least six logged reviews, with a fixture that holds one environment change and one shift.

**Order of pull requests.** 5a (runforge), then 5b (ScalarScope, pinned to 5a's merge commit), then 5c. Each is merged by the director. A live session loads a local model on the GPU, so it is announced before it starts.

**Questions for the director.**

| # | Question | Recommendation |
|---|---|---|
| P1 | Where the shared crate lives | `crates/workbench` in runforge, pinned by git `rev` (above). The alternative is a new repo, which would also need its own Atlas map, CI and shipcheck. |
| P2 | Simulated or real knob runs for the exit fixture | Simulated now, so the exit test is runnable. Real runs later, if they are worth the GPU time. |
| P3 | Project key for history | (dataset, model) fingerprints of side B, as above |

## Phases

| Phase | Content | Exit test |
|---|---|---|
| 1. Inputs | RunTrace read and validate, fingerprints, `.gz`, memory/CPU/GPU, `.log`, folders, guardrails, TFRT preset shown | The three 2.0 oracles pass in Rust: the golden pair, nearly identical and broken |
| 2. Statistics | S1–S6, the interval ΔTc, wall-clock alignment, the difference/spectrum/threshold/strip/heat-map/segment views | Tests for each statistic against hand-computed values. The golden pair keeps its 2.0 verdicts, or the spec records why one changes |
| 3. Product shell | Delta tiles and Why, Welcome with a sample, Guide, Settings with Light/Dark/System and accessibility, shortcuts, review mode, PNG/SVG export, bundle compatibility in both directions | A 2.0 user's files, bundles and preferences open. The Store QA items in `STORE_LISTING.md` pass |
| 4a. Geometry exporter (D1) | In `aspire-si` (the restored `aspire-ai`), record per step the student's pooled hidden state and each teacher dimension's score, then write the 2.0 `GeometryRun` export: a 2-D projection fitted over the run, velocity, curvature, effective dimension, eigen spectrum, evaluators and failures | A short training run writes an export that ScalarScope 2.0 opens. Its fields match the two 2.0 samples (`Resources/Raw/Samples/*_professors.json`) |
| 4b. Geometry views (D1) | Geometry import and deltas with 2.0 defaults, static trajectory/eigen/evaluator/failure views, optional playback, frame export | The 2.0 geometry tests pass in Rust. The demo pair and an export from 4a render |
| 5. Workbench and history | The shared crate (D2), knobs, measures, hypotheses, evidence, envelope, history change points | The RunForge workbench tests pass on the shared crate. A live session on the golden folders ends with program-set states |

Store submission is held by the director (2026-10-06) until phase 5 lands and a geometry export from a real aspire-si training run has been opened in ScalarScope and its comparison checked by a person. This replaces "follows phase 4b". WACK, the listing, screenshots, samples for testers and certification notes come then, as they did for RunForge.

## Verdict changes from 2.0

Phase 2 changes how four rules answer. Each change is deliberate and has a test.

| Rule | 2.0 | 3.0 | Why |
|---|---|---|---|
| ΔF | 3-sigma outliers over the whole series; fires when B has more | Samples beyond 5 robust deviations (1.4826 × MAD) from the median, steady samples only; fires when B's excess is beyond chance (one-sided exact conditional binomial test, p < 0.05) | A warmup sample is startup cost, not a runtime anomaly (C3); the standard deviation is inflated by the very spikes it is meant to find (C5); one extra spike is not evidence |
| ΔO | Population standard deviation from the steady step, in ms; fires on a 1% difference | Relative spread of the steady samples, (p90 − p10) / p50; fires when the 95% block-bootstrap interval on B's over A's excludes 1 | Spikes are ΔF's to report; a 1% threshold fires on noise (C1, C5); in ms, a faster run with the same proportional jitter would be called steadier |
| ΔTc | Any nonzero difference in steady step (inference comparer) | Steps stated in RunTrace files keep the 2.0 rule. Otherwise each run's shape decides (flat, warmup, slowdown, no steady state); ΔTc fires only when both settle and their steady-start ranges do not overlap | Many runs never settle (W1); a segment boundary is not a measurement (W2, W3) |
| Ratios, ΔO, ΔF | — | Need 20 steady samples per side | A bootstrap of a handful of samples returns the same handful, so its interval is a point |

The band became the p10–p90 of an 11-sample centred window and the marks became the 5-MAD rule (V5, C5). The 2.0 window heuristic stays as a cross-check: when it puts steady state outside the detected range, the page says so (W4).

**The golden pair.** 2.0's `expected_assertions.json` says ΔF fires, because the optimized run's first sample (40 ms) is a 3-sigma outlier of the whole series. That sample is warmup. Under the 3.0 rule ΔF is quiet. `rust/tests/runtrace_tests.rs` asserts the 2.0 oracle still says "fired" and that 3.0 does not. Every other golden assertion holds.

**Phase 3.** S3 keeps the 2.0 3-sigma rule as a setting that bundles record. It is on the Settings page (`AnomalyRule` in `preferences.json`), and the caption names the rule a review used.

## Geometry: open questions from the port

Phase 4b ports 2.0's five geometry deltas exactly, including behaviour that looks wrong. Each is
marked `C# BUG?` in `rust/src/geometry_deltas.rs`, and each keeps 2.0's answer until it is
decided. The tests pin 2.0's results, so a change here is deliberate and shows up as a failing
oracle.

| # | Rule | 2.0 does | Question |
|---|---|---|---|
| G1 | ByConvergence alignment | Pins its own constants instead of reading `ConvergenceConfig` | Should alignment follow the config ΔTc uses? |
| G2 | First-instability anchor | Thresholds the mean of *signed* curvature; the stability rule uses absolute curvature | Use absolute curvature in both? |
| G3 | ΔF step | Multiplies normalised time by the step count, not count − 1 | A last-step failure reports one step past the end |
| G4 | ΔTc sign | The delta is +1 whichever run converged | Sign it by run? |
| G5 | ΔĀ name | "Evaluator agreement" is λ1/Σλ, how concentrated the spectrum is | Rename, or measure agreement? |
| G6 | ΔĀ direction | Which path is "higher" comes from the whole-run mean; the size comes from the longest segment | The two can disagree |
| G7 | ΔO anchor on a tie | The anchor goes to the left run, the words to Path B | Only reachable with DeltaFloor ≤ 0 |
| G8 | ΔF persistence | Three recorded failures, hard-coded; time from the third, kind from the first | Read `PersistenceWindow`; one failure's time and kind |
| G9 | ΔĀ missing steps | A step without enough eigenvalues counts as alignment 0 | Leave such steps out? |
| G10 | ΔO threshold | Adaptive per run: max(1.5 × median, 1 × robust sigma) of that run's own absolute curvature | A very straight run gets a tiny threshold and is called unstable for one small curl. Seen on the aspire-si demo pair: the steady run is flagged, the regressing one is not. A threshold shared by both runs would compare like with like |

### Decision brief (2026-10-06)

Each fix was applied by itself on a scratch copy and run on the two pairs ScalarScope has. The pairs are the 2.0 demo pair (correlated against orthogonal professors) and the simulated aspire-si pair (steady against regressing). Each was run in both orders and under step, convergence and first-instability alignment. Nothing was changed in the code. A bundle stores its deltas, so saved reviews keep the answers they were saved with. A fix changes only comparisons computed after it.

| # | What the fix changes on this data | Tests it breaks | Recommendation |
|---|---|---|---|
| G1 | Nothing. The defaults equal the pinned constants, so only a non-default convergence config would differ. | None | **Fix.** It is invisible today and removes a trap for anyone who tunes the config. |
| G2 | Nothing, in all three alignments. | None | **Fix, fully.** Take absolute curvature for the mean *and* for the comparison. The measured patch changed only the mean. |
| G3 | Nothing. Both pairs' failure steps land on the same integer either way. On the aspire pair that depends on how floating point rounds 0.5763 × 59, so it is fragile. | None | **Fix.** It removes the one-past-the-end step. |
| G4 | Nothing. The branch is never reached: neither demo run converges, and both aspire runs do. | `convergence_when_only_one_run_settles_is_present` (asserts +1) | **Fix, using the sign convention of the both-converged branch** (right minus left): +1 when only the left run converged, −1 when only the right did. Update the one test. |
| G5 | Only wording: the tile name "Agreement", the explanation, "Similar evaluator alignment", the summary tail " while evaluator alignment differed" (the aspire verdict ends with it) and the Guide's help line. | Four string assertions in `geometry_delta_tests.rs` | **Rename** to "Spectrum concentration". Keep the id and the symbol, so 2.0 bundles still match. |
| G6 | No direction flips and no status changes. On the aspire pair the reported change grows from ±0.142 to ±0.301 under step alignment. After the fix the change no longer equals right minus left of the means shown beside it. | None | **Keep 2.0.** The two disagree on neither pair, and the fix makes the tile's numbers inconsistent with each other. Revisit if a real export shows a disagreement. |
| G7 | Unreachable at the default floor: a tie is below the 0.05 floor and is suppressed before the anchor code runs. | n/a | **Leave it, and close the question.** |
| G8 | The failure *kind* in ΔF changes on both pairs. The time is unchanged, because the third failure was already the time source. Demo: "correctness_dip near step 6" becomes "tradeoffs_failure near step 6". Aspire: "practicality_dip" becomes "correctness_dip", which is tied at that time with two others. | Two oracle explanation tests, and the compat bundle verdict | **Fix.** Today's sentence names one failure's kind at another failure's time, which is wrong as written. The 2.0 oracles would then record 2.0's sentence and 3.0's side by side, as the golden pair already does for ΔF. |
| G9 | Nothing. No step in these files is short of eigenvalues. | None, which means no test covers it | **Fix, and add a test with a sparse step.** |
| G10 | A shared threshold over both runs' pooled curvature turns the demo pair's ΔO from present to "Both runs stable". On the aspire pair the steady run is **still** flagged, with a lower score (0.517 to 0.403 under step alignment), and under first-instability alignment the regressing run becomes flagged. | Two oracle tests, and the compat bundle (3 fired tiles, not 4) | **Do not adopt the pooled threshold.** It does not cure the case that raised the question. Decide G10 on the two real aspire-si exports (phase 4a's exit test), where it can be judged on runs that were actually trained. |

In short: G1, G2, G3, G4 and G9 are safe fixes that are invisible on today's data. G5 and G8 change words a user sees. G6 and G7 stay as they are. G10 waits for real exports.

**Rulings (2026-10-06).** The Publisher session, which coordinates these repos at the director's direction, ruled as recommended, and the director may override any of these:
- G1, G2 (absolute curvature for both the mean and the comparison), G3, G4 (the right-minus-left sign), G8 and G9 (with a sparse-step test) are fixed.
- G5 is renamed "Spectrum concentration", keeping its id and symbol.
- G6 and G7 stay as 2.0 has them, and G7 is closed.
- G10 waits for the real exports.
- **G10, decided 2026-10-07 on the real aspire-si exports:** keep 2.0's per-run threshold, and G10 is closed. On both real pairs the runs' own thresholds are nearly equal, and a shared threshold changes no verdict or owner. The evidence is in `docs/receipts/geometry/README.md`.

The fixes are applied in `rust/src/geometry_deltas.rs`, each marked `G<n> (ruled 2026-10-06)`. The oracles that change keep 2.0's wording in a comment.

One rounding choice for G3: the step is rounded to the nearest one rather than truncated. A time stored as i / (n − 1) can come back as i − 0.000…1, as it nearly did on the aspire-si pair.

## Geometry export contract: step axis and score source (2026-10-07)

On the real aspire-si exports, three of the five deltas fired on how the data was laid out, not on what changed (`docs/receipts/geometry/README.md`). The page's headline was built from them, which is a trust problem for a Store user, so this contract is required before the Store submission.

**The fields.** They were agreed with aspire-si. All three are optional and live in `run_metadata`. Their schema is version 1.1.
- `step_axis`:
  - `"training_step"`: steps are training time, in order.
  - `"checkpoint_by_item"`: steps are blocks, one per checkpoint, over a fixed item order, so they are not time within a block. This is what drift and probe exports use.
- `checkpoints`: an integer, given with `checkpoint_by_item`, so the page can draw block boundaries.
- `scalar_source`:
  - `"live"`: every step is scored fresh.
  - `"replayed"`: epochs after the first replay cached scores.
  - `"fixed_per_item"`: each item's scores repeat in every block.

**Reading the fields.**
- In a 1.1 file, an absent field means `training_step` or `live`.
- A 1.0 file states neither. It is read exactly as before, with every 2.0 rule and every 2.0 oracle unchanged.
- An unknown value is a warning, and the file is read as if the field were absent.

**What ScalarScope withholds.** A withheld delta has the status 2.0 declared and never produced, `Indeterminate`, shown as "withheld". Its explanation states the reason.
- **Either run has `checkpoint_by_item`:**
  - ΔTc and ΔO are withheld: "steps are checkpoint × item, not time".
  - ΔTd is withheld with the same reason (ruled 2026-10-07): when a dominant direction emerges depends on step order, as ΔTc's settle point does. The principle is that any reading that depends on step order is withheld when steps are not time.
  - ΔF is withheld too, because its divergence and collapse checks read across steps.
  - The trajectory is drawn as unordered points per checkpoint block, not as one joined line, and is labelled so.
- **Either run's `scalar_source` is `replayed` or `fixed_per_item`:**
  - ΔF is withheld: its recorded failures are score dips, and the scores repeat.
  - The score panels say the scores are replayed or fixed per item.
- **ΔĀ on checkpoint × item** (ruled 2026-10-07). 2.0's rule keeps the longest stretch of consecutive steps, which depends on step order.
  - On these exports, each run's λ1/Σλ is averaged over each checkpoint block's items, which is order-free within a block.
  - Consistency across blocks replaces the sustained-stretch filter: ΔĀ fires only if B − A clears the delta floor (0.05) with the same sign in every block. The wording is "Path B had a more concentrated spectrum at all N checkpoints", with no "sustained N steps".
  - Otherwise it is quiet, with the reason: "differs by checkpoint", or the checkpoints where the difference is below the floor.
  - It is withheld when the blocks cannot pair: a different number of checkpoints, or one run whose steps are time.
  - Time-ordered exports keep 2.0's rule unchanged.
- **The headline** is built only from deltas that stand: neither quiet nor withheld.

**Also fixed with this:** evaluator labels that overlap when two evaluators point the same way.

## Corrections

- The 3.0.0 changelog says ΔTc's 3-step floor is "the same resolution the .NET app used". That is true of 2.0's geometry convergence delta (`ResolutionSteps = 3`), not its inference comparer, which fires on any nonzero difference. The line is corrected in this change.
