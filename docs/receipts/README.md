# Receipts

## Live workbench sessions, 2026-10-06 (phase 5 exit test, run 1)

Two sessions with `qwen3:14b` on the local Ollama. The digest is in each file. They were run with `rust/examples/live_workbench.rs`.

- **`workbench-golden-pair.json`** (2.0 golden pair, 6 calls, 34 s). The model tried a list of measures where a formula goes, and the program refused it twice. Its first tool, `p99 / p90`, was refused: the runs have 7 and 13 steady samples, and a p99 needs 368. It then kept `initial_slowdown = quantile_between(0, 9, 0.5) / p50` (1.25 and 1.06) and finished with a note. It proposed nothing; the files record no knobs.
- **`workbench-knob-folder.json`** (the simulated batch folder, 6 calls). It kept `tail_heaviness = p99 / p90`. Twice it proposed "raise batch → tail_heaviness up". The program refused the hypothesis's reason both times, because the reason said "99th percentile" and the fence forbids digits in the model's words. The session ended with no hypothesis recorded.

Every number in both records is the program's, and every refusal is the program's. **What the exit test asked for is only partly met.** No session left a hypothesis in a program-set state, because the one proposed was stopped at the wording fence.

The finding: the fence was written for RunForge's loss measures, whose names have no digits. A latency workbench names its measures `p50` and `p99`, so a reason that names them is refused, and the refusal does not say how to fix it. The fence lives in the shared `workbench` crate. The fix belongs there, followed by a second run.

## Run 2, 2026-10-06, after the fence fix (runforge#9, workbench pinned at `39c0ce7`)

Same driver, model and digest. The files are in `run2/`.

- **The fix works as built.** Refusals now end "name the measure (p50, p90, p99) and write no other digits", and an exact `p99` would pass.
- **The model did not use it.** On the knob folder, qwen3:14b proposed "raise batch → tail_heaviness up" twice. Both reasons were word for word the same and again said "99th percentile". Up to three calls run in one round, so the record cannot show whether the second came before or after the model read the first refusal. The program then offered only finish. On the golden pair its closing note was dropped for the same reason. No hypothesis was recorded, again.

The fence did its job twice: it kept an ordinal out of the model's words. What is left is how the model is asked. The tool schema's description of `why` says "in words, with no numbers" but never says to call a measure by its name. Whether that changes is a decision for the director or the Publisher, because the description is shared with RunForge.

## Run 3, 2026-10-06, with ScalarScope's own reason hint (runforge#10, workbench pinned at `0192366`)

Same driver, model and digest, with the GPU checked free before the run and the model unloaded after it. The files are in `run3/`, and each step now records its chat round.

- **The exit test is met.** On the knob folder the model proposed two hypotheses, in rounds 3 and 4. Their reasons named measures by name. The program recorded both and set their states:
  - **h1**, "When batch size goes up, tail_heaviness goes higher": **inconclusive**. Batch 8 against batch 1 gives 2.81 against 2.71, a gap inside the spread between seeds of one setting (1.68).
  - **h2**, "When batch size goes up, throughput_p50 goes higher": **passes its test on these runs alone**. 497 against 182; every comparison of runs agrees under the exact rank test.
- **The golden pair** still records no hypothesis. That is right: those files record no knobs. The model's note was kept this time.

Three things seen in the record. None changes a state, and each is for the workbench crate's owner:

1. The model's closing note says the throughput hypothesis "is supported". The program's state is "passes its test on these runs alone": a single set of runs, not a checkpoint verdict. The note sits under the label that says it is the model's words, which is why that label exists.
2. The program's sentence reads "On these runs it is passes its test on these runs alone." That state word does not fit the sentence frame it is put in. This is a wording bug in the shared crate, and RunForge prints the same frame.
3. h1's run plan says "batch size at 1 and 4" after comparing 1 against 8. The plan picks two levels by its own rule, and whether that is the intended pair should be checked in `experiment_for`.

The first two runs stay as the record of what it took: the fence fix (runforge#9), then the host's own reason hint (runforge#10).
