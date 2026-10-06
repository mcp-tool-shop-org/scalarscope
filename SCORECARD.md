# Scorecard

**Repo:** mcp-tool-shop-org/scalarscope
**Date:** 2026-10-06
**Type tags:** `[desktop]` Windows review. Store package `3.0.0.0`. Not an npm package.

## Pre-Remediation Assessment

| Category | Score | Notes |
|----------|-------|-------|
| A. Security | 6/10 | SECURITY.md and a privacy line existed. The README had no Trust model heading the gate recognizes. |
| B. Error Handling | 4/10 | The window shows a sentence. It does not carry hint, cause, and retryable. |
| C. Operator Docs | 7/10 | README, CHANGELOG, and LICENSE were already real. |
| D. Shipping Hygiene | 5/10 | CI builds and tests. No `verify.sh`. No release tag. The MSIX is built in CI and not installed here. |
| E. Identity (soft) | 4/10 | Seven README translations and a site existed. The landing hero still introduced a .NET MAUI app. No homepage or topics. |
| **Overall** | **26/50** | |

## Key Gaps

1. The landing hero said the product only loads two TFRT traces.
2. The README had no Trust model section.
3. GitHub Pages was not enabled, so the handbook was not a live site.
4. The error sentence is not the full structured shape.
5. Fifteen feature-audit highs stay open. This treatment does not close them.

## Post-Remediation

Measured 2026-10-06. The fifteen feature-audit highs stay open. This treatment does not close them.

| Category | Score | Notes |
|----------|-------|-------|
| A. Security | 8/10 | Trust model is in the README. `shipcheck security-docs` passed. Vulnerability alerts are on. No Dependabot update bot. |
| B. Error Handling | 4/10 | Unchanged. The window shows a sentence. It does not carry hint, cause, and retryable. |
| C. Operator Docs | 8/10 | README, CHANGELOG, LICENSE, and the seven handbook pages. The handbook was not re-scaffolded. |
| D. Shipping Hygiene | 6/10 | `verify.sh` is the product check. No tag. The MSIX is not installed. The site npm tree reported 18 vulnerabilities and was not force-fixed. |
| E. Identity (soft) | 9/10 | Logo, coverage badge at 96%, GitHub metadata, seven translations, and a live landing page. The handbook's dark palette finding stays open. |
| **Overall** | **35/50** | |

Translations: TranslateGemma 27B, concurrency 1, 7/7 succeeded in 388.6s. Japanese has CJK text and a translated trust-model section. `README.pt.md` was not left behind.

`shipcheck audit` before the live check: Checked 12, Unchecked 1, Skipped 24. After the landing line was checked against the live site: Checked 13, Unchecked 0, Skipped 24. The landing page is live. Re-fetched HTML for `/`, `/handbook/`, `/handbook/rust-review/`, and `pagefind/pagefind.js` all returned 200. The hero contains "Rust review" and "previous .NET package". ASPIRE is not in that hero. The published Japanese README contains the trust-model heading and the content-check sentence. Identity scan of those fetched pages was CLEAN.

CI on `c560ed6` was green: Build and Test, Coverage, and Deploy site to GitHub Pages.

`shipcheck security-docs` passed. Identity scan of the tree before translations was CLEAN.

`bash verify.sh` passed in this treatment: FixtureTests 131/131, and the locked Rust suites (ui 18, bundle 11, edge 18, history 10, prefs 7, review 19). It does not pack or install.

Local site build: 8 pages, search index at `dist/pagefind/`. `site/dist` is gitignored. `npm ci` in `site/` reported 18 vulnerabilities (3 low, 1 moderate, 13 high, 1 critical). Those are the site's build tools. They are not a claim about the Rust review or the .NET shell, and `npm audit fix --force` was not run.

Codecov badge for this repo returned 96%. The coverage workflow already uploads that number. No second coverage job was added.

Swarm `swarm-1791037308-1945` remains `test`. Open findings at collect: 15 high, 72 medium, 11 low. Gate F against the public dogfood index is still `DOGFOOD_NO_RECORD`. Atlas was not applied. The engine cannot read the C# or Rust product, so there is no boundary file.

No tag. Store version stays `3.0.0.0`. The unsigned MSIX was not rebuilt and was not installed.
