namespace ScalarScope.Services;

/// <summary>
/// Resume keeps the session file until a load succeeds.
/// </summary>
public static class RecoveryResume
{
    public static bool ShouldClearSession(bool loadSucceeded) => loadSucceeded;

    public static string Alert(bool loadSucceeded, bool fileWasPresent, string? error, string fallbackTab)
    {
        if (loadSucceeded)
            return "";
        if (!fileWasPresent)
            return $"The session file is missing. Opening {fallbackTab}.";
        return $"Could not resume: {error ?? "the file did not load"}. The session file was kept. Opening {fallbackTab}.";
    }
}
