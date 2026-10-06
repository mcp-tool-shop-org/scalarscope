namespace ScalarScope.Services;

/// <summary>
/// Preferences the shell reads at startup and when a run loads.
/// </summary>
public static class ShellStartup
{
    public static bool ShouldOpenSavedSession(bool autoLoad, string? loadedFilePath)
        => autoLoad && !string.IsNullOrWhiteSpace(loadedFilePath);

    public static int ClampColorVisionIndex(int index)
        => index >= 0 && index <= (int)ColorPaletteMode.Monochrome ? index : 0;

    public static float ClampTextScale(float scale)
        => float.IsFinite(scale) ? Math.Clamp(scale, 0.75f, 2f) : 1f;

    public static void ApplyAccessibility()
    {
        var index = ClampColorVisionIndex(UserPreferencesService.GetColorVisionMode());
        AccessibilityService.Instance.Settings = new AccessibilitySettings
        {
            ColorPaletteMode = (ColorPaletteMode)index,
            HighContrastEnabled = UserPreferencesService.GetHighContrastMode(),
            ReducedMotionEnabled = UserPreferencesService.GetReduceAnimations(),
            ScreenReaderEnabled = UserPreferencesService.GetScreenReaderMode(),
            TextScale = ClampTextScale(UserPreferencesService.GetTextScale()),
            LargePointer = UserPreferencesService.GetLargePointer()
        };
    }
}
