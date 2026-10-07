# Store listing screenshots (3.1.0)

Six screenshots at 1920 × 1080, taken from the release build at text scale 1.3, on the files in `samples/` and the built-in sample. No path or personal data is on screen.

| Order | File | Caption |
|---|---|---|
| 1 | `1-compare-headline.png` | B/A at p50 and p90 with 95% intervals, and a tile for each delta that says whether it fired, and why. |
| 2 | `2-distribution-why.png` | Every delta explains itself: the rule and this pair's numbers. Drag a threshold to read P(latency > x) for each run. |
| 3 | `3-heat-map.png` | Six views, none animated. The heat map shows where each run's samples fall, step by step, on a shared scale. |
| 4 | `4-geometry-drift.png` | ASPIRE training geometry. A reading the data cannot support is withheld, with the reason. |
| 5 | `5-workbench.png` | Workbench: weigh a setting across many runs. A local model proposes; ScalarScope tests and decides. |
| 6 | `6-guide.png` | A searchable guide to every rule on the page. |

## Sources

| Shot | What is open |
|---|---|
| 1–3 | The built-in sample comparison (`rust/samples/baseline.csv`, `rust/samples/optimized.csv`): Series view, then Distribution with the ΔO Why panel open, then Heat map |
| 4 | `samples/geometry/local-teacher.drift.geometry.json` against `composite-teacher.drift.geometry.json`, from two real aspire-si training runs |
| 5 | The nine simulated runs in `samples/workbench` (the page says they are filed under "simulated") |
| 6 | Guide |

## Retaking them

The release build has a capture mode that is off unless `SCALARSCOPE_CAPTURE` is set. With it set, the window opens at 1920 × 1080, sets the page, view and Why panel, writes a PNG of itself, and closes. From the repo root:

```bash
SCALARSCOPE_CAPTURE=docs/store/1-compare-headline.png SCALARSCOPE_CAPTURE_SCALE=1.3 \
SCALARSCOPE_CAPTURE_PAGE=compare SCALARSCOPE_CAPTURE_VIEW=1 \
  rust/target/release/scalarscope.exe rust/samples/baseline.csv rust/samples/optimized.csv
```

The other variables are `SCALARSCOPE_CAPTURE_WHY` (a delta symbol such as `ΔO`) and `SCALARSCOPE_CAPTURE_RUNS` (a folder of runs for the Workbench). `SCALARSCOPE_CAPTURE_PAGE` takes `welcome`, `compare`, `workbench`, `history`, `guide` or `settings`.
