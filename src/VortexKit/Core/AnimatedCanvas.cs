using SkiaSharp;
using SkiaSharp.Views.Maui;
using SkiaSharp.Views.Maui.Controls;

namespace VortexKit.Core;

/// <summary>
/// Base class for time-synchronized SkiaSharp canvases.
/// Provides automatic invalidation on time change and standard interaction patterns.
/// </summary>
public abstract class AnimatedCanvas : SKCanvasView
{
    public static readonly BindableProperty CurrentTimeProperty =
        BindableProperty.Create(
            nameof(CurrentTime),
            typeof(double),
            typeof(AnimatedCanvas),
            0.0,
            propertyChanged: OnTimeChanged);

    public static readonly BindableProperty ShowGridProperty =
        BindableProperty.Create(
            nameof(ShowGrid),
            typeof(bool),
            typeof(AnimatedCanvas),
            true,
            propertyChanged: OnPropertyChangedInvalidate);

    public static readonly BindableProperty AnimationEnabledProperty =
        BindableProperty.Create(
            nameof(AnimationEnabled),
            typeof(bool),
            typeof(AnimatedCanvas),
            true);

    /// <summary>
    /// Current time position (0.0 to 1.0). Changes trigger re-render.
    /// </summary>
    public double CurrentTime
    {
        get => (double)GetValue(CurrentTimeProperty);
        set => SetValue(CurrentTimeProperty, value);
    }

    /// <summary>
    /// Whether to show background grid lines.
    /// </summary>
    public bool ShowGrid
    {
        get => (bool)GetValue(ShowGridProperty);
        set => SetValue(ShowGridProperty, value);
    }

    /// <summary>
    /// Whether smooth animations are enabled.
    /// </summary>
    public bool AnimationEnabled
    {
        get => (bool)GetValue(AnimationEnabledProperty);
        set => SetValue(AnimationEnabledProperty, value);
    }

    /// <summary>
    /// Fired when user taps on the canvas.
    /// </summary>
    public event Action<SKPoint>? Tapped;

    /// <summary>
    /// Fired when user drags on the canvas.
    /// </summary>
    public event Action<SKPoint, SKPoint>? Dragged;

    private const float TapSlop = 10f;

    private static AnimatedCanvas? _focused;
    private readonly List<string> _labels = new();
    private Microsoft.UI.Xaml.FrameworkElement? _native;

    private SKPoint? _pressPoint;
    private bool _dragged;
    private long? _activePointerId;

    /// <summary>Text a screen reader gets for this canvas. The playhead is always in it.</summary>
    public string AccessibleText { get; private set; } = "";

    protected SKPoint? LastTouchPoint { get; private set; }

    protected AnimatedCanvas()
    {
        PaintSurface += OnPaintSurface;
        EnableTouchEvents = true;
        Touch += OnTouch;
        Focused += (_, _) => _focused = this;
        Unfocused += (_, _) =>
        {
            if (_focused == this)
                _focused = null;
        };
        PublishAccess();
    }

    /// <summary>
    /// Left and Right scrub the playhead. Enter fires the same tap the pointer fires.
    /// Returns false when no canvas is focused, so the window hook can keep the player keys.
    /// </summary>
    public static bool TryHandleFocusedKey(string key)
    {
        var canvas = _focused;
        return canvas != null && canvas.HandleAccessKey(key);
    }

    /// <summary>Keyboard path for scrub and tap. Arrows move CurrentTime. Enter raises Tapped.</summary>
    public bool HandleAccessKey(string key)
    {
        if (!CanvasAccess.TryApplyKey(key, CurrentTime, out var next, out var tap))
            return false;

        if (tap)
        {
            var x = Width > 0 ? (float)Width / 2f : 0f;
            var y = Height > 0 ? (float)Height / 2f : 0f;
            Tapped?.Invoke(LastTouchPoint ?? new SKPoint(x, y));
            return true;
        }

        CurrentTime = next;
        return true;
    }

    protected override void OnHandlerChanged()
    {
        base.OnHandlerChanged();
        if (_native != null)
        {
            _native.GotFocus -= OnNativeFocus;
            _native.LostFocus -= OnNativeBlur;
            _native = null;
        }

        if (Handler?.PlatformView is Microsoft.UI.Xaml.FrameworkElement element)
        {
            element.IsTabStop = true;
            element.UseSystemFocusVisuals = true;
            element.GotFocus += OnNativeFocus;
            element.LostFocus += OnNativeBlur;
            _native = element;
        }
    }

    private void OnNativeFocus(object sender, Microsoft.UI.Xaml.RoutedEventArgs e) => _focused = this;

    private void OnNativeBlur(object sender, Microsoft.UI.Xaml.RoutedEventArgs e)
    {
        if (_focused == this)
            _focused = null;
    }

    private static void OnTimeChanged(BindableObject bindable, object oldValue, object newValue)
    {
        if (bindable is AnimatedCanvas canvas)
        {
            canvas.OnTimeUpdated((double)oldValue, (double)newValue);
            canvas.PublishAccess();
            canvas.InvalidateSurface();
        }
    }

    protected static void OnPropertyChangedInvalidate(BindableObject bindable, object oldValue, object newValue)
    {
        if (bindable is AnimatedCanvas canvas)
            canvas.InvalidateSurface();
    }

    /// <summary>
    /// Called when time changes. Override for custom time-change handling.
    /// </summary>
    protected virtual void OnTimeUpdated(double oldTime, double newTime) { }

    private void OnPaintSurface(object? sender, SKPaintSurfaceEventArgs e)
    {
        var canvas = e.Surface.Canvas;
        var info = e.Info;

        // Clear with background
        canvas.Clear(GetBackgroundColor());
        _labels.Clear();

        // Optional grid
        if (ShowGrid)
            DrawGrid(canvas, info);

        // Subclass rendering
        OnRender(canvas, info, CurrentTime);

        // Optional overlay (for annotations, etc.)
        OnRenderOverlay(canvas, info, CurrentTime);
        PublishAccess();
    }

    /// <summary>
    /// Main rendering method. Override to implement custom visualization.
    /// </summary>
    protected abstract void OnRender(SKCanvas canvas, SKImageInfo info, double time);

    /// <summary>
    /// Optional overlay rendering (annotations, labels, etc.).
    /// </summary>
    protected virtual void OnRenderOverlay(SKCanvas canvas, SKImageInfo info, double time) { }

    /// <summary>
    /// Background color for the canvas.
    /// </summary>
    protected virtual SKColor GetBackgroundColor() => VortexColors.Background;

    /// <summary>
    /// Draw background grid.
    /// </summary>
    protected virtual void DrawGrid(SKCanvas canvas, SKImageInfo info)
    {
        var center = new SKPoint(info.Width / 2f, info.Height / 2f);

        // Declared before the paint so the paint releases the effect, then the dash is freed.
        using var dash = SKPathEffect.CreateDash([5, 5], 0);
        using var paint = new SKPaint
        {
            Color = VortexColors.Grid,
            StrokeWidth = 1,
            IsAntialias = true
        };

        // Center axes stay solid. The dash is only the grid.
        canvas.DrawLine(0, center.Y, info.Width, center.Y, paint);
        canvas.DrawLine(center.X, 0, center.X, info.Height, paint);

        paint.PathEffect = dash;
        var gridSpacing = Math.Min(info.Width, info.Height) / 8f;

        for (int i = 1; i <= 4; i++)
        {
            var offset = i * gridSpacing;

            // Horizontal
            canvas.DrawLine(0, center.Y + offset, info.Width, center.Y + offset, paint);
            canvas.DrawLine(0, center.Y - offset, info.Width, center.Y - offset, paint);

            // Vertical
            canvas.DrawLine(center.X + offset, 0, center.X + offset, info.Height, paint);
            canvas.DrawLine(center.X - offset, 0, center.X - offset, info.Height, paint);
        }
    }

    private void OnTouch(object? sender, SKTouchEventArgs e)
    {
        switch (e.ActionType)
        {
            case SKTouchAction.Pressed:
                if (_activePointerId.HasValue && e.Id != _activePointerId.Value)
                    break;

                _activePointerId = e.Id;
                _pressPoint = e.Location;
                LastTouchPoint = e.Location;
                _dragged = false;
                e.Handled = true;
                break;

            case SKTouchAction.Moved:
                if (!IsActivePointer(e) || !_pressPoint.HasValue)
                    break;

                if (!_dragged && SKPoint.Distance(_pressPoint.Value, e.Location) >= TapSlop)
                    _dragged = true;

                if (_dragged && LastTouchPoint.HasValue)
                    Dragged?.Invoke(LastTouchPoint.Value, e.Location);

                LastTouchPoint = e.Location;
                e.Handled = true;
                break;

            case SKTouchAction.Released:
                if (!IsActivePointer(e))
                    break;

                if (_pressPoint.HasValue && !_dragged &&
                    SKPoint.Distance(_pressPoint.Value, e.Location) < TapSlop)
                {
                    Tapped?.Invoke(e.Location);
                    OnTapped(e.Location);
                }

                ClearGesture();
                e.Handled = true;
                break;

            case SKTouchAction.Cancelled:
            case SKTouchAction.Exited:
                if (_activePointerId.HasValue && e.Id != _activePointerId.Value)
                    break;

                ClearGesture();
                e.Handled = true;
                break;
        }
    }

    private bool IsActivePointer(SKTouchEventArgs e) =>
        _activePointerId.HasValue && e.Id == _activePointerId.Value;

    private void ClearGesture()
    {
        _pressPoint = null;
        LastTouchPoint = null;
        _dragged = false;
        _activePointerId = null;
    }

    /// <summary>
    /// Called when user taps on the canvas. Override for custom tap handling.
    /// </summary>
    protected virtual void OnTapped(SKPoint location) { }

    #region Helper Methods

    /// <summary>
    /// Convert data coordinates to screen coordinates.
    /// </summary>
    protected SKPoint ToScreen(double x, double y, SKPoint center, float scale)
    {
        return new SKPoint(
            center.X + (float)x * scale,
            center.Y - (float)y * scale
        );
    }

    /// <summary>
    /// Convert screen coordinates to data coordinates.
    /// </summary>
    protected (double x, double y) FromScreen(SKPoint screen, SKPoint center, float scale)
    {
        return (
            (screen.X - center.X) / scale,
            (center.Y - screen.Y) / scale
        );
    }

    /// <summary>
    /// Draw text with background for readability.
    /// </summary>
    protected void DrawLabelWithBackground(
        SKCanvas canvas,
        string text,
        float x,
        float y,
        SKPaint textPaint,
        SKColor? backgroundColor = null)
    {
        RememberLabel(text);
        var bgColor = backgroundColor ?? VortexColors.Surface.WithAlpha(200);
        var textWidth = textPaint.MeasureText(text);
        var padding = 4f;

        using var bgPaint = new SKPaint { Color = bgColor, Style = SKPaintStyle.Fill };
        var rect = new SKRect(
            x - padding,
            y - textPaint.TextSize - padding,
            x + textWidth + padding,
            y + padding
        );

        canvas.DrawRoundRect(rect, 3, 3, bgPaint);
        canvas.DrawText(text, x, y, textPaint);
    }

    /// <summary>Redraws the accessible value from CurrentTime and the labels drawn this frame.</summary>
    public void PublishAccess()
    {
        AccessibleText = CanvasAccess.Describe(CurrentTime, _labels);
        AutomationProperties.SetName(this, "Playback canvas");
        SemanticProperties.SetDescription(this, AccessibleText);
    }

    private void RememberLabel(string text)
    {
        if (string.IsNullOrWhiteSpace(text) || _labels.Contains(text))
            return;
        _labels.Add(text.Trim());
        PublishAccess();
    }

    /// <summary>
    /// Draw an arrow from one point to another.
    /// </summary>
    protected void DrawArrow(SKCanvas canvas, SKPoint from, SKPoint to, SKPaint paint, float headSize = 10f)
    {
        canvas.DrawLine(from, to, paint);

        var angle = MathF.Atan2(to.Y - from.Y, to.X - from.X);
        var headAngle = 0.5f;

        var p1 = new SKPoint(
            to.X - headSize * MathF.Cos(angle - headAngle),
            to.Y - headSize * MathF.Sin(angle - headAngle));
        var p2 = new SKPoint(
            to.X - headSize * MathF.Cos(angle + headAngle),
            to.Y - headSize * MathF.Sin(angle + headAngle));

        canvas.DrawLine(to, p1, paint);
        canvas.DrawLine(to, p2, paint);
    }

    #endregion
}

/// <summary>
/// Accessible name and keyboard step for a playback canvas. No view is required.
/// </summary>
public static class CanvasAccess
{
    public const double Step = 0.01;

    public static string Describe(double time, IReadOnlyList<string>? labels)
    {
        var clamped = double.IsFinite(time) ? Math.Clamp(time, 0, 1) : 0;
        var percent = (int)Math.Round(clamped * 100, MidpointRounding.AwayFromZero);
        var text = $"Playback canvas. Playhead {percent}%.";
        if (labels == null || labels.Count == 0)
            return text;

        var unique = new List<string>();
        foreach (var label in labels)
        {
            if (string.IsNullOrWhiteSpace(label))
                continue;
            var trimmed = label.Trim();
            if (unique.Contains(trimmed))
                continue;
            unique.Add(trimmed);
            if (unique.Count == 8)
                break;
        }

        return unique.Count == 0 ? text : text + " Labels: " + string.Join(", ", unique) + ".";
    }

    /// <summary>
    /// Left and Right move the playhead by one step. Enter is a tap and leaves the time unchanged.
    /// </summary>
    public static bool TryApplyKey(string? key, double time, out double nextTime, out bool tap)
    {
        nextTime = double.IsFinite(time) ? time : 0;
        tap = false;
        switch (key)
        {
            case "Left":
                nextTime = Math.Clamp(nextTime - Step, 0, 1);
                return true;
            case "Right":
                nextTime = Math.Clamp(nextTime + Step, 0, 1);
                return true;
            case "Enter":
                tap = true;
                return true;
            default:
                return false;
        }
    }
}
