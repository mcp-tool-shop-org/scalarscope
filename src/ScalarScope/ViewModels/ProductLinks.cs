namespace ScalarScope.ViewModels;

/// <summary>
/// Public repository links shared by Help and Settings.
/// </summary>
public static class ProductLinks
{
    public const string Repository = "https://github.com/mcp-tool-shop-org/scalarscope";
    public const string Documentation = "https://github.com/mcp-tool-shop-org/scalarscope/blob/main/README.md";
    public const string NewIssue = "https://github.com/mcp-tool-shop-org/scalarscope/issues/new/choose";

    public static Task OpenDocumentation() => Open(Documentation);

    public static Task OpenRepository() => Open(Repository);

    public static Task OpenNewIssue() => Open(NewIssue);

    public static async Task Open(string url)
    {
        try
        {
            var opened = await Launcher.OpenAsync(url);
            if (!opened)
                await Show(url);
        }
        catch (Exception)
        {
            await Show(url);
        }
    }

    private static async Task Show(string url)
    {
        if (Shell.Current?.CurrentPage != null)
            await Shell.Current.CurrentPage.DisplayAlert("Could not open link", url, "OK");
        else if (Shell.Current != null)
            await Shell.Current.DisplayAlert("Could not open link", url, "OK");
    }
}
