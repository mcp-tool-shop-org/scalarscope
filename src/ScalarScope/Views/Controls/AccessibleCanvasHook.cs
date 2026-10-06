using SkiaSharp;
using SkiaSharp.Views.Maui.Controls;
using ScalarScope.Services;

namespace ScalarScope.Views.Controls;

/// <summary>
/// Paints a canvas with the accessibility record the shell already stored.
/// </summary>
public static class AccessibleCanvasHook
{
    public static void Attach(SKCanvasView view)
    {
        AccessibilityService.Instance.SettingsChanged += () =>
        {
            try
            {
                MainThread.BeginInvokeOnMainThread(view.InvalidateSurface);
            }
            catch (InvalidOperationException)
            {
                view.InvalidateSurface();
            }
        };
    }

    public static void Paint(SKCanvas canvas, SKImageInfo info, SKCanvasView view, SKColor fallback)
    {
        var service = AccessibilityService.Instance;
        var settings = service.Settings;
        var tuned = settings.ColorPaletteMode != ColorPaletteMode.Default || settings.HighContrastEnabled;
        var background = tuned
            ? service.TransformColor(service.CurrentPalette.Background, ColorRole.Background)
            : fallback;
        canvas.Clear(background);

        var scale = ShellStartup.ClampTextScale(settings.TextScale);
        var pointer = settings.LargePointer ? 1.8f : 1f;
        if (view is VortexKit.Core.AnimatedCanvas playback)
        {
            playback.PublishAccess();
            if (service.ScreenReaderMode)
            {
                var playhead = SemanticProperties.GetDescription(view);
                SemanticProperties.SetDescription(
                    view,
                    playhead + $" Text scale {scale:P0}. Pointer scale {pointer:0.0}. Palette {service.CurrentPalette.Name}.");
            }
        }
        else
        {
            SemanticProperties.SetDescription(
                view,
                service.ScreenReaderMode
                    ? $"Canvas. Text scale {scale:P0}. Pointer scale {pointer:0.0}. Palette {service.CurrentPalette.Name}."
                    : null);
        }
        _ = info;
    }

    public static float ScaledFont(float size)
        => size * ShellStartup.ClampTextScale(AccessibilityService.Instance.Settings.TextScale);

    public static float PointerSize(float size)
        => AccessibilityService.Instance.Settings.LargePointer ? size * 1.8f : size;

    public static SKColor Ink(SKColor fallback)
        => AccessibilityService.Instance.TransformColor(fallback, ColorRole.Text);
}
