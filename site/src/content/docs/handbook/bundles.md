---
title: Bundles & Review
description: Reproducible exports and review mode.
sidebar:
  order: 3
---

## Reproducible bundles

Export comparison results as `.scbundle` archives (ComparisonBundle v1.0.0). Bundles are self-contained. A matching SHA-256 is a content check, not a signature.

### Bundle contents

| File | Purpose |
|------|---------|
| `manifest.json` | Bundle metadata, app version, comparison labels, alignment mode, privacy disclosure |
| `repro/repro.json` | Input fingerprints, preset hash, determinism seed, environment info |
| `findings/deltas.json` | Canonical deltas with confidence scores, anchors, trigger types, and debug info |
| `findings/why.json` | Human-readable explanations, guardrails, parameter chips, confidence breakdowns |
| `findings/summary.md` | Auto-generated Markdown summary |
| `insights/insights.json` | *(optional)* Insight feed events for the Review Mode insights tray |

Bundles exported with the **Audit** profile may also include a `repro/audit.json` payload with full reproducibility audit data.

### Bundle profiles

Bundles can be exported in three profiles:

- **Share** — standard export for team sharing
- **Review** — optimized for read-only review
- **Audit** — includes extended reproducibility and audit data

### Integrity

Every file in the bundle is hashed with SHA-256. The bundle hash is a content check, not a signature. If any file has been modified since export, the hash does not match and Review Mode flags the discrepancy.

### Privacy

The manifest includes a privacy disclosure block that records:

- Whether raw run data is included
- Whether any PII is present
- What redactions were applied (none, labels only, paths removed)

This lets recipients understand what data the bundle contains before opening it.

## Review mode

Open any `.scbundle` without recomputing:

- A matching SHA-256 checks the archived bytes. It is a content check, not a signature.
- Frozen deltas are displayed exactly as they were at export time
- A review-mode banner says the stored review is shown, not recomputed
- The insights tray shows frozen insight events from the bundle
- Both parties see identical results when sharing bundles

Review mode is read-only — you cannot modify the bundle contents from within the app.

### Reproducibility status

Each bundle tracks its reproducibility status:

- **Reproducible** — inputs and configuration match; results can be recreated
- **Modified** — something changed (inputs, preset, seed, or delta spec)
- **Nondeterministic** — results may vary between runs

The specific reason flags (inputs changed, preset changed, seed changed, delta spec changed) are recorded so you can trace exactly what diverged.

## Export workflow

1. Complete a comparison in the **Compare** tab
2. Click **Export Bundle** in the bundle export panel
3. Choose a profile (Share, Review, or Audit)
4. Choose a location for the `.scbundle` file
5. Share the file with your team
6. Recipients open it in ScalarScope — Review mode activates automatically
