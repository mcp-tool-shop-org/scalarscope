namespace ScalarScope.Services;

/// <summary>
/// Export folder and size the Settings page stored.
/// </summary>
public static class ExportPreferences
{
    public static string FolderOr(string fallback)
    {
        var saved = UserPreferencesService.GetDefaultExportPath();
        return string.IsNullOrWhiteSpace(saved) ? fallback : saved;
    }

    public static (int Width, int Height) SizeOr(int fallbackWidth, int fallbackHeight)
    {
        var (width, height) = UserPreferencesService.GetDefaultExportResolution();
        if (width <= 0 || height <= 0)
            return (fallbackWidth, fallbackHeight);
        return (width, height);
    }
}
