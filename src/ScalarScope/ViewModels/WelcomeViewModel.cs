using System.Text.Json;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ScalarScope.Services;

namespace ScalarScope.ViewModels;

/// <summary>
/// ViewModel for the Welcome/Landing page.
/// Shows workspace state, capabilities, and actions.
/// </summary>
public partial class WelcomeViewModel : ObservableObject
{
    private const string LegacyRecentKey = "recent_comparisons";
    private const string LegacyMigratedKey = "comparison_log_migrated";
    private const int HomeLimit = 12;

    [ObservableProperty]
    private bool _hasComparisonLog;

    [ObservableProperty]
    private List<ComparisonLogEntry> _comparisonLogEntries = [];

    [ObservableProperty]
    private string _workspaceStatus = "No runs loaded";

    [ObservableProperty]
    private string _workspaceDetail = "Load inference traces or open a review bundle to begin comparison.";

    [ObservableProperty]
    private bool _canLoadRuns = true;

    public WelcomeViewModel()
    {
        RefreshComparisonLog();
    }

    /// <summary>
    /// Navigate to the Compare tab to start a new comparison.
    /// </summary>
    [RelayCommand]
    private async Task NavigateToCompare()
    {
        await Shell.Current.GoToAsync("//compare");
    }

    /// <summary>
    /// Open a review bundle from the file system.
    /// </summary>
    [RelayCommand]
    private async Task OpenBundle()
    {
        try
        {
            var result = await FilePicker.PickAsync(new PickOptions
            {
                PickerTitle = "Open Review Bundle",
                FileTypes = new FilePickerFileType(new Dictionary<DevicePlatform, IEnumerable<string>>
                {
                    { DevicePlatform.WinUI, new[] { ".scbundle" } },
                    { DevicePlatform.macOS, new[] { "scbundle" } }
                })
            });

            if (result == null) return;

            // Compare reads this when it appears. The query string is the same request.
            App.Comparison.RequestBundleOpen(result.FullPath);
            await Shell.Current.GoToAsync($"//compare?bundle={Uri.EscapeDataString(result.FullPath)}");
        }
        catch (Exception ex)
        {
            if (Shell.Current != null)
            {
                await Shell.Current.DisplayAlert("Could not open bundle", ex.Message, "OK");
            }
        }
    }

    /// <summary>
    /// Load the built-in example comparison.
    /// </summary>
    [RelayCommand]
    private async Task TryExample()
    {
        try
        {
            App.Comparison.RequestDemoOpen();
            await Shell.Current.GoToAsync("//compare?demo=true");
        }
        catch (Exception ex)
        {
            if (Shell.Current != null)
                await Shell.Current.DisplayAlert("Could not open the example", ex.Message, "OK");
        }
    }

    /// <summary>
    /// Reopen a logged review. A bundle opens the bundle. Two run files open those files.
    /// An example loads the built-in runs.
    /// </summary>
    [RelayCommand]
    private async Task OpenLogEntry(ComparisonLogEntry? entry)
    {
        if (entry == null)
            return;

        try
        {
            if (!string.IsNullOrWhiteSpace(entry.BundlePath) && File.Exists(entry.BundlePath))
            {
                App.Comparison.RequestBundleOpen(entry.BundlePath);
                await Shell.Current.GoToAsync($"//compare?bundle={Uri.EscapeDataString(entry.BundlePath)}");
                return;
            }

            if (string.Equals(entry.Kind, "example", StringComparison.Ordinal))
            {
                App.Comparison.RequestDemoOpen();
                await Shell.Current.GoToAsync("//compare?demo=true");
                return;
            }

            if (!string.IsNullOrWhiteSpace(entry.LeftPath)
                && !string.IsNullOrWhiteSpace(entry.RightPath)
                && File.Exists(entry.LeftPath)
                && File.Exists(entry.RightPath))
            {
                App.Comparison.RequestRunsOpen(entry.LeftPath, entry.RightPath);
                await Shell.Current.GoToAsync("//compare");
                return;
            }

            if (Shell.Current != null)
            {
                await Shell.Current.DisplayAlert(
                    "Cannot reopen",
                    "The files for this review are no longer on disk. The deltas that fired are still in the log.",
                    "OK");
            }
        }
        catch (Exception ex)
        {
            if (Shell.Current != null)
                await Shell.Current.DisplayAlert("Cannot reopen", ex.Message, "OK");
        }
    }

    /// <summary>
    /// Clear the local comparison log.
    /// </summary>
    [RelayCommand]
    private void ClearLog()
    {
        try
        {
            ComparisonLog.Clear(ComparisonLog.DefaultDirectory);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Comparison log: {ex.Message}");
        }

        ComparisonLogEntries = [];
        HasComparisonLog = false;
    }

    /// <summary>
    /// Reload the log Home shows.
    /// </summary>
    public void RefreshComparisonLog()
    {
        try
        {
            MigrateLegacyRecents();
            var entries = ComparisonLog.Read(ComparisonLog.DefaultDirectory)
                .Take(HomeLimit)
                .ToList();
            ComparisonLogEntries = entries;
            HasComparisonLog = entries.Count > 0;
        }
        catch
        {
            ComparisonLogEntries = [];
            HasComparisonLog = false;
        }
    }

    private static void MigrateLegacyRecents()
    {
        if (Preferences.Get(LegacyMigratedKey, false))
            return;

        Preferences.Set(LegacyMigratedKey, true);

        var json = Preferences.Get(LegacyRecentKey, "[]");
        var legacy = JsonSerializer.Deserialize<List<LegacyRecentItem>>(json) ?? [];
        if (legacy.Count == 0)
            return;

        if (ComparisonLog.Read(ComparisonLog.DefaultDirectory).Count > 0)
            return;

        foreach (var item in legacy.OrderBy(item => item.Timestamp))
        {
            ComparisonLog.Record(new ComparisonLogEntry
            {
                Kind = item.IsBundle ? "bundle" : "compare",
                LeftName = string.IsNullOrWhiteSpace(item.Title) ? "Earlier review" : item.Title,
                BundlePath = item.IsBundle ? item.FilePath : null,
                LeftPath = item.IsBundle ? null : item.FilePath,
                FinishedAt = item.Timestamp
            }, ComparisonLog.DefaultDirectory);
        }
    }

    private sealed class LegacyRecentItem
    {
        public string Title { get; set; } = "";
        public string? FilePath { get; set; }
        public bool IsBundle { get; set; }
        public DateTimeOffset Timestamp { get; set; }
    }
}
