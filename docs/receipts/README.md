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
