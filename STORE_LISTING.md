# ScalarScope Microsoft Store Listing

**Store ID:** 9P3HT1PHBKQK  
**Package name:** mcp-tool-shop.ScalarScope  
**Publisher:** CN=5305D976-6952-4F00-9C21-3A5DB090359F  
**Publisher display name:** mcp-tool-shop  
**Package Family Name:** mcp-tool-shop.ScalarScope_yn6b8xqrexa5j  
**Package SID:** S-1-15-2-850189134-3642041993-2632034504-2327210781-2703906947-3194312410-3859322172  
**Display version:** 3.1.0 (the crate version. Settings > About shows "ScalarScope 3.1.0 (package 3.1.0.0 …)".)
**ApplicationVersion:** 31 (package integer, not the About string)
**Package identity version:** 3.1.0.0 (`packaging/AppxManifest.xml`, which `pack.ps1` reads, and the MAUI `Package.appxmanifest`, held equal by a test)

Packages already in the Partner Center submission:

- `ScalarScope_1.0.3.0_x64.msix`
- `ScalarScope_v2.0.0_Store.msixupload`

The next upload is `ScalarScope_3.1.0.0_Store.msixupload`. Its package version is `3.1.0.0`, which is above both of those. 3.0.0.0 was never uploaded.

---

## Short Description (200 characters max)

Compare two ML runs and see how sure the comparison is: inference latency with intervals, training loss, ASPIRE geometry, and a local workbench. Offline; nothing leaves your PC.

---

## Full Description

ScalarScope compares two machine-learning runs and says how run B differs from run A, with an interval or a stated reason for each difference. It holds back what the data cannot support.

**Inference runs.** Open two latency traces: a CSV, a benchmark JSON, a Chrome or PyTorch profiler trace, a runtime log, or a RunTrace.
- **The headline** is B/A at p50, p90 and p99, each with a 95% bootstrap interval. A percentile without enough samples is named, not printed.
- **Three deltas,** for new anomalies, settle point and variability, each say whether they fired, stayed quiet or were withheld, and why.
- **Six views:** series, warmup, distribution, difference by percentile, tail spectrum and heat map.

**Training and geometry.**
- **Training:** a backpropagate run history is drawn as training loss.
- **Geometry:** two ASPIRE training runs side by side, with their trajectory, evaluator scores and eigen spectrum. When an export says its steps are not time, or its scores were replayed, the readings that depend on them are withheld with that reason.

**Workbench.** Open runs that differ in a setting, such as batch size or precision. Ask a local model, through Ollama on your own computer, to measure them and propose what the setting does. ScalarScope tests every hypothesis and sets every verdict. The model's note is labelled as its words.

**History.** Finished comparisons are grouped by project, and each measure is drawn across them with the points where it shifted.

**Reviews you can reopen.** Save a review as a bundle and reopen it exactly as saved. A matching SHA-256 is a content check, not a signature. Bundles, settings and recent files from ScalarScope 2.0 carry over.

**Private by design.** No account, no telemetry and no analytics. The only connection ScalarScope can make is the Workbench's Ask, to a local Ollama at 127.0.0.1. Cloud models are refused, so nothing leaves your machine.

---

## Feature Bullets (5)

1. Compare two ML runs: B/A at p50, p90 and p99 with intervals
2. Deltas that say whether they fired, stayed quiet or were withheld, and why
3. ASPIRE training geometry, with readings withheld when the data cannot carry them
4. A local workbench: a model on your PC proposes, ScalarScope tests and decides
5. Fully offline: no telemetry, no account, no cloud

---

## Screenshots

The listing screenshots are in `docs/store/`, taken from the release build on the files in `samples/`. Their order and captions are in `docs/store/README.md`.

---

## QA Runbook (Final Pre-Release)

The quick checks a certification tester can run are in `TESTING.md`, on the files in `samples/`.

### Install Flow
1. [ ] Download from Microsoft Store (9P3HT1PHBKQK)
2. [ ] Launch ScalarScope. Welcome appears with Try the sample comparison, Compare two runs and Open a review bundle.
3. [ ] Settings > About shows ScalarScope 3.1.0, package 3.1.0.0.

### Core Workflow
4. [ ] Try the sample comparison: Compare shows the headline, three tiles and the series.
5. [ ] Open path A and B with `samples/inference/baseline.runtrace.json` and `optimized.runtrace.json`.
6. [ ] Verify deltas. These files state their steady-state milestone, so ΔTc reads it: it fires, 6 steps earlier. A missing milestone must not appear as a stabilization time.
7. [ ] Click a delta tile: the Why panel opens. Click Show me: the view moves to the anchor.
8. [ ] Open the geometry pair in `samples/geometry` (`.drift` files): ΔF, ΔTc and ΔO show withheld, with the reason.

### Export and Review Flow
9. [ ] Save bundle writes a `.scbundle`. Copy hash copies its SHA-256.
10. [ ] Open bundle on that file: review mode banner, load buttons disabled, same hash.
11. [ ] Export SVG and Export PNG write files where chosen.

### Workbench, History and Settings
12. [ ] Workbench: Open runs… with the nine files in `samples/workbench`. Batch size is listed as varying, and Try a formula `p50` gives a value per run.
13. [ ] Ask without Ollama running says the local model is not running.
14. [ ] History lists the reviews logged in steps 4–6.
15. [ ] Settings: theme change applies at once; Report an issue opens GitHub.

### Uninstall
16. [ ] Uninstall from Windows Settings
17. [ ] Verify clean removal (no leftover files)

---

## Categories

- **Primary:** Developer tools
- **Secondary:** Productivity

## Age Rating

- **ESRB:** Everyone
- **PEGI:** 3

## System Requirements

- **OS:** Windows 10 version 1809 (build 17763) or later. That is the package `MinVersion` and `TargetPlatformMinVersion` (`10.0.17763.0`).
- **Architecture:** x64
- **RAM:** 4 GB minimum
- **Disk:** 100 MB

---

## Release Notes (v3.1.0.0)

ScalarScope 3.1 is a rewrite. It is the same product, name and publisher, and your 2.0 files, bundles and settings carry over.

- **Inference comparisons say how sure they are.** B/A at p50, p90 and p99 comes with an interval, percentiles without enough samples are not printed, and several runs per side measure run-to-run variation.
- **Deltas give their reasons.** Each tile says fired, quiet or withheld, with the rule and this pair's numbers. Show me moves to the evidence.
- **Six views:** series, warmup, distribution with a threshold, difference by percentile, tail spectrum and heat map.
- **ASPIRE training geometry,** with readings withheld when an export says its steps are not time or its scores were replayed.
- **A local workbench and a history** of your comparisons.
- **Light theme, SVG and PNG export,** and keyboard shortcuts.

The full list is in CHANGELOG.md.

## Release Notes (v2.0.0)

### What's New

**Phase H: Humanizing & UI/UX Finish**
- New Welcome page with streamlined first-60-seconds experience
- Unified design system (typography, spacing, colors)
- Comprehensive About section with privacy statement
- Bundle hash explanation for non-technical users

**Inference Optimization Focus**
- TFRT Runtime Preset with automatic delta suppression
- RunTrace comparison with milestone alignment
- Scientific rigor: a spread band, guardrails, reproducibility. The band is not a confidence interval.

**Quality of Life**
- Recent comparisons on Welcome page
- Designed empty states throughout
- Reduced motion mode respects system preferences
- No telemetry pledge clearly stated

---

## Contact

- **Support:** https://github.com/mcp-tool-shop-org/scalarscope/issues
- **Documentation:** https://github.com/mcp-tool-shop-org/scalarscope/blob/main/README.md
- **Privacy Policy:** Local-only app, no data collection
