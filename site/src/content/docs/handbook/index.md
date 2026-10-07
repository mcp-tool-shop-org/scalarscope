---
title: Handbook
description: Everything you need to know about ScalarScope 3.1.0.
sidebar:
  order: 0
---

Welcome to the ScalarScope handbook: the guide to comparing two machine-learning runs, and to reading how sure the comparison is.

## What's inside

- **[Getting Started](/scalarscope/handbook/getting-started/)**: install, run the review, and see a file of each kind
- **[The review](/scalarscope/handbook/rust-review/)**: Compare, the headline, the tiles and the views
- **[Delta Analysis](/scalarscope/handbook/delta-analysis/)**: the deltas and the rules that make them fire
- **[Geometry](/scalarscope/handbook/geometry/)**: two ASPIRE training runs, and the export contract
- **[Workbench](/scalarscope/handbook/workbench/)**: weigh a setting across many runs with a local model
- **[History](/scalarscope/handbook/history/)**: a project's measures across its logged reviews
- **[Bundles & Review](/scalarscope/handbook/bundles/)**: saved reviews, the hash check and review mode
- **[Beginners](/scalarscope/handbook/beginners/)**: a step-by-step first comparison
- **[VortexKit](/scalarscope/handbook/vortexkit/)**: 2.0's visualization library, kept in the repo

## What is ScalarScope?

ScalarScope opens two inference traces, two training histories, or two ASPIRE geometry exports. It says how run B differs from run A, gives each difference an interval or its reason, and holds back what the data cannot support. A matching SHA-256 on a saved review is a content check, not a signature.

Version 3.1.0 is the Rust review, on the Microsoft Store as `9P3HT1PHBKQK`, an update of the same listing. Its tabs are **Welcome**, **Compare**, **Workbench**, **History**, **Guide** and **Settings**. Files, bundles and settings from 2.0 carry over.

[Back to landing page](/scalarscope/)
