---
title: Getting Started
description: Install ScalarScope and compare your first two runs.
sidebar:
  order: 1
---

## Install from the Microsoft Store

The easiest way to get ScalarScope:

1. Visit the [Microsoft Store listing](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (Store ID: `9P3HT1PHBKQK`)
2. Click Install
3. Requires Windows 10 (build 17763) or later

## Your first comparison

1. Open ScalarScope — the **Home** tab shows your workspace status
2. Click **Compare Two Runs** to open the Compare tab
3. Load your baseline TFRT trace (before optimization)
4. Load your optimized TFRT trace (after optimization)
5. Review deltas in the **Compare** tab — only statistically meaningful differences appear
6. Click **Export Bundle** to save a `.scbundle` for reproducible sharing

### Try the built-in example

If you do not have traces yet, click **Try Example** on the Home tab. This loads a built-in demo comparison so you can explore the interface without your own data. The demo walks through the delta analysis workflow with pre-loaded traces.

## Navigation

ScalarScope uses four tabs:

| Tab | Purpose |
|-----|---------|
| **Home** | Workspace status, recent comparisons, quick actions |
| **Compare** | Side-by-side delta comparison — the core workflow |
| **Guide** | Interpretation help and onboarding |
| **Settings** | Theme, playback, export, and accessibility preferences |

Switch tabs by clicking the tab bar or pressing `1`–`4` on your keyboard.

## Build from source

```bash
# Prerequisites:
#   .NET 9.0 SDK (global.json pins 9.0.100)
#   dotnet workload install maui-windows

git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet restore
dotnet build

# Run the desktop app
dotnet run --project src/ScalarScope
```

## VortexKit

VortexKit is the project at `src/VortexKit`. It is not published to NuGet.

See the [VortexKit page](/scalarscope/handbook/vortexkit/) for the types.

## Running tests

```bash
# All tests
dotnet test

# Fixture smoke tests only
dotnet test --filter Category=FixtureSmoke

# Determinism tests (verifies reproducible deltas)
dotnet test --filter Category=Determinism

# With coverage
dotnet test --collect:"XPlat Code Coverage"
```

## Settings overview

Open the **Settings** tab to configure:

- **Theme**: System, Light, or Dark
- **Playback**: default speed (0.25x to 4x), auto-play on load
- **Export**: default resolution (width/height), export folder
- **Accessibility**: high contrast mode, color vision modes (deuteranopia, protanopia, tritanopia, monochrome), screen reader support, text scale (75% to 200%), large pointer
- **Session**: auto-load last session, recent files limit

All preferences are stored locally and persist between sessions.
