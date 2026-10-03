namespace ScalarScope.Views;

/// <summary>
/// The four shell tabs. Recovery and insight navigation must not invent others.
/// </summary>
public static class ShellDestinations
{
    public const string Welcome = "//welcome";
    public const string Compare = "//compare";
    public const string Help = "//help";
    public const string Settings = "//settings";

    public static bool IsTab(string? route)
    {
        return Name(route) is "welcome" or "compare" or "help" or "settings";
    }

    /// <summary>
    /// A saved shell location may be a bare word or a URI such as //compare?bundle=...
    /// An unknown page with a file still on disk opens Compare. Otherwise Home.
    /// </summary>
    public static string Resume(string? savedLocation, bool fileStillOnDisk)
    {
        return Name(savedLocation) switch
        {
            "welcome" => Welcome,
            "compare" => Compare,
            "help" => Help,
            "settings" => Settings,
            _ => fileStillOnDisk ? Compare : Welcome
        };
    }

    public static string? Name(string? location)
    {
        if (string.IsNullOrWhiteSpace(location))
            return null;

        var text = location.Trim();
        var cut = text.IndexOfAny(['?', '#']);
        if (cut >= 0)
            text = text[..cut];

        text = text.Trim().Trim('/');
        var slash = text.LastIndexOf('/');
        if (slash >= 0)
            text = text[(slash + 1)..];

        text = text.Trim().ToLowerInvariant();
        return text.Length == 0 ? null : text;
    }
}
