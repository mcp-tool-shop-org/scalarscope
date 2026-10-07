# Workbench fixture (simulated)

Nine inference RunTraces: batch size 1, 4 and 8, three seeds each, with precision `fp16` on every run. Each file carries its knobs in `metadata.knobs`.

**These runs are simulated, not measured.** `generate.py` draws them from a stated model, in which a larger batch is slower per step and has a heavier tail. They exist so the workbench's exit test has runs that vary one knob while everything else stays equal. The 2.0 golden pair records no knobs at all. Real inference timings would be better, but they need GPU time and a model to serve.

To regenerate, run `python generate.py` in this folder. The output is the same every time.
