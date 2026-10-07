# Store listing screenshots (3.1.1)

Six screenshots at 1920 × 1080, taken from the release build on the files in `samples/` and the built-in sample: text scale 1.3, except shot 5 at 1.0 so a whole session fits. No path or personal data is on screen. Shots 4 and 6 were retaken for 3.1.1: the drift pair now withholds ΔTd and reads ΔĀ as a difference between the evaluator setups, and the Guide has a geometry section. Shots 1–3 and 5 are unchanged from 3.1.0, and 3.1.1 draws those pages the same way.

| Order | File | Caption |
|---|---|---|
| 1 | `1-compare-headline.png` | B/A at p50 and p90 with 95% intervals, and a tile for each delta that says whether it fired, and why. |
| 2 | `2-distribution-why.png` | Every delta explains itself: the rule and this pair's numbers. Drag a threshold to read P(latency > x) for each run. |
| 3 | `3-heat-map.png` | Six views, none animated. The heat map shows where each run's samples fall, step by step, on a shared scale. |
| 4 | `4-geometry-drift.png` | ASPIRE training geometry. A reading the data cannot support is withheld, with the reason. |
| 5 | `5-workbench.png` | Workbench: a local model proposes what a setting does. ScalarScope tests each hypothesis and sets its state, and the model's note is labelled as its words. |
| 6 | `6-guide.png` | A searchable guide to every rule on the page. |

## Sources

| Shot | What is open |
|---|---|
| 1–3 | The built-in sample comparison (`rust/samples/baseline.csv`, `rust/samples/optimized.csv`): Series view, then Distribution with the ΔO Why panel open, then Heat map |
| 4 | `samples/geometry/local-teacher.drift.geometry.json` against `composite-teacher.drift.geometry.json`, from two real aspire-si training runs |
| 5 | The nine simulated runs in `samples/workbench` (the page says they are filed under "simulated"), after a real session with qwen3:14b on the local GPU (2026-10-07). The program recorded two hypotheses: tail heaviness up with batch size, inconclusive; throughput up with batch size, passes on these runs alone. The model's note calls the second "supported", which is why the note sits under the label saying it is the model's words. |
| 6 | Guide |

## Retaking them

The release build has a capture mode that is off unless `SCALARSCOPE_CAPTURE` is set. With it set, the window opens at 1920 × 1080, sets the page, view and Why panel, writes a PNG of itself, and closes. From the repo root:

```bash
SCALARSCOPE_CAPTURE=docs/store/1-compare-headline.png SCALARSCOPE_CAPTURE_SCALE=1.3 \
SCALARSCOPE_CAPTURE_PAGE=compare SCALARSCOPE_CAPTURE_VIEW=1 \
  rust/target/release/scalarscope.exe rust/samples/baseline.csv rust/samples/optimized.csv
```

Shot 4 opens the two drift files in `samples/geometry` instead, and shot 6 sets `SCALARSCOPE_CAPTURE_PAGE=guide` with no files. Shot 5 adds `SCALARSCOPE_CAPTURE_RUNS=samples/workbench` and `SCALARSCOPE_CAPTURE_ASK="<your call>"`, which runs a real session against a local Ollama model that can call tools and waits for it. Check the GPU is free first, and unload the model after.

The other variables are `SCALARSCOPE_CAPTURE_WHY` (a delta symbol such as `ΔO`) and `SCALARSCOPE_CAPTURE_RUNS` (a folder of runs for the Workbench). `SCALARSCOPE_CAPTURE_PAGE` takes `welcome`, `compare`, `workbench`, `history`, `guide` or `settings`.
