using ScalarScope.Services.Connectors;
using SkiaSharp;
using SkiaSharp.Views.Maui;
using SkiaSharp.Views.Maui.Controls;

namespace ScalarScope.Views.Controls;

/// <summary>
/// Two drawings of one trace review. Series is the aligned window over time.
/// Distribution is the empirical CDF of those same samples.
/// </summary>
public class TraceReviewCanvas : SKCanvasView
{
    public static readonly BindableProperty ReviewProperty =
        BindableProperty.Create(nameof(Review), typeof(TraceReview), typeof(TraceReviewCanvas),
            propertyChanged: Invalidate);

    public static readonly BindableProperty CurrentTimeProperty =
        BindableProperty.Create(nameof(CurrentTime), typeof(double), typeof(TraceReviewCanvas), 0.0,
            propertyChanged: Invalidate);

    public static readonly BindableProperty ShowDistributionProperty =
        BindableProperty.Create(nameof(ShowDistribution), typeof(bool), typeof(TraceReviewCanvas), false,
            propertyChanged: Invalidate);

    private static readonly SKColor PlotBackground = SKColor.Parse("#12121f");
    private static readonly SKColor LeftColor = SKColor.Parse("#4ecdc4");
    private static readonly SKColor RightColor = SKColor.Parse("#ff6b6b");
    private static readonly SKColor Grid = SKColor.Parse("#2a2a4e");
    private static readonly SKColor Playhead = SKColor.Parse("#ffd93d");
    private static readonly SKColor CaptionColor = SKColor.Parse("#9aa0b4");

    public TraceReviewCanvas()
    {
        AccessibleCanvasHook.Attach(this);
        PaintSurface += OnPaintSurface;
        MinimumHeightRequest = 220;
    }

    public TraceReview? Review
    {
        get => (TraceReview?)GetValue(ReviewProperty);
        set => SetValue(ReviewProperty, value);
    }

    public double CurrentTime
    {
        get => (double)GetValue(CurrentTimeProperty);
        set => SetValue(CurrentTimeProperty, value);
    }

    public bool ShowDistribution
    {
        get => (bool)GetValue(ShowDistributionProperty);
        set => SetValue(ShowDistributionProperty, value);
    }

    private static void Invalidate(BindableObject bindable, object oldValue, object newValue)
    {
        if (bindable is TraceReviewCanvas canvas)
            canvas.InvalidateSurface();
    }

    private void OnPaintSurface(object? sender, SKPaintSurfaceEventArgs e)
    {
        var canvas = e.Surface.Canvas;
        var info = e.Info;
        AccessibleCanvasHook.Paint(canvas, e.Info, this, PlotBackground);

        using var font = new SKFont { Size = Math.Clamp(info.Height / 22f, 14, 28) };
        using var paint = new SKPaint { IsAntialias = true, Color = CaptionColor };
        var review = Review;
        if (review == null || (review.LeftValues.Count == 0 && review.RightValues.Count == 0))
        {
            canvas.DrawText("This trace has no numeric samples on the shared signal.", 24, 48, font, paint);
            return;
        }

        var plot = new SKRect(28, 36, info.Width - 20, info.Height - 28);
        if (ShowDistribution)
            DrawDistribution(canvas, plot, review, CurrentTime, font, paint);
        else
            DrawSeries(canvas, plot, review, font, paint);
    }

    private void DrawSeries(SKCanvas canvas, SKRect plot, TraceReview review, SKFont font, SKPaint paint)
    {
        var count = Math.Max(review.LeftValues.Count, review.RightValues.Count);
        var finite = Finite(review.LeftValues).Concat(Finite(review.RightValues)).ToList();
        foreach (var band in review.LeftBand.Concat(review.RightBand))
        {
            if (band is TraceBandPoint point)
            {
                finite.Add(point.Low);
                finite.Add(point.High);
            }
        }

        if (finite.Count == 0 || count == 0)
            return;

        var hasThroughput = review.LeftThroughput.Count > 0 && review.RightThroughput.Count > 0;
        var seriesPlot = plot;
        SKRect throughputPlot = default;
        if (hasThroughput)
        {
            var split = plot.Top + plot.Height * 0.72f;
            seriesPlot = new SKRect(plot.Left, plot.Top, plot.Right, split - 8);
            throughputPlot = new SKRect(plot.Left, split + 4, plot.Right, plot.Bottom);
        }

        var min = finite.Min();
        var max = finite.Max();
        Pad(ref min, ref max);
        DrawSteadySpan(canvas, seriesPlot, review.LeftSteadyIndex, count, LeftColor.WithAlpha(36));
        DrawSteadySpan(canvas, seriesPlot, review.RightSteadyIndex, count, RightColor.WithAlpha(36));
        DrawFrame(canvas, seriesPlot, paint);
        DrawBand(canvas, seriesPlot, review.LeftBand, count, min, max, LeftColor.WithAlpha(48));
        DrawBand(canvas, seriesPlot, review.RightBand, count, min, max, RightColor.WithAlpha(48));
        DrawPolyline(canvas, seriesPlot, review.LeftValues, count, min, max, LeftColor);
        DrawPolyline(canvas, seriesPlot, review.RightValues, count, min, max, RightColor);
        DrawAnomalies(canvas, seriesPlot, review.LeftValues, review.LeftAnomalies, count, min, max, LeftColor);
        DrawAnomalies(canvas, seriesPlot, review.RightValues, review.RightAnomalies, count, min, max, RightColor);

        var index = IndexAt(CurrentTime, count);
        var x = XAt(seriesPlot, index, count);
        paint.Color = Playhead;
        paint.StrokeWidth = 2;
        canvas.DrawLine(x, seriesPlot.Top, x, hasThroughput ? throughputPlot.Bottom : seriesPlot.Bottom, paint);
        paint.Color = CaptionColor;
        var marks = review.LeftAnomalies.Count + review.RightAnomalies.Count;
        canvas.DrawText($"step {index + 1} of {count} · {marks} anomaly marks", seriesPlot.Left, 24, font, paint);

        if (!hasThroughput)
            return;

        var throughput = Finite(review.LeftThroughput).Concat(Finite(review.RightThroughput)).ToList();
        if (throughput.Count == 0)
            return;

        var throughputMin = throughput.Min();
        var throughputMax = throughput.Max();
        Pad(ref throughputMin, ref throughputMax);
        DrawSteadySpan(canvas, throughputPlot, review.LeftSteadyIndex, count, LeftColor.WithAlpha(28));
        DrawFrame(canvas, throughputPlot, paint);
        DrawPolyline(canvas, throughputPlot, review.LeftThroughput, count, throughputMin, throughputMax, LeftColor);
        DrawPolyline(canvas, throughputPlot, review.RightThroughput, count, throughputMin, throughputMax, RightColor);
    }

    private static void DrawDistribution(SKCanvas canvas, SKRect plot, TraceReview review, double time, SKFont font, SKPaint paint)
    {
        var values = review.LeftDistribution.Select(point => point.Value)
            .Concat(review.RightDistribution.Select(point => point.Value))
            .ToList();
        if (values.Count == 0)
            return;

        var min = values.Min();
        var max = values.Max();
        Pad(ref min, ref max);
        DrawFrame(canvas, plot, paint);
        DrawCdf(canvas, plot, review.LeftDistribution, min, max, LeftColor);
        DrawCdf(canvas, plot, review.RightDistribution, min, max, RightColor);
        DrawPercentileTick(canvas, plot, review.LeftP99, min, max, LeftColor);
        DrawPercentileTick(canvas, plot, review.RightP99, min, max, RightColor);
        paint.Color = CaptionColor;
        canvas.DrawText("empirical CDF", plot.Left, 24, font, paint);
        MarkPlayhead(canvas, plot, review.LeftValues, review.LeftDistribution, time, min, max, LeftColor);
        MarkPlayhead(canvas, plot, review.RightValues, review.RightDistribution, time, min, max, RightColor);
    }

    private static void MarkPlayhead(
        SKCanvas canvas,
        SKRect plot,
        IReadOnlyList<double?> series,
        IReadOnlyList<TraceDistributionPoint> cdf,
        double time,
        double min,
        double max,
        SKColor color)
    {
        var index = IndexAt(time, series.Count);
        if (index < 0 || index >= series.Count || series[index] is not double value || cdf.Count == 0)
            return;

        var rank = cdf.Count(point => point.Value <= value) / (double)cdf.Count;
        var x = MapRange(value, min, max, plot.Left, plot.Right);
        var y = plot.Bottom - (float)rank * plot.Height;
        using var paint = new SKPaint { IsAntialias = true, Color = color, Style = SKPaintStyle.Fill };
        canvas.DrawCircle(x, y, 6, paint);
    }

    private static void DrawSteadySpan(SKCanvas canvas, SKRect plot, int? start, int count, SKColor color)
    {
        if (start is not int index || count <= 0 || index >= count)
            return;

        var left = XAt(plot, index, count);
        using var paint = new SKPaint { Color = color, Style = SKPaintStyle.Fill, IsAntialias = true };
        canvas.DrawRect(new SKRect(left, plot.Top, plot.Right, plot.Bottom), paint);
    }

    private static void DrawBand(
        SKCanvas canvas,
        SKRect plot,
        IReadOnlyList<TraceBandPoint?> band,
        int count,
        double min,
        double max,
        SKColor color)
    {
        using var paint = new SKPaint { Color = color, Style = SKPaintStyle.Fill, IsAntialias = true };
        using var path = new SKPath();
        var run = new List<(float X, float Low, float High)>();
        void Flush()
        {
            if (run.Count == 0)
                return;
            path.MoveTo(run[0].X, run[0].High);
            foreach (var point in run)
                path.LineTo(point.X, point.High);
            for (var index = run.Count - 1; index >= 0; index--)
                path.LineTo(run[index].X, run[index].Low);
            path.Close();
            run.Clear();
        }

        for (var index = 0; index < band.Count; index++)
        {
            if (band[index] is not TraceBandPoint point)
            {
                Flush();
                continue;
            }

            var x = XAt(plot, index, count);
            var high = plot.Bottom - (float)((point.High - min) / (max - min)) * plot.Height;
            var low = plot.Bottom - (float)((point.Low - min) / (max - min)) * plot.Height;
            run.Add((x, low, high));
        }

        Flush();
        canvas.DrawPath(path, paint);
    }

    private static void DrawAnomalies(
        SKCanvas canvas,
        SKRect plot,
        IReadOnlyList<double?> values,
        IReadOnlyList<int> indices,
        int count,
        double min,
        double max,
        SKColor color)
    {
        using var fill = new SKPaint { Color = Playhead, Style = SKPaintStyle.Fill, IsAntialias = true };
        using var ring = new SKPaint { Color = color, Style = SKPaintStyle.Stroke, StrokeWidth = 2, IsAntialias = true };
        foreach (var index in indices)
        {
            if (index < 0 || index >= values.Count || values[index] is not double value || !double.IsFinite(value))
                continue;
            var x = XAt(plot, index, count);
            var y = plot.Bottom - (float)((value - min) / (max - min)) * plot.Height;
            canvas.DrawCircle(x, y, 6, fill);
            canvas.DrawCircle(x, y, 6, ring);
        }
    }

    private static void DrawPercentileTick(SKCanvas canvas, SKRect plot, double? percentile, double min, double max, SKColor color)
    {
        if (percentile is not double value || !double.IsFinite(value))
            return;

        var x = MapRange(value, min, max, plot.Left, plot.Right);
        using var paint = new SKPaint { Color = color, StrokeWidth = 2, IsAntialias = true };
        canvas.DrawLine(x, plot.Bottom, x, plot.Bottom - 14, paint);
    }

    private static float XAt(SKRect plot, int index, int count)
    {
        return plot.Left + (count <= 1 ? plot.Width / 2f : (float)index / (count - 1) * plot.Width);
    }

    private static void DrawPolyline(
        SKCanvas canvas,
        SKRect plot,
        IReadOnlyList<double?> values,
        int count,
        double min,
        double max,
        SKColor color)
    {
        using var paint = new SKPaint
        {
            IsAntialias = true,
            Color = color,
            Style = SKPaintStyle.Stroke,
            StrokeWidth = 3,
            StrokeCap = SKStrokeCap.Round
        };
        using var path = new SKPath();
        var penDown = false;
        for (var index = 0; index < values.Count; index++)
        {
            if (values[index] is not double value || !double.IsFinite(value))
            {
                penDown = false;
                continue;
            }

            var x = plot.Left + (count == 1 ? plot.Width / 2f : (float)index / (count - 1) * plot.Width);
            var y = plot.Bottom - (float)((value - min) / (max - min)) * plot.Height;
            if (!penDown)
            {
                path.MoveTo(x, y);
                penDown = true;
            }
            else
            {
                path.LineTo(x, y);
            }
        }

        canvas.DrawPath(path, paint);
    }

    private static void DrawCdf(
        SKCanvas canvas,
        SKRect plot,
        IReadOnlyList<TraceDistributionPoint> points,
        double min,
        double max,
        SKColor color)
    {
        if (points.Count == 0)
            return;

        using var paint = new SKPaint
        {
            IsAntialias = true,
            Color = color,
            Style = SKPaintStyle.Stroke,
            StrokeWidth = 3
        };
        using var path = new SKPath();
        var first = points[0];
        path.MoveTo(MapRange(first.Value, min, max, plot.Left, plot.Right), plot.Bottom);
        foreach (var point in points)
        {
            var x = MapRange(point.Value, min, max, plot.Left, plot.Right);
            var y = plot.Bottom - (float)point.Probability * plot.Height;
            path.LineTo(x, y);
        }

        canvas.DrawPath(path, paint);
    }

    private static void DrawFrame(SKCanvas canvas, SKRect plot, SKPaint paint)
    {
        paint.Color = Grid;
        paint.Style = SKPaintStyle.Stroke;
        paint.StrokeWidth = 1;
        canvas.DrawRect(plot, paint);
        paint.Style = SKPaintStyle.Fill;
    }

    private static int IndexAt(double time, int count)
    {
        if (count <= 1)
            return 0;
        var clamped = Math.Clamp(time, 0, 1);
        return (int)Math.Round(clamped * (count - 1));
    }

    private static float MapRange(double value, double min, double max, float left, float right)
    {
        return left + (float)((value - min) / (max - min)) * (right - left);
    }

    private static void Pad(ref double min, ref double max)
    {
        if (Math.Abs(max - min) < 1e-9)
        {
            min -= 1;
            max += 1;
        }
    }

    private static IEnumerable<double> Finite(IReadOnlyList<double?> values)
    {
        foreach (var value in values)
        {
            if (value is double number && double.IsFinite(number))
                yield return number;
        }
    }
}
