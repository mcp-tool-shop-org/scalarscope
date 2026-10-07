---
title: Geometry
description: Comparing two ASPIRE training runs, and the export contract that says when a reading cannot stand.
sidebar:
  order: 4
---

A geometry file is the export that [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) writes for one training run:
- the student's states projected to two dimensions;
- each evaluator's scores per step;
- the eigen spectrum of the states;
- the evaluators' directions;
- the failures it recorded.

Open two on **Compare** with Open path A and Open path B.

## The page

- **The header** names each run, its teachers and its layout.
- **The headline** is built from the deltas that stand.
- **Five tiles:**
  - **ΔF:** failure events.
  - **ΔTc:** convergence timing.
  - **ΔTd:** structural emergence.
  - **ΔĀ:** spectrum concentration, λ1/Σλ, how much of the evaluators' spread falls in one direction. 2.0 called it evaluator agreement.
  - **ΔO:** stability.
- **The trajectory**, with each evaluator's direction.
- **A panel per evaluator score.**
- **Speed, curvature, effective dimension and anisotropy.**
- **The eigen spectrum over time.**
- **The failures** on a shared timeline.
- **A time slider** that moves a marker through every view. Nothing animates.

The five rules are 2.0's, ported and checked against 2.0's own results. The corrections made in 3.x are listed with their reasons in the spec ([decision brief](https://github.com/mcp-tool-shop-org/scalarscope/blob/main/docs/parity-and-beyond.spec.md)).

## The export contract (schema 1.1)

A geometry export can state two things about its own layout in `run_metadata`:
- **`step_axis`:** `training_step` means the steps are training time. `checkpoint_by_item` means the steps are blocks, one per checkpoint, over a fixed item order, as in a drift export. Then `checkpoints` gives the number of blocks.
- **`scalar_source`:** `live`, `replayed` (epochs after the first replay cached scores), or `fixed_per_item`.

When a run's steps are not time:
- ΔTc, ΔTd, ΔO and ΔF are **withheld**, with the reason "steps are checkpoint × item, not time".
- The trajectory is drawn as unordered points, one shade per checkpoint.

When a run's scores repeat:
- ΔF is withheld, because its failures are score dips.
- The score panels say so.

In both cases, the "Why" of a withheld tile keeps what 2.0 would have said, and the headline speaks only from the deltas that stand. A file that states neither field, such as a schema 1.0 file, is read exactly as 2.0 read it.

## Reading real runs

Two real training runs, compared per step and by drift, are in `samples/geometry`. Their screenshots and a reading of each delta are in the repo's [geometry receipt](https://github.com/mcp-tool-shop-org/scalarscope/blob/main/docs/receipts/geometry/README.md).

One limit holds even with the contract. In a per-step export, consecutive steps are often different prompts. ΔO can then read the student moving from prompt to prompt as instability. The contract cannot tell the two apart, because those steps really are in training order.
