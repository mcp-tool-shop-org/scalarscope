# Changelog

All notable changes to ScalarScope will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.1.0] - 2026-10-07

### Fixed
- Evaluator labels on the geometry trajectory no longer overlap when two evaluators point the same way.
- With `scalarscope <A> <B>`, a side that does not open keeps its error on the page after the other side loads.

### Added
- `scalarscope <path A> <path B>` opens that pair on Compare at launch.
- The geometry export contract (schema 1.1): `run_metadata.step_axis`, `checkpoints` and `scalar_source`. When a run's steps are checkpoints × items rather than time, ΔTc, ΔO and ΔF are withheld with that reason, and the trajectory is drawn as unordered points per checkpoint. When its scores are replayed or fixed per item, ΔF is withheld. The headline speaks only from deltas that stand, and the header and score panels state the layout. Files that state neither field read exactly as before.
- Fixtures from real aspire-si training runs (`rust/tests/fixtures/aspire-si/real-*`): per-step and drift exports of a local-teacher run and a composite-teacher run. Tests check that they open in full and that the evaluator-spectrum reading holds.
- A Workbench tab, on the workbench RunForge shares with it. Open runs there, or use the runs being compared. Press Ask and a local model that can call tools (Ollama on this computer; cloud models are refused) measures the runs, builds formula tools and proposes what each knob does. ScalarScope computes every number and sets every verdict: a hypothesis is not testable, confounded, inconclusive, supported or refuted on one set of runs, and verdicts across sets come only at checkpoints, by e-BH at a 5% false discovery rate. The page puts the program's verdicts first, with how many hypotheses were tried and how many hold. It then has a box for your own call before you ask, and the model's note labelled as its words. It also lists the measures the model never looked at, and Save session record writes the model, its digest, every call and answer, and no paths.
- Knobs. A run's settings are read from `metadata.knobs` in a RunTrace, a `knobs.json` beside the run, or `key=value` pairs in its folder name, in that order, and the page says where each came from.
- Latency measures for the workbench's formulas: p50, p90 and p99 of the steady samples (a percentile without enough samples is refused, as on the headline), p99/p50, steady mean, warmup cost, anomaly rate under the chosen rule, throughput, memory peak, and windows over steps.
- A History tab. Each logged inference review now records side B's fingerprints and its built-in measures. The tab groups reviews into projects by side B's dataset and model, and draws a measure across a project's reviews. It marks where the level shifts (PELT on the logarithm with one noise scale for the series) and where the code or the environment changed. A shift beside a change gets one sentence that says so and claims no cause. Fewer than six reviews are drawn, not segmented. Reviews logged by 2.0 open, under Unsorted, without measures.
- `rust/tests/fixtures/workbench`: nine simulated runs (batch 1, 4 and 8, three seeds each) with their generator, labelled as simulated.

### Changed
- The geometry delta ΔĀ is called **Spectrum concentration**. 2.0 called it evaluator agreement, but it measures how much of the evaluators' spread sits in one direction (λ1/Σλ), not whether they agree. Its tile, explanation, summary and Guide text say so. Its id and symbol are unchanged, so 2.0 bundles still match.
- ΔF names the failure it dates. With several recorded failures, 2.0 gave the time of the third but the kind of the first, so a sentence could read "correctness_dip near step 6" when the failure at step 6 was a tradeoffs_failure. Both now come from the same failure. How many failures make one persistent follows the persistence window setting (three by default, as before).
- Smaller geometry fixes, none of which changes a result on the 2.0 samples:
  - Alignment at convergence reads the same settings as ΔTc.
  - Alignment at first instability uses absolute curvature, as the stability delta does.
  - A failure on the last step is reported on the last step, not one past it.
  - When only one run settles, ΔTc's sign says which one.
  - A step without enough eigenvalues is left out of ΔĀ instead of counting as zero.

## [3.0.0] - 2026-10-03

### Added
- Export SVG writes the current view (series, warmup, distribution with its threshold, difference, spectrum) as a vector drawing with axes, legend and caption. Export PNG saves a picture of the window.
- 2.0 bundles open. A 2.0 comparison bundle shows its stored deltas as tiles, with why.json's explanation and parameters. A 2.0 inference review, which 2.0 auto-saved without integrity.json, opens as unverified: the page says its bytes cannot be checked and shows the hash its JSON states. Real 2.0 files are kept in `tests/Fixtures/Bundles`, written by the 2.0 code (`BundleFixtureWriter.cs`), and a bundle this review writes is read back by the 2.0 importer in the .NET tests.
- The app opens on a Welcome tab, as 2.0 did. It offers Compare two runs, Try the sample comparison (two synthetic runs built into the app), and Open a review bundle, with four cards on what the review gives and the recent reviews. A Guide tab explains, with search: reading a comparison, the headline, each delta, why ΔTd and ΔĀ are not on the inference page, run shapes, the views, several runs, bundles, inputs and shortcuts.
- Keyboard shortcuts: F1 opens the Guide, Ctrl+, Settings, Ctrl+H Welcome; on Compare, 1 to 6 choose the view, and Esc closes the Why panel.
- Text fields and plot backgrounds sit a shade deeper than the page, so a text field is visible on the dark theme.
- A Settings tab, written to the 2.0 `preferences.json` keys so a 2.0 user's choices carry over, with every other key kept: theme (Follow Windows, Light or Dark, 2.0's `Theme`), series colors for color-vision modes, high contrast, text scale, the recent-files limit and Clear recent files. It also has the anomaly rule: the MAD rule, or the 2.0 3-sigma rule as a choice (`AnomalyRule`, new in 3.0). The review's caption names the rule, so a stored review says which one made it. An About section gives the version, the privacy statement, and links to report an issue, the privacy policy and the source.
- A Light theme. The page, the text and the default series colors change together. 3.0 was dark only, and the window background stayed near-black whatever the theme until the app set its own clear color.
- A tile for each delta (ΔF, ΔTc, ΔO) says whether it fired, stayed quiet or was withheld. A tile opens a "Why" panel with the rule, this pair's numbers, and the parameters, plus Copy finding and Show me. Show me moves to the view the delta is about and shades the stretch. The tiles are stored in the bundle with run labels scrubbed, like the verdict.
- A Warmup view shows each run from its first sample, with where ΔTc reads it as settled shaded in its color. The series view starts at the steady state and leaves the warmup out. The samples before the window are stored in the bundle so a reopened review shows it too.
- Review mode: with a stored review open, a banner says so, the load buttons are off, and Close review returns to loading. Copy hash copies the full SHA-256 content check of the open bundle.
- The review is built once per pair of inputs, not on every frame. It runs the bootstraps.
- Elapsed time: a RunTrace's `wallTimeSeconds`, a profiler trace's event timestamps, or a CSV `time_s` / `elapsed_s` / `wall_time_s` / `timestamp_s` column give each sample its seconds since the run's first. The series view can then draw against elapsed seconds, and each run's settle range is also given in seconds. When the seconds and the steps disagree about whether the two runs settle apart (different step times), the page says so. ΔTc still counts steps. 2.0 had a wall-clock alignment mode that returned an error instead of a review.
- CPU and GPU utilization (RunTrace `cpu_*` / `gpu_*` series, or CSV `cpu_percent` / `gpu_util` columns) draw in a utilization plot under throughput.
- A side can be several runs of one configuration: pick several files together, or open a folder whose subfolders are each a run (a folder with files of its own keeps the 2.0 rule and opens as one run). The headline and the difference plot then resample whole runs as well as samples within each run, so their intervals carry run-to-run variation. The headline says what its interval covers: within one run per side, or across N runs, and it is called indicative below 3 runs per side. The series view and the deltas use the first run, and say so.
- Five views of an inference pair, none animated: **Series** (latency by step, with each run's levels as dashed lines), **Distribution** (the empirical CDF with a threshold to click or drag, read as P(latency > x) for each run, and 20 quantile dots per run), **Difference** (B − A by percentile over the steady samples, with 95% intervals and a zero line), **Spectrum** (latency against percentile on a log tail axis, each run stopping where its sample count stops bounding the percentile), and **Heat map** (step × latency per run on a shared scale, each column its own distribution). The difference points and the run levels are stored in the bundle, so a reopened review shows them.
- ScalarScope's stored RunTrace JSON opens, the format of the repo's golden fixtures. It keeps the milestones written in the file, its fingerprints and its memory series. The validator runs the 2.0 stages with the same codes (`RT_TIMELINE_*`, `RT_SCALAR_*`, `RT_MILESTONE_*`, `RT_FINGERPRINT_INVALID`, `RT_ENVIRONMENT_MISSING`, `RT_CAPABILITY_MISMATCH`). A broken timeline stops the comparison with the 2.0 message, so no delta is computed and nothing can be saved. The three 2.0 golden assertion files pass in `rust/tests/runtrace_tests.rs`.
- Fingerprint notes (model changed; dataset, code or environment differ or are absent) and the TFRT guardrails (no steady state, warmup over half the run, aggregated stats only) show under the verdict and are stored in the bundle. A bundle without them keeps the bytes and hash it had.
- A gzipped profiler trace (`.json.gz`), a runtime log (`.log`) and a run folder open. A folder opens its best source in the 2.0 order (profiler trace, benchmark CSV, benchmark JSON, runtime log), and its `config.json` `warmup_steps` sets where warmup ends. A log reads `latency_ms: 12.5` and `memory_mb: 512`, which the 2.0 patterns read as no number.
- Memory draws under throughput when both runs have it. The caption names the preset the page applies.
- A Chrome profiler trace with a complete `ProfilerStep` contributes one latency sample per step. The duration is microseconds converted to milliseconds. Ops nested in that step, including a TensorRT event, are not extra inferences. A trace with no such step still keeps events whose names contain TensorRT or inference. A trace that has neither is refused. The previous .NET connector still uses only that name filter.
- A packaged run reads `preferences.json` from the same LocalState folder. Series colors follow a saved color-vision mode, or the high-contrast palette when that checkbox is on and the mode is still Default. Text scale follows the saved value, clamped from 0.75 to 2. Recent files and gallery views reopen from that folder. Writing a recent file keeps every other field. A missing file stays in the list. A file that is not a JSON object is left unchanged. Plugins in that folder are not loaded.
- The release workflow packs the Rust review as the unsigned `ScalarScope_3.0.0.0_x64.msix`. The package name stays `mcp-tool-shop.ScalarScope`, the publisher stays `CN=5305D976-6952-4F00-9C21-3A5DB090359F`, the version stays `3.0.0.0`, and the architecture stays x64. It is not uploaded. The Store copy is still the previous .NET package. A packaged run keeps `comparison-log.json` in that package's LocalState folder. An unpackaged run does not write that folder.
- An inference series draws a vertical line at the steady-state milestone when both sides have one. A missing milestone draws no line, and the last step is not called a stabilization time.
- The Rust review writes a `.scbundle`. The archive uses the Phase 7.2 content hash: each file except `integrity.json` is SHA-256 of the stored UTF-8 bytes, and the bundle hash is SHA-256 of the ordinal `path:hash` lines. Opening the file shows that stored review. A matching hash is a content check, not a signature.
- The review app in `rust/` is the ScalarScope being built. It opens two inference traces, or two backpropagate `run_history.json` files. Training loss stays training loss, and the eval numbers sit beside that curve. An inference pair reports ΔF and ΔO. ΔTc is reported only when both traces have a steady-state milestone, and a missing milestone is not called a stabilization time. ΔTd and ΔĀ stay off that page.
- Compare opens a real inference trace. A latency CSV, a benchmark JSON, or a profiler trace loads as the series it measured. The same samples can be drawn as a series or as their distribution. The series drawing adds a rolling spread band, 3-sigma marks, a steady-state span when that milestone exists, and throughput when both traces have it. A geometry JSON still opens the path view.
- Home keeps a local comparison log. A review is written when both runs are loaded: the two names, the deltas that fired across the full run (ΔF, ΔTc, ΔTd, ΔĀ, ΔO), and the bundle hash once a bundle is saved or opened. The file stays in app data.

### Fixed
- A 2.0 comparison bundle did not open at all: 2.0 compresses bundle entries with Deflate, and the review read stored entries only. Real 2.0 delta ids are camelCase (`failurePresence`) and are now matched in either case.
- A detected milestone is a step number everywhere. ΔO read it as a sample index, which mis-placed it when steps do not start at 0.
- One introduced anomaly reads "Introduced 1 new runtime anomaly". A 2.0 bundle with the plural sentence still opens.
- The page scrolls, so the throughput and memory plots are not cut off.
- The spread band on an inference series is drawn as one trapezoid per step. It was one long polygon, which the plot filled as a fan of wedges across the chart. The legend shows one spread entry per side, in that side's color.
- ΔTc stays quiet when both steady-state milestones are within 3 steps of each other. That is the resolution of the .NET geometry convergence delta; the .NET inference comparer fired on any nonzero difference, and so did 3.0 until this change. `docs/parity-and-beyond.spec.md` replaces the floor with an interval rule. A missing milestone still withholds ΔTc.
- A reopened bundle names its two runs above the review, instead of Path A and Path B.
- A bundle's hash is recomputed from the files in the archive. A stated hash that does not match those bytes is rejected, including a file that was left out of the hash list. The exporter writes the same UTF-8 bytes it hashes.
- Home opens a `.scbundle`. Compare imports that bundle, and Try Example loads the built-in runs. The comparison log records a bundle when it loads.
- Playback advances on the UI thread. A tick that is already queued is dropped.
- Recovery resumes to Home, Compare, Help, or Settings. Start fresh returns to Home.

### Changed
- An inference review leads with B/A at p50, p90 and p99 over the steady samples, each with a 95% interval from 1000 moving-block bootstrap resamples. The block length is the cube root of the sample count and the seed is fixed, so a review repeats exactly. A percentile without enough samples to bound it says how many it needs (p99 needs 368), and the ratios, ΔO and ΔF need 20 steady samples per side.
- Each run gets a shape (flat, warmup, slowdown, no steady state, or too short to tell) from PELT change points on log-latency, and a steady-start range across three penalties. ΔTc fires only when both runs settle and their ranges do not overlap. Steps stated in a RunTrace file keep the 2.0 rule. The 2.0 window heuristic is a cross-check, and the page says when it disagrees. This replaces the 3-step floor.
- ΔO compares relative spread, (p90 − p10) / p50, and fires when its interval excludes 1. ΔF counts samples beyond 5 robust deviations (MAD) in the steady samples and fires when B's excess is beyond chance (one-sided exact test, p < 0.05). On the 2.0 golden pair ΔF is now quiet: 2.0 counted the optimized run's first warmup sample as an anomaly. `docs/parity-and-beyond.spec.md` lists every verdict change.
- The spread band is the p10–p90 of an 11-sample window, and the marks use the MAD rule.
- The Store package declares Windows.Desktop only, tested up to Windows 11 (10.0.26100.0). Earlier packages also declared Windows.Universal, which offered a full-trust desktop app to device families that cannot run it. The pack script refuses any other family.
- Package identity version is `3.0.0.0` for the next Partner Center update of store `9P3HT1PHBKQK`. Name, publisher, and publisher display name are unchanged.
- The release workflow publishes an unsigned `ScalarScope_3.0.0.0_Store.msixupload` whose binary is the Rust review, the same upload shape as `ScalarScope_v2.0.0_Store.msixupload`. Partner Center signs it. The old workflow built the .NET package.
- Welcome and Settings read the assembly version instead of a hardcoded `2.0.0`.

## [2.0.2] - 2026-03-25

### Added
- 3 version consistency tests (VortexKit semver, ScalarScope semver, >= 1.0.0)
- Test step in CI build workflow

### Fixed
- SHA-pinned CI actions for supply-chain safety

## [2.0.1] - 2026-03-24

### Added
- Starlight handbook with landing page

## [2.0.0] - 2026-02-09

### Added - Phase H: Humanizing & UI/UX Finish

#### Gate H1: Welcome Experience
- **WelcomePage**: New landing page with first-60-seconds experience
- **Hero Section**: App icon, title, one-liner promise ("Compare inference runs with scientific rigor")
- **Trust Badge**: "Reproducible • Auditable • No telemetry"
- **Primary CTAs**: "Compare Two Runs" and "Open Review Bundle" buttons
- **Try Example Section**: Onboarding card for new users
- **Recent Comparisons**: Quick resume list with timestamp and metric summary

#### Gate H2: Design System
- **DesignSystem.xaml**: Unified visual grammar resource dictionary
- **Typography Scale**: Hero (32pt), Section (15pt), Body (14pt), Caption (11pt)
- **Spacing Rhythm**: 4/8/12/16/24/32/40px consistent spacing tokens
- **Status Chips**: Semantic colors (success/warning/error/neutral) with consistent styling
- **Card Patterns**: Subtle borders, corner radius, and hover states

#### Gate H3: Navigation Simplification
- **Reduced Tab Count**: 5 tabs → 4 (Home, Compare, Guide, Settings)
- **Merged Views**: Consolidated redundant pages for cleaner flow
- **Tab Bar Styling**: Unified colors and selection states

#### Gate H4: Settings & About Enhancement
- **Version Display**: Show version with commit hash (2.0.0+7887384)
- **Privacy Statement**: Added inline privacy explanation
- **Bundle Hash Explanation**: Help users understand cryptographic integrity
- **Quick Links**: Documentation, GitHub, Report Issue commands

#### Gate H5: Marketplace Presentation
- **STORE_LISTING.md**: Complete Microsoft Store presentation pack
- **Screenshots Guide**: 5 required screenshots with descriptions and alt text
- **Feature Bullets**: Optimized for Store discovery
- **Pre-Flight Checklist**: Submission readiness validation

### Added - Prep Phase: Publishing

#### Gate P1: Package Identity Lock
- **Version 2.0.0.0**: Synchronized across manifest and csproj
- **InformationalVersion**: Includes commit hash for traceability
- **Updated Description**: Reflects inference optimization focus

#### Gate P2: Dependencies Included
- **SelfContained=true**: Bundles full .NET 9 runtime
- **WindowsAppSDKSelfContained=true**: Bundles Windows App SDK
- **78MB Package**: Works on clean Windows 10 without prerequisites

#### Gate P3: Signing & MSIXUpload
- **Unsigned Package**: For Store upload (Store signs during ingestion)
- **SHA256 Hash**: 0292A6189C522DCEE7EA69F59A0E114ECC0747F23E3B2C5506E1DD11094FB5FE

#### Gate P4: Store Compliance
- **PRIVACY.md**: Privacy policy (local-only, no data collection)
- **Accessibility**: AutomationProperties on all interactive controls
- **Minimal Capabilities**: runFullTrust only (justified for MAUI desktop)

#### Gate P5: Dry Run
- **44/44 Tests Passing**: Fixture and determinism tests green
- **Final Artifact**: ScalarScope_v2.0.0.msixupload (78.42 MB)

### Technical
- `WelcomePage.xaml/.cs`: New landing experience with recent comparisons
- `WelcomeViewModel.cs`: Recent comparison tracking and navigation
- `DesignSystem.xaml`: Centralized styles, colors, typography
- `AppShell.xaml`: Simplified 4-tab navigation
- `SettingsPage.xaml`: Expanded About section with privacy and links
- `SettingsViewModel.cs`: OpenDocumentation, OpenGitHub, OpenIssue commands

## [1.4.0] - 2026-02-09

### Added - Phase 5: UI Polish & Release Hardening

#### Gate 5.1: Idle Calm
- **Demo State Service**: Orchestrates idle animations across components
- **Pulse Breathing**: Subtle glow animations reduced during active comparison
- **Respiration Hook**: Animation intensity responds to user activity

#### Gate 5.2: State Continuity & Transitions
- **Focus Preservation**: Single→Compare mode switch preserves scroll/focus state
- **Choreographed Navigation**: "Show me" commands animate smoothly to anchors
- **Loading Skeletons**: Unified PlotScaffold and DeltaList loading states
- **No-Flicker Policy**: LayoutDebouncer prevents rapid layout thrashing

#### Gate 5.3: Confidence ↔ Intensity
- **Confidence Tokens**: Tiered visual weight (High/Medium/Low/Negligible)
- **Glow Modulation**: Highlight intensity correlates with confidence level
- **Confidence Badges**: Color-coded badges (HIGH/MED/LOW/???) on deltas
- **Tooltip Prefixes**: "High confidence:", "Medium confidence:" in explanations
- **Accessibility Labels**: SemanticProperties and AutomationProperties

#### Gate 5.4: Shareability Polish
- **Screenshot Ready Options**: IncludeWatermark, IncludeLegend, IncludeConfidenceBadges
- **Copy Cards**: Plain-text, Markdown, and summary card formats for deltas
- **Export Branding**: Legend and watermark drawing in exports
- **Plain-Language Export**: Non-technical executive summaries for stakeholders
- **Social Card Templates**: Twitter (1200x628), LinkedIn (1200x627), Slide (1920x1080)

#### Gate 5.5: Release Hardening
- **Version Watermark**: VersionInfo service embeds version in all exports
- **CI Validation**: Enhanced pipeline with version tracking and build summaries
- **Error Boundary**: Graceful error handling with user-friendly messages
- **Performance Profiler**: Gate thresholds for frame time, delta calc, exports

### Technical
- `DemoStateService`: Singleton for coordinating idle animations
- `ConfidenceTokens`: Maps confidence values to visual weight
- `LayoutDebouncer`/`AxisRangeInterpolator`: Smooth layout transitions
- `DeltaCopyService`: Multi-format copy card generation
- `SocialCardService`: Social media-optimized image export
- `VersionInfo`: Assembly version metadata access
- `ErrorBoundary`: Safe execution wrappers with retry/timeout
- `PerformanceProfiler`: Timing metrics with percentile support

## [1.3.0] - 2026-02-08

### Added
- **Heat Map Overlay**: Optional density visualization showing where the trajectory lingers longest
  - Logarithmic scaling for better visualization of density variations
  - 7-color gradient from deep blue (sparse) to red (dense)
  - Toggle via `ShowHeatMap` property

- **Vector Field Grid**: Background arrows showing local flow directions
  - 12x12 grid with contextual velocity indicators
  - Muted styling to avoid distracting from main trajectory
  - Toggle via `ShowVectorField` property

- **Color Mode Selection**: Choose how trajectory color is determined
  - **Time mode** (default): Gradient from start (cyan) to end (red)
  - **Velocity mode**: Slow (blue) to fast (red) coloring
  - **Curvature mode**: Smooth (green) to sharp turns (orange)
  - Set via `ColorMode` property with `TrajectoryColorMode` enum

### Technical
- Added `TrajectoryColorMode` enum for trajectory coloring options
- Implemented `DrawHeatMap()` with 64x64 density grid and Gaussian-like visualization
- Implemented `DrawVectorFieldGrid()` with directional arrows
- Added color mode-aware trajectory rendering throughout glow and spline drawing

## [1.2.0] - 2026-02-08

### Added
- **Catmull-Rom Spline Smoothing**: Trajectory paths now render with smooth, organic curves instead of jagged line segments
- **Adaptive Stroke Width**: Line thickness varies inversely with velocity (faster motion = thinner lines, like calligraphy)
- **Trail Opacity Decay**: Older trajectory points fade naturally, with recent path emphasized
- **Glow Effects**: Multi-layered ambient glow around trajectory for enhanced visual depth and energy visualization
- **v2.0 Roadmap**: Added comprehensive roadmap document outlining the path to ScalarScope 2.0

### Technical
- Implemented Catmull-Rom spline interpolation with 8 subdivisions per segment for smooth curves
- Added velocity-normalized stroke width calculation (1.5px - 5px range)
- Added progressive opacity fade starting at 40% for trajectory history
- Multi-pass glow rendering with blur masks for realistic luminance

## [1.1.0] - 2026-02-08

### Added
- **Session Recovery**: Full crash recovery now restores playback position, current page, and playback state
  - Automatically reloads the file that was open when the app crashed
  - Resumes playback from exact position
  - Navigates to the page you were viewing

### Fixed
- **Zero-Warning Build**: Eliminated all 11 compiler warnings
  - CS0108: Added `new` keyword to intentionally hidden members (BackgroundColor, ScaleProperty)
  - CS0618: Migrated deprecated `Frame` to modern `Border` in DemoAnnotationBanner
  - CS1998: Removed unnecessary `async` modifier from synchronous methods
  - CS8602: Fixed potential null reference in TrajectoryCanvas

### Changed
- DemoAnnotationBanner now uses `Border` with `StrokeShape` instead of deprecated `Frame`

## [1.0.8] - 2026-02-08

### Added
- **Settings Page**: New dedicated tab for configuring application preferences
  - Theme selection (System / Light / Dark)
  - High contrast mode toggle
  - Reduce animations toggle (accessibility)
  - Annotation density (Minimal / Standard / Full)
  - Default playback speed selection
  - Auto-play on load toggle
  - Auto-load last session toggle
  - Configurable recent files limit (5 / 10 / 20)
  - Default export folder selection
  - Default export resolution
  - Reset demo / first-run state
  - Reset all settings to defaults
- **Theme Persistence**: Selected theme is saved and restored on app startup
- **Accessibility**: `SemanticProperties` added to all Settings page controls for screen readers

### Changed
- UserPreferencesService expanded with 15+ new settings methods
- Recent files limit now user-configurable (was hardcoded to 10)

## [1.0.7] - 2026-02-08

### Changed
- **MVVM AOT Compatibility**: Migrated all `[ObservableProperty]` fields to partial property syntax for WinRT/AOT compatibility
  - ComparisonViewModel (15 properties)
  - VortexSessionViewModel (13 properties)
  - TrajectoryPlayerViewModel (10 properties)
  - ExportViewModel (14 properties)
  - VortexKit PlaybackController (6 properties)
- **Frame → Border Migration**: Replaced all deprecated `Frame` XAML elements with modern `Border` controls
  - Migrated 30+ Frame elements across 4 XAML files
  - ExportPanel, RecoveryPage, OverviewPage, HelpPage
  - `BorderColor` → `Stroke`, `CornerRadius` → `StrokeShape="RoundRectangle X"`

### Technical
- Eliminates 104+ MVVM Toolkit source generator AOT warnings
- Future-proofs XAML for .NET 9+ where Frame is deprecated
- Zero functional changes, pure technical debt reduction

## [1.0.6] - 2026-02-08

### Changed
- **SkiaSharp API Modernization**: Migrated all text rendering from deprecated `SKPaint.TextSize`, `SKPaint.TextAlign`, and `SKCanvas.DrawText(string, float, float, SKPaint)` to modern `SKFont`-based APIs
  - All controls now use `SKFont` for text size and typeface
  - `DrawText` calls updated to use 6-parameter overload with `SKTextAlign` and `SKFont`
  - `MeasureText` calls updated to use `SKFont.MeasureText(string, SKPaint)`
- Reduced SkiaSharp deprecation warnings from 170+ to 0 (text API related)

### Technical
- Future-proofs codebase for SkiaSharp 4.x compatibility
- Files updated: TrajectoryCanvas, ComparisonTrajectoryCanvas, ComparisonAnalyticsPanel, FailuresTimeline, EigenSpectrumView, ScalarRingStack, AnnotationOverlay, LoadingOverlay, ExportService

## [1.0.5] - 2026-02-08

### Added
- **Drag-and-Drop**: Drag JSON files directly onto the Overview page to load training runs
- **Hover Tooltips**: Hover over trajectory points to see real-time values (time, position, velocity)
- **Zoom & Pan**: Mouse wheel zoom and right-click drag to pan on trajectory canvas
- **Loading Shimmer**: Animated loading overlay with progress messages when opening files
- **Reset View Button**: One-click reset for zoom/pan state in trajectory view

### Changed
- TrajectoryCanvas now supports touch/pointer events for interactive exploration
- GeometryRun model enhanced with computed velocity magnitude property
- Refactored session loading to expose loading state via observable properties

## [1.0.4] - 2026-02-08

### Added
- **Recent Files**: Quick access to up to 10 recently opened training runs from the Overview tab
- **Keyboard Shortcut**: Added `Ctrl+E` as an additional export shortcut (alongside existing `S` and `Ctrl+S`)
- **Fine Step Control**: `Shift+Left/Right` for 0.1% precision stepping (documented)
- **Tab Navigation**: `1-6` keyboard shortcuts for switching tabs (now displayed in UI)

### Fixed
- **CI Build**: Fixed GitHub Actions workflow targeting wrong .NET framework (`net10.0` → `net9.0`)
- **Session Recovery**: File path now correctly stored for crash recovery (was using object.ToString())

### Changed
- VortexKit library promoted from `1.0.0-rc.1` to `1.0.0` stable release
- Updated keyboard shortcuts documentation and UI display

## [1.0.1.1] - 2026-02-04

### Added
- **Phase 4: Instrument Readiness & Trust**
  - InvariantGuard service for runtime assertions (soft-fail in release, hard-fail in debug)
  - ConsistencyCheckService for centralized metric calculations
  - GoldenRunService for regression testing via golden snapshots
  - Cross-view consistency verification (eigenvalue interpretations match everywhere)

### Changed
- DemoService now uses multi-path fallback for bundled file loading (better MAUI deployment compatibility)
- Version bump to 1.0.1.1 for Microsoft Store resubmission (cannot reuse version numbers)

### Fixed
- Bundled demo files now load correctly across all MAUI deployment scenarios

## [1.0.0-rc.1] - 2025-02-04

### Added
- **Trajectory Visualization**: Real-time 2D training trajectory playback with GPU-accelerated SkiaSharp rendering
- **Side-by-Side Comparison**: Compare two training runs with synchronized playback
- **Annotation System**: Automatic detection and display of:
  - Phase transitions (dimensional shifts)
  - Curvature warnings (instability indicators)
  - Eigenvalue insights (λ₁ dominance analysis)
  - Failure markers (severity-coded)
- **Export Capabilities**:
  - Single frame PNG export (up to 4K)
  - Frame sequence export for video/GIF creation
  - Comparison exports with side-by-side layout
- **Failures Timeline**: Dedicated view for analyzing failure events with severity coding
- **Comparison Analytics Panel**: Statistical comparison with automated verdict
- **Playback Controls**:
  - Play/Pause, step forward/backward
  - Variable speed (0.25x to 4x)
  - Keyboard shortcuts (Space, Arrow keys, +/-)
- **VortexKit Library**: Reusable visualization framework extracted for future projects

### Documentation
- Demo script for 5-minute walkthrough
- Quick reference card for keyboard shortcuts
- Paper companion section for publication appendix

### Technical
- .NET 9.0 + .NET MAUI for cross-platform desktop
- SkiaSharp 3.x for high-performance 2D rendering
- CommunityToolkit.Mvvm for MVVM architecture
- Support for both light and dark themes

## [0.1.0] - 2025-01-15

### Added
- Initial project scaffold
- Basic MAUI shell structure
- Core data models for training runs
