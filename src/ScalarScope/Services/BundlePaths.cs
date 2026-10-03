namespace ScalarScope.Services;

/// <summary>
/// Path checks for bundle entries. A matching hash is a content check;
/// these names are rejected before anything is hashed or extracted.
/// </summary>
public static class BundlePaths
{
    public static bool EscapesBundleRoot(string? path)
    {
        if (string.IsNullOrWhiteSpace(path))
            return true;

        var normalized = path.Replace('\\', '/');
        if (normalized.StartsWith('/') || normalized.Contains(':'))
            return true;

        var segments = normalized.Split('/');
        return segments.Any(segment => segment == "..");
    }

    /// <summary>
    /// Asset names must be one relative segment under assets/.
    /// Dotdot, a drive prefix, a leading slash, or a directory separator is rejected.
    /// </summary>
    public static bool TryAssetEntry(string? fileName, out string entryPath, out string error)
    {
        entryPath = "";
        if (string.IsNullOrWhiteSpace(fileName))
        {
            error = "Asset name is empty.";
            return false;
        }

        var name = fileName.Trim();
        if (name.Contains("..", StringComparison.Ordinal)
            || name.Contains(':')
            || name.StartsWith('/')
            || name.StartsWith('\\')
            || name.Contains('/')
            || name.Contains('\\'))
        {
            error = "Asset name must be a single path segment under assets/ and must not escape the archive.";
            return false;
        }

        entryPath = "assets/" + name;
        error = "";
        return true;
    }
}
