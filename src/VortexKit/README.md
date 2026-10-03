# VortexKit

Source folder for the ScalarScope visualization helpers. It is not a NuGet package. Do not install `VortexKit` from a feed. Reference this project from the ScalarScope solution.

The public surface is the types in this folder. `ComparisonView`, `AnnotationLayer`, `AnnotationPanel`, `FailureTimeline`, `FailureSeverity`, and `VortexFonts` are not in this assembly.

Samples below assume:

```csharp
using SkiaSharp;
using VortexKit;
using VortexKit.Annotations;
using VortexKit.Core;
```

## Playback

`PlaybackController` (`VortexKit.Core`) is a shared 0..1 clock.

```csharp
var player = new PlaybackController { Duration = 10, Loop = false };
player.TimeChanged += () => view.CurrentTime = player.Time;

player.PlayPause();
player.StepForward();
player.JumpToTime(0.5);
player.SetSpeed(1.5);
player.Stop();
```

`TimeChanged` fires when `Time` changes. That includes a direct `Time` assignment and `PlayPause` restarting from the end. Setting the same value again does not fire it a second time.

`Time` stays in `[0, 1]`. `Speed` stays in `[0.1, 10]`. `SetSpeed` ends at that clamped value for the same input, whatever the previous preset index was. `Duration` must be positive. Preset steps are `IncreaseSpeed` and `DecreaseSpeed` (0.25, 0.5, 1, 2, 4).

Playback ticks are applied on the UI thread. They do not write `Time` or `IsPlaying` from the timer thread.

## Canvas

Subclass `AnimatedCanvas` (`VortexKit.Core`). `CurrentTime` in `[0, 1]` invalidates the surface.

```csharp
public sealed class MyCanvas : AnimatedCanvas
{
    protected override void OnRender(SKCanvas canvas, SKImageInfo info, double time)
    {
        canvas.Clear(VortexColors.Background);
    }
}
```

`Tapped` fires on release only when the pointer did not drag past the tap slop. `Dragged` is `(from, to)`. A cancelled or exited pointer clears the gesture, and a second pointer does not steal it.

## Series and failures

`TimeSeries<T>` implements `ITimeSeriesData<T>`:

```csharp
var series = new TimeSeries<double>(new List<double> { 0.1, 0.4, 0.9 });
double atHalf = series.GetAtTime(0.5);
```

`IFailureEvent.Severity` is a `string` (`FailureEvent` defaults it to `"info"`). There is no `FailureSeverity` enum and no failure timeline view.

## Annotations

`IAnnotation` (`VortexKit.Annotations`) uses a tuple position, not `SKPoint`.

```csharp
var note = new Annotation
{
    Label = "phase",
    TheoreticalBasis = "regime change",
    Category = AnnotationCategory.Phase,
    Time = 0.25,
    Position = (0.1, 0.2),
    Priority = 1
};
```

`AnnotationCategory` is `Phase`, `Warning`, `Insight`, `Failure`, or `Custom`. There is no annotation view in this project. Category colors are `GetCategoryColor()`.

## Export

`ExportService` takes a render callback and a path. It does not take `IRenderable`, and sequence export has no start or end time. Frames sample `t` from 0 to 1. One frame uses `t = 0`. Non-positive `Fps`, `Duration`, `Width`, or `Height` fail the sequence export instead of writing `sequence_info.txt`.

```csharp
var exporter = new ExportService();

await exporter.ExportFrameAsync(
    (canvas, info, time) => canvas.Clear(VortexColors.Background),
    player.Time,
    "frame.png",
    new ExportOptions { Width = 1920, Height = 1080 });

await exporter.ExportSequenceAsync(
    (canvas, info, time) => canvas.Clear(VortexColors.Background),
    "frames",
    new ExportOptions { Width = 1280, Height = 720, Fps = 30, Duration = 2 });

await exporter.ExportComparisonAsync(
    (canvas, info, time) => { },
    (canvas, info, time) => { },
    player.Time,
    "compare.png",
    new ComparisonExportOptions
    {
        Width = 1920,
        Height = 1080,
        ShowLabels = true,
        LeftLabel = "A",
        RightLabel = "B"
    });
```

`ExportOptions` is `Width`, `Height`, `Fps`, `Duration`, `BackgroundColor`, and `ProgressCallback`. `ComparisonExportOptions` adds `DividerWidth`, `DividerColor`, `ShowLabels`, `LeftLabel`, `RightLabel`, `LeftColor`, and `RightColor`. There is no `ShowAnnotations` property.

PNG writes replace the file. A shorter export does not leave the previous file's tail.

## SVG

`SvgExportService.ExportSvgAsync(SvgExportData, string, SvgExportOptions?)` writes the document from `BuildSvg`.

```csharp
var svg = new SvgExportService();
await svg.ExportSvgAsync(
    new SvgExportData
    {
        Trajectories = new List<SvgTrajectoryData>
        {
            new SvgTrajectoryData
            {
                Label = "run",
                Color = "#89B4FA",
                Points = new List<SvgPoint> { new(0, 0), new(1, 1) }
            }
        }
    },
    "traj.svg",
    new SvgExportOptions { Title = "Run", ColorMode = SvgColorMode.Solid });
```

`SvgColorMode.Solid` uses the trajectory color, or the palette trajectory color when that string is not `#RRGGBB` or `#RRGGBBAA`. `Time` colors each segment from `TrajectoryStart` to `TrajectoryEnd` by point index. `Velocity` and `Curvature` require `Velocities` or `Curvatures` with one value per point and color each segment from those values. The file does not reference a gradient that was not written.

`SvgColorPalette` presets are `Default`, `Light`, `HighContrast`, and `Publication`.

## Colors

`VortexColors` is a static palette of `SKColor` values: `Background`, `Surface`, `Elevated`, `Grid`, `Primary`, `Success`, `Danger`, `Warning`, `Info`, `Highlight`, `Holdout`, `Text`, `TextMuted`, `TextDisabled`, `CompareLeft`, `CompareRight`, plus severity helpers. `GetSeverityColor` takes the same severity strings as `IFailureEvent`.

## Files

```
VortexKit/
├── Annotations/
│   └── IAnnotation.cs
├── Core/
│   ├── AnimatedCanvas.cs
│   ├── ExportService.cs
│   ├── ITimeSeries.cs
│   ├── PlaybackController.cs
│   └── SvgExportService.cs
├── Theme/
│   └── VortexColors.cs
├── README.md
└── VortexKit.csproj
```

The project targets `net9.0-windows10.0.19041.0` and references `SkiaSharp.Views.Maui.Controls` and `CommunityToolkit.Mvvm`. Those are its dependencies, not a VortexKit package.

## License

MIT.
