namespace ScalarScope.Services;

/// <summary>
/// Help-guide matching. Each whitespace-separated token must appear in the
/// bag. The bag is stored as written, so a symbol such as Δ is not folded.
/// </summary>
public static class GuideSearch
{
    public static bool Matches(string bag, string query)
    {
        if (string.IsNullOrEmpty(query))
            return true;
        if (string.IsNullOrEmpty(bag))
            return false;

        var tokens = query.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
        foreach (var token in tokens)
        {
            if (!bag.Contains(token, StringComparison.Ordinal))
                return false;
        }

        return true;
    }
}
