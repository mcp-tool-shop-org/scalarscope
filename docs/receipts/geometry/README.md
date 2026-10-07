# Real aspire-si exports on the geometry page (2026-10-07)

This is the second Store gate: a geometry export from a real aspire-si training run, opened in ScalarScope, with its comparison checked by eye. The fixtures are in `rust/tests/fixtures/aspire-si/real-*.geometry.json`, from aspire-si `0972ef5`.

The student is Qwen2.5-1.5B-Instruct (4-bit, LoRA r16), trained on 32 prompts for 3 epochs at batch 1, with seed 42. Side A's teacher is a local Qwen2.5-32B. Side B's teacher is a composite vote of that model and Gemma 4 31B.

Screenshots were taken from the release build, opened with `scalarscope <A> <B>`:
- `real-per-step-pair.png`: the per-step exports (96 training steps each).
- `real-drift-pair.png`: the drift exports. Each step is one checkpoint's hidden state minus the base student's, for one of the 32 exchanges, laid out checkpoint by checkpoint.

## What the five deltas say (step alignment)

| Delta | Per-step pair | Drift pair | Reading |
|---|---|---|---|
| ΔF | Fired: both fail, B first | Fired: both fail, A first | **Artefact.** The "failures" are dips in per-dimension scores: 83 and 133 per step, 88 and 138 in drift. In the per-step pair, epochs 2 and 3 replay the cached scores of epoch 1. In the drift pair, the scores repeat every 32 steps by construction. Either way they are prompt-to-prompt variation. |
| ΔTc | Quiet: neither converged | Fired: "B settled 49 steps before" (confidence 0.15) | Per step, quiet is correct. In drift it is **an artefact of the layout**: drift steps are checkpoint × exchange, not time, so "settling" means nothing there. |
| ΔTd | Quiet | Quiet | Both are dominant from step 0. |
| ΔĀ | Fired: B more concentrated, 0.74 against 0.57, over 69 steps | Fired: B more concentrated, 0.74 against 0.57, over 31 steps | **The one reading that holds in both pairs, under every alignment.** B's two teachers score differently (they agree at r = 0.53), and its evaluator spread falls more into one direction. This matches what aspire-si expected: B differs in score level, spread and the second professor. |
| ΔO | Fired: B, 2.79 against 1.84. Under first-instability alignment it flips to A. | Fired: B 1.05, A 0 | Per step, the curvature comes from hopping between prompts, which is why the run it blames depends on alignment. In drift, consecutive steps are different exchanges, so the curvature again is not instability over time. |

## Does the page read correctly on real data?

**The page renders correctly, but on these runs it tells more than the data can carry.**
- **Every field reads.** Both files open with no warnings: 96 steps, 9 dimensions, and 1 or 2 professors. The header names the teachers and the drift layout, and the score panels show the 32-prompt repeat plainly.
- **The trajectory joins each step to the next.** Per step, that draws prompt identity: the same ~32 points are revisited each epoch, and the distance between prompts is about 10⁴ times the training drift. In drift, the line joins different exchanges. Neither line is a path through time.
- **ΔF, ΔTc (drift) and ΔO fire on structure, not change.** The rules have no way to know that a step axis is not time, or that scores were replayed.
- **The evaluator labels overlap** where two teachers point the same way: "local:gemma-4-31B-it" sits over "local:Qwen2.5-32B-Instruct". This is a cosmetic bug.

**Proposed follow-up, not done here:** aspire-si could state in its export when steps are not time (`metadata.step_axis`), and when scores are replayed. ScalarScope would then withhold ΔTc and ΔO, and label the trajectory as unordered, instead of reporting them. That needs a small change to the export contract on both sides.

## G10: decided on these exports

G10 asked whether ΔO's threshold should be shared by both runs instead of adaptive per run. On both real pairs, the two runs get nearly the same per-run threshold: 0.62 and 0.62 per step, 0.63 and 0.62 in drift. A threshold pooled over both runs leaves every ΔO verdict and its owner unchanged. Per step, the run blamed stays B under step alignment and A under first-instability alignment. In drift, B is blamed with 1.03 against 1.05. The case that raised G10, a very straight run judged by its own tiny threshold, happened only on the simulated pair. **Decision: keep 2.0's per-run threshold and close G10.**

## For the director

The comparison is drawn faithfully. What a person should take from it is the ΔĀ reading and the score panels, not the other four tiles. Whether that is enough to lift the Store hold is the director's call. The screenshots are here so that call is made by looking.
