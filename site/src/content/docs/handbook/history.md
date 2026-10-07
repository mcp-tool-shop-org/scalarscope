---
title: History
description: A project's measures across its logged reviews, with the points where they shifted.
sidebar:
  order: 6
---

In the Store package, each inference comparison you finish is added to the comparison log. It records side B's RunTrace fingerprints and its measures: p50, p90, p99, p99/p50, steady mean, throughput and memory peak. The log keeps the last 40 reviews.

## Projects

The **History** tab groups reviews into projects. A project is side B's dataset and model fingerprints. The code and environment are left out of the key on purpose: their changes are what the history is for. Reviews without fingerprints, and every review 2.0 logged, go to **Unsorted**.

## Shifts and changes

Pick a project and a measure. The plot draws the measure across the project's reviews, oldest first.

- **A solid line** marks where the program finds a new level. It uses PELT with a mean-shift cost on the logarithm, scaled by one robust noise estimate for the series. The penalty is 2 · ln n, and each segment is at least three reviews long.
- **A dashed line** marks a review whose code or environment fingerprint differs from the one before it.
- **A shift within one review of a change** gets one sentence, such as "p99 shifted at 2026-10-15, with an environment change". That names a coincidence, not a cause.

With fewer than six reviews of a measure, the page draws the points and says there are too few to look for shifts.
