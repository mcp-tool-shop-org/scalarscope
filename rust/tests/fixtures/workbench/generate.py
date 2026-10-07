"""Write the workbench fixture: nine SIMULATED inference RunTraces.

These are not measurements. They are drawn from the model below so the workbench's exit test
has runs that vary one knob (batch size 1, 4 and 8, three seeds each) with everything else
equal. Real timings would be better; they need GPU time and a model to serve.

The model, per step i of 400:
  latency = (4 + 1.5 * batch) * warmup(i) * noise * spike
  warmup(i) = 1 + 3 * exp(-i / 2) for the first 8 steps, then 1
  noise     = exp(0.05 * z), z standard normal
  spike     = 3 with probability 0.01 * batch, else 1
So a larger batch is slower per step and has a heavier tail (p99 / p50 goes up).

Run from this folder: python generate.py. The output is byte-for-byte repeatable.
"""

import json
import math

STEPS = 400
WARMUP = 8


class Rng:
    """xorshift64*, so the files do not depend on Python's generator."""

    def __init__(self, seed):
        self.state = (seed * 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF or 1

    def uniform(self):
        x = self.state
        x ^= x >> 12
        x ^= (x << 25) & 0xFFFFFFFFFFFFFFFF
        x ^= x >> 27
        self.state = x
        return ((x * 0x2545F4914F6CDD1D) & 0xFFFFFFFFFFFFFFFF) / 2.0**64

    def normal(self):
        u1 = max(self.uniform(), 1e-12)
        u2 = self.uniform()
        return math.sqrt(-2.0 * math.log(u1)) * math.cos(2.0 * math.pi * u2)


def run(batch, seed):
    rng = Rng(batch * 1000 + seed)
    base = 4.0 + 1.5 * batch
    latency, throughput, memory = [], [], []
    for i in range(STEPS):
        warm = 1.0 + 3.0 * math.exp(-i / 2.0) if i < WARMUP else 1.0
        noise = math.exp(0.05 * rng.normal())
        spike = 3.0 if rng.uniform() < 0.01 * batch else 1.0
        value = round(base * warm * noise * spike, 4)
        latency.append(value)
        throughput.append(round(batch * 1000.0 / value, 4))
        memory.append(round(800.0 + 120.0 * batch + 2.0 * rng.normal(), 3))
    return {
        "schemaVersion": "1.0.0",
        "runId": f"workbench-batch{batch}-seed{seed}",
        "runType": "inference",
        "framework": "simulated",
        "createdUtc": "2026-10-06T00:00:00Z",
        "label": f"batch {batch} seed {seed}",
        "metadata": {
            "modelFingerprint": "5" * 64,
            "datasetFingerprint": "a" * 64,
            "codeFingerprint": "b" * 64,
            "environmentFingerprint": "c" * 64,
            "seed": seed,
            "tags": ["fixture", "simulated", "workbench"],
            "knobs": {"batch": batch, "precision": "fp16"},
        },
        "timeline": {"steps": list(range(STEPS)), "wallTimeSeconds": [round(i * 0.05, 2) for i in range(STEPS)]},
        "scalars": {
            "series": [
                {"name": "latency_ms", "unit": "milliseconds", "aggregation": "none", "values": latency},
                {"name": "throughput_items_per_sec", "unit": "items/s", "aggregation": "none", "values": throughput},
                {"name": "memory_mb", "unit": "MiB", "aggregation": "none", "values": memory},
            ]
        },
        "milestones": {
            "list": [
                {"type": "warmup_end", "step": WARMUP - 1, "label": "Warmup ends"},
                {"type": "steady_state_start", "step": WARMUP, "label": "Steady state begins"},
            ]
        },
        "artifacts": [],
        "capabilities": {"hasLatency": True, "hasThroughput": True, "hasMemory": True},
        "provenance": {"source": "generate.py", "sourceVersion": "1", "ingestedUtc": "2026-10-06T00:00:00Z"},
    }


if __name__ == "__main__":
    for batch in (1, 4, 8):
        for seed in (1, 2, 3):
            name = f"batch{batch}_seed{seed}_runtrace.json"
            with open(name, "w", encoding="utf-8", newline="\n") as out:
                json.dump(run(batch, seed), out, indent=1)
                out.write("\n")
