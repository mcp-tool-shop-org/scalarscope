---
title: Handbook
description: Everything you need to know about ScalarScope.
sidebar:
  order: 0
---

Welcome to the ScalarScope handbook. This is the complete guide to comparing ML inference runs with scientific rigor.

## What's inside

- **[Getting Started](/scalarscope/handbook/getting-started/)** — Install, run the review, and see a file of each kind
- **[Rust review](/scalarscope/handbook/rust-review/)** — The program this repo builds
- **[Delta Analysis](/scalarscope/handbook/delta-analysis/)** — The five canonical delta types and detector configuration
- **[Bundles & Review](/scalarscope/handbook/bundles/)** — Reproducible exports, bundle profiles, and review mode
- **[VortexKit](/scalarscope/handbook/vortexkit/)** — The standalone visualization framework
- **[Beginners](/scalarscope/handbook/beginners/)** — Step-by-step guide for new users

## What is ScalarScope?

Most ML teams eyeball logs. ScalarScope replaces that with structured, reproducible comparison. The program this repo builds is the Rust review. It opens an inference file or a training history, and it writes a `.scbundle`. A matching SHA-256 is a content check, not a signature.

The Store listing (ID: `9P3HT1PHBKQK`) is still the previous .NET package. That package has four tabs: **Home** (workspace and recent comparisons), **Compare** (side-by-side delta analysis), **Guide** (interpretation help), and **Settings** (theme, playback, export, accessibility).

[Back to landing page](/scalarscope/)
