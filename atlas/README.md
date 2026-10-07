# scalarscope: how it works

Mapped at 2026-10-07 from commit 9a109be by Atlas 1.24.0.

## What this is

A review of two machine-learning runs. The app is the Rust program in rust/, packed as an MSIX for the Microsoft Store; src/ holds the earlier .NET MAUI app the Store copy still runs. (written by a person)

10 parts, mostly C# (181 files), Rust (30), CSS (2), PowerShell (2), TypeScript (2), Astro (1), JavaScript (1) and shell (1). Work enters through 6 doors; the busiest is Build and Test, which reaches 4 parts. It deploys a site to GitHub Pages. People run scalarscope.

## What changed since 2026-10-07 (3d471dc)

Nothing structural changed since 2026-10-07; 6 files changed content.

## What comes in

1. **Build and Test.** On a pull request to main touching 10 paths; on a push to main touching 10 paths; or by hand. Runs packaging/pack.ps1 and tests/ScalarScope.FixtureTests/ScalarScope.FixtureTests.csproj; builds src/ScalarScope/ScalarScope.csproj and src/VortexKit/VortexKit.csproj; checks rust/src/lib.rs and rust/src/main.rs.
2. **Release.** When a tag matching `v*` is pushed; or by hand. Runs packaging/pack.ps1; builds rust/src/main.rs; checks rust/src/lib.rs.
3. **Coverage.** On a pull request; on a push to main. Runs rust/src/geometry.rs, rust/src/shape.rs, rust/src/stats.rs and 14 more.
4. **Deploy site to GitHub Pages.** On a push to main touching 2 paths; or by hand. Runs site/astro.config.mjs and site/src/.
5. **Publish to NuGet.** When a release is published; or by hand. Builds src/VortexKit/VortexKit.csproj.
6. **scalarscope** (a command people run). Runs rust/src/main.rs.

## What happens through Build and Test

1. The workflow runs packaging/pack.ps1 in packaging and tests/ScalarScope.FixtureTests/ScalarScope.FixtureTests.csproj in tests; it builds src/ScalarScope/ScalarScope.csproj and src/VortexKit/VortexKit.csproj in src; it checks rust/src/lib.rs and rust/src/main.rs in rust.

## Who reads the results

Build and Test writes nothing this map can see.

## The other doors

**Release** runs packaging/pack.ps1, checks rust/src/lib.rs, creates a GitHub release, and builds rust/src/main.rs into a binary for Windows and uploads them to the release.

**Coverage** runs rust/src/geometry.rs, rust/src/shape.rs, rust/src/stats.rs and 14 more, and uploads coverage to Codecov.

**Deploy site to GitHub Pages** runs site/astro.config.mjs and site/src/, and deploys the site.

**Publish to NuGet** builds src/VortexKit/VortexKit.csproj.

**scalarscope** (a command people run) runs rust/src/main.rs.

## What breaks what

- **rust** is imported by no other part and sits on the path of 4 doors.
- **packaging** is imported by no other part and sits on the path of 2 doors.
- **src** is imported by no other part and sits on the path of 2 doors.

packaging and src hold only C# and PowerShell files, which this map does not read, so what uses them cannot be seen.

## What tends to change together

- **rust/src/open.rs** and **rust/tests/review_tests.rs** changed together in 9 of 11 commits, inside the rust part.
- **rust/src/review.rs** and **rust/src/ui.rs** changed together in 15 of 19 commits, inside the rust part.
- **rust/src/ui.rs** and **rust/src/ui_tests.rs** changed together in 12 of 19 commits, inside the rust part.
- **rust/src/review.rs** and **rust/tests/review_tests.rs** changed together in 10 of 16 commits, inside the rust part.
- **rust/src/ui_tests.rs** and **rust/tests/edge_tests.rs** changed together in 8 of 13 commits, inside the rust part.

Confidence is low: fewer than 25 source files reach 10 revisions in the window.

Window: 180 days; a pair counts from 3 shared commits, since 6 source files reach 10 revisions; the floor rises to 10 when 25 do.

## What no test touches

Every code part this map reads is imported by at least one test.

packaging and src hold only C# and PowerShell files, which this map does not read, so whether a test touches them cannot be seen.

verify.ps1 runs in no workflow.

verify.sh runs in no workflow.

## Written but never read

No place this map can see is written, so none goes unread.

## Helpers that look duplicated

No two parts export a helper that looks alike.

## Generated, never hand-edited

Nothing in this repository writes to a tracked place this map can see.

## Hand-authored

People write .github/, UI_Reference/, assets/, docs/, the repository root and site/; 3 writes with paths built at run time may land here.

## Where to start

rust/src/main.rs → rust/src/lib.rs → rust/src/ui.rs

Read those in order to follow one run of scalarscope end to end. This path follows scalarscope (a command people run) from its entry, since Build and Test only checks code.

## What this map cannot see

- 3 writes and 3 reads use paths built at run time and are not named here.
- 4 writes and 11 reads go to a path their caller passes, not to this repository.
- 1 write goes to a temporary directory, not to this repository.
- Statistics confidence is low: fewer than 25 source files reach 10 revisions in the window.

Regenerate with `npx --yes @dogfood-lab/atlas map`.
