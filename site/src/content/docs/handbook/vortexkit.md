---
title: VortexKit
description: The standalone visualization framework for .NET MAUI apps.
sidebar:
  order: 4
---

VortexKit is the visualization engine in `src/VortexKit`. It ships with this repo. It is not a NuGet package. It builds time-synced animated canvases, comparison views, and export pipelines.

## Components

| Component | What It Does |
|-----------|-------------|
| `PlaybackController` | Shared 0-1 timeline with play/pause/step/loop, speed presets (0.25x-4x), ~60 fps tick |
| `AnimatedCanvas` | Abstract `SKCanvasView` base with time-synced invalidation, grid drawing, touch/drag events, coordinate helpers |
| `ITimeSeries<T>` / `TimeSeries<T>` | Generic time-series with index-to-time mapping and trail enumeration |
| `ExportService` | Single-frame PNG, frame sequences (with ffmpeg hints), side-by-side comparison export |
| `SvgExportService` | Full-vector SVG with Inkscape layers, Catmull-Rom splines, heatmaps, vector fields, four color palettes |
| `IAnnotation` | Typed annotations (Phase, Warning, Insight, Failure, Custom) with theoretical basis and priority |
| `VortexColors` | Semantic color palette — background layers, accent semantics, severity coding, eigenvalue palette, lerp/gradient helpers |

## Shared playback

The `PlaybackController` drives all canvases from a single timeline. Bind multiple views so they stay in sync:

```csharp
using VortexKit.Core;

var player = new PlaybackController { Duration = 10.0, Loop = true };

// Bind multiple canvases to the same timeline
player.TimeChanged += () =>
{
    trajectoryCanvas.CurrentTime = player.Time;
    eigenCanvas.CurrentTime      = player.Time;
    scalarsCanvas.CurrentTime    = player.Time;
};
```

Speed presets range from 0.25x to 4x. The controller exposes `IncreaseSpeed()` and `DecreaseSpeed()` methods and a `SpeedDisplay` property for UI binding.

## Custom canvas

Subclass `AnimatedCanvas` and override `OnRender` to draw with SkiaSharp at the current timeline position:

```csharp
public class MyTrajectoryCanvas : AnimatedCanvas
{
    protected override void OnRender(
        SKCanvas canvas, SKImageInfo info, double time)
    {
        // Your SkiaSharp rendering at the current timeline position
    }
}
```

`AnimatedCanvas` provides built-in support for grid drawing, touch and drag event handling, and coordinate system helpers for mapping between data space and pixel space.

## Export

### Side-by-side comparison PNG

```csharp
var exporter = new ExportService();
await exporter.ExportComparisonAsync(
    leftRender, rightRender, time: 0.5,
    outputPath: "comparison.png",
    new ComparisonExportOptions
    {
        Width = 1920, Height = 1080,
        LeftLabel = "Baseline", RightLabel = "Optimized",
        ShowLabels = true
    });
```

### Layered SVG (Inkscape-compatible)

```csharp
var svgExporter = new SvgExportService();
await svgExporter.ExportSvgAsync(svgData, "trajectory.svg",
    new SvgExportOptions
    {
        Palette = SvgColorPalette.Publication,
        UseCatmullRomSplines = true,
        EnableGlow = false
    });
```

SVG export supports four color palettes: Default, Light, HighContrast, and Publication. Each palette is designed for a specific output context — use Publication for papers and reports where muted colors reproduce well in print.

## Color system

`VortexColors` provides a semantic color palette with:

- **Background layers** — layered depth for card/panel nesting
- **Accent semantics** — consistent meaning across the UI (e.g., primary, secondary, warning)
- **Severity coding** — colors for success, warning, error, and neutral states
- **Eigenvalue palette** — distinct colors for spectral visualization
- **Lerp and gradient helpers** — interpolation utilities for heatmaps and continuous color mapping
