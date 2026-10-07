# Samples

Files to open in ScalarScope 3.1.0. [TESTING.md](../TESTING.md) says which to open where.

| Folder | What it is | Where it comes from |
|---|---|---|
| `inference/` | Two inference runs of one model, before and after an optimization: `baseline.runtrace.json` and `optimized.runtrace.json` | The 2.0 golden pair (`tests/Fixtures/InferenceOptimization`). They are synthetic RunTrace files. |
| `geometry/` | Two real ASPIRE training runs, compared two ways. `local-teacher` is trained against one teacher, and `composite-teacher` against a vote of two. The `.drift` files are each checkpoint's hidden state minus the base model's. | aspire-si, 2026-10-07 (`rust/tests/fixtures/aspire-si/real-*`), schema 1.1 |
| `workbench/` | Nine inference runs at batch size 1, 4 and 8, three seeds each, with the batch size recorded as a knob | **Simulated** (`rust/tests/fixtures/workbench`, made by its `generate.py`) |

Each file is identical to the fixture it comes from. A test in `rust/tests/samples_tests.rs` checks that, and checks that each file opens.
