# Ship Gate

> No repo is "done" until every applicable line is checked.
> Copy this into your repo root. Check items off per-release.

**Tags:** `[all]` `[desktop]` — Windows review. Not an npm package, not an MCP server, not a flag CLI.

**Date:** 2026-10-06. Store package version stays `3.0.0.0`. This treatment does not cut a tag and does not upload the MSIX.

---

## A. Security Baseline

- [x] `[all]` SECURITY.md exists (report URL, supported versions, response timeline) — `shipcheck security-docs` passed the contact check (2026-10-06)
- [x] `[all]` README includes a Trust model (data touched, data not touched, permissions) — `shipcheck security-docs` (2026-10-06)
- [x] `[all]` No secrets, tokens, or credentials in the git tree — identity scan of the git-tracked tree was CLEAN on 2026-10-06. `shipcheck secrets` skips because there is no npm package to pack.
- [x] `[all]` No telemetry by default — stated in the Trust model and in SECURITY.md (2026-10-06)

### Default safety posture

- [ ] `[cli|mcp|desktop]` SKIP: the review does not kill, delete, or restart anything. Opening a file and saving a bundle are the person's own actions, so there is no `--allow-*` flag.
- [ ] `[cli|mcp|desktop]` SKIP: a bundle is written only to the path the person picks. A packaged run's comparison log and preferences stay in that package's LocalState folder. The review does not walk the disk for its own path.
- [ ] `[mcp]` SKIP: not an MCP server
- [ ] `[mcp]` SKIP: not an MCP server

## B. Error Handling

- [ ] `[all]` SKIP: the review shows one sentence on the window. The .NET shell has catalog codes such as `FILE_FORMAT_INVALID`. Neither surface puts `hint`, `cause`, and `retryable` on every error.
- [ ] `[cli]` SKIP: the review is a window, not a flag CLI, so it has no process exit codes.
- [ ] `[cli]` SKIP: not a flag CLI. The window does not print a stack.
- [ ] `[mcp]` SKIP: not an MCP server
- [ ] `[mcp]` SKIP: not an MCP server
- [x] `[desktop]` Errors on the review are a sentence on the window, not a raw stack (2026-10-06)
- [ ] `[vscode]` SKIP: not a VS Code extension

## C. Operator Docs

- [x] `[all]` README is current: what it does, how to run the review, the Store listing, and the package version (2026-10-06)
- [x] `[all]` CHANGELOG.md follows Keep a Changelog (2026-10-06)
- [x] `[all]` LICENSE is MIT, and SECURITY.md states the supported version (2026-10-06)
- [ ] `[cli]` SKIP: the review is a window. It has no `--help` command list.
- [ ] `[cli|mcp|desktop]` SKIP: there is no diagnostic log with silent, normal, verbose, and debug levels. The review shows a note and does not write a secret log.
- [ ] `[mcp]` SKIP: not an MCP server
- [ ] `[complex]` SKIP: not a background daemon. The product guide is the Starlight handbook, not an ops runbook.

## D. Shipping Hygiene

- [x] `[all]` `verify.sh` runs `verify.ps1`: FixtureTests and the locked Rust suite. It does not pack, install, or soak. (2026-10-06)
- [ ] `[all]` SKIP: there is no release tag. The Store version stays `3.0.0.0`. This treatment does not cut a tag. `shipcheck manifest` skips because there is no npm manifest.
- [ ] `[all]` SKIP: `shipcheck ci` finds no root package.json, Cargo.toml, or pyproject.toml. Lockfiles live in `rust/` and `site/`. Dependabot update PRs stay off (org CI-minutes rule). The site job sets `ASTRO_TELEMETRY_DISABLED`.
- [ ] `[all]` SKIP: `shipcheck deps` audits npm lockfiles only. The product dependencies are the Rust crate and the .NET projects. This line is not claimed from an npm audit of the site.
- [ ] `[all]` SKIP: no Dependabot update bot and no dependabot.yml. Vulnerability alerts are on as of 2026-10-06. Update PRs stay off.
- [ ] `[npm]` SKIP: not published to npm
- [ ] `[npm]` SKIP: not published to npm
- [ ] `[npm]` SKIP: no root npm package. `site/package-lock.json` and `rust/Cargo.lock` are committed.
- [ ] `[npm]` SKIP: not an npm or PyPI package
- [ ] `[vsix]` SKIP: not a VS Code extension
- [ ] `[desktop]` SKIP: the pack-review job builds the unsigned x64 MSIX. This treatment does not install it and does not upload it to Partner Center.

## E. Identity (soft gate — does not block ship)

- [x] `[all]` Logo in the README header, from the brand repo, 1024×1024 (2026-10-06)
- [x] `[all]` Translations (polyglot-mcp, 8 languages) — local TranslateGemma 27B, concurrency 1, 7/7 succeeded on 2026-10-06. Japanese is Japanese.
- [x] `[org]` Landing page (@mcptoolshop/site-theme) — live at https://mcp-tool-shop-org.github.io/scalarscope/ on 2026-10-06. Handbook and pagefind returned 200. The hero says the Rust review, and the Store copy is still the previous .NET package.
- [x] `[all]` GitHub repo metadata: description, homepage, topics (2026-10-06). Description names the Rust review and the previous Store package. Homepage is the Pages URL. Topics are rust, windows, machine-learning, and inference.

---

## Gate Rules

**Hard gate (A–D):** Must pass before any version is tagged or published.
If a section doesn't apply, mark `SKIP:` with justification — don't leave it unchecked.

**Soft gate (E):** Should be done. Product ships without it, but isn't "whole."

**Checking off:** `- [x]` with a date. **Skipping:** `- [ ]` with `SKIP:` and a reason.
