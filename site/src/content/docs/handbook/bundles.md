---
title: Bundles & Review
description: Saving a review as a .scbundle, how its hash is checked, and review mode.
sidebar:
  order: 7
---

A bundle is a review saved as one `.scbundle` file, a zip archive. Open it later, or send it to someone, and it shows the review that was saved. Its SHA-256 hash is a content check, not a signature.

## Saving

**Save bundle** on Compare writes the review open on the page. Both sides have to be open. The page then says "Saved the stored review" and gives the bundle's hash.

## What a bundle holds

| File | What it holds |
|------|---------------|
| `review/review.json` | The stored review: the headline, the tiles, the notices, and what the page needs to redraw (see below) |
| `findings/deltas.json` | The deltas that fired, as 2.0 wrote them, so 2.0 can read the bundle |
| `findings/why.json` | The sentence for each of those deltas, as 2.0 wrote it |
| `findings/summary.md` | The review in Markdown |
| `repro/repro.json` | Hashes of the stored review and of the findings, and the sample count |
| `environment/environment.json` | The app version, the platform and whether the process was 64-bit |
| `manifest.json` | The bundle id, the versions and a privacy block |
| `README.md` | A short note on what the file is |
| `integrity.json` | Each file's hash and the bundle hash |

What `review/review.json` keeps for redrawing depends on the review:
- **An inference review:** the steady-state series, the bands, the distributions and the percentiles of both runs.
- **A training review:** both loss curves.
- **A geometry review:** both runs in full.

File paths and the machine name are not stored. In an inference or training review, a run label that is a file path is cut to its last part.

A bundle contains run data, and its manifest says so (`includesRawData`): an inference bundle holds both runs' series, and a geometry bundle contains both runs in full. Share a bundle only if you would share the runs.

## The hash

1. Each file's SHA-256 is taken over its bytes as stored in the archive.
2. The paths are sorted in ordinal order, and each is written as `path:hash` on its own line.
3. The SHA-256 of that text is the bundle hash.

`integrity.json` lists every file's hash and the bundle hash, and is not part of the hash itself. The hash checks that the bytes are the ones it was computed from. It does not say who made them: anyone who changes a file can write a new `integrity.json` to match.

## Opening a bundle

**Open bundle** checks the hash before anything is shown. ScalarScope refuses the bundle, and opens nothing, when:
- `integrity.json` is missing or cannot be read;
- a listed file is missing, or a file's bytes do not match its hash;
- the archive holds a file that is not listed;
- the bundle hash does not match.

The message names what failed: "The bundle hash does not match these bytes (…)". The one exception is a 2.0 auto-saved review, described below.

## Review mode

A bundle that opens puts the page in review mode. A banner says a stored review is open, and loading runs is off until **Close review**. The page shows the hash and **Copy hash**.

- **The headline and the tiles are the stored ones**, the text the hash vouches for.
- **Inference and training reviews** draw their charts from the stored series.
- **A geometry review** redraws its views from the two stored runs, and today's rules read those runs again.
  - If today's rules reach a different verdict, it appears below the stored one, labelled "Current reading (rules since 3.1.1)". It is never shown as the stored verdict.
  - If the bundle stored no tiles, the tiles come from today's rules and are labelled "Current reading".

Review mode is read-only. Nothing on the page changes the bundle.

## Bundles from 2.0

- **A 2.0 comparison bundle** has `integrity.json` and its `findings/` files, but no `review/review.json`.
  - Its hash is checked as above, and a mismatch is refused.
  - The page shows the findings 2.0 stored, as tiles, without recomputing them.
  - It has no series, so there is no chart.
- **A 2.0 auto-saved review** is `review/review.json` alone, with no `integrity.json`, so nothing can check its bytes.
  - It opens marked "Unverified 2.0 review", and the hash shown is the one its JSON states.
  - The tiles are the deltas 2.0 stored, and there is no chart.
