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
    private const string RecentComparisonsKey = "recent_comparisons";
    private const int MaxRecentItems = 3;

    [ObservableProperty]
    private bool _hasRecentComparisons;

    [ObservableProperty]
    private List<RecentComparisonItem> _recentComparisons = [];

    [ObservableProperty]
    private string _workspaceStatus = "No runs loaded";

    [ObservableProperty]
    private string _workspaceDetail = "Load inference traces or open a review bundle to begin comparison.";

    [ObservableProperty]
    private bool _canLoadRuns = true;

    public WelcomeViewModel()
    {
        RefreshRecentComparisons();
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
        App.Comparison.RequestDemoOpen();
        await Shell.Current.GoToAsync("//compare?demo=true");
    }

    /// <summary>
    /// Open a recent comparison.
    /// </summary>
    [RelayCommand]
    private async Task OpenRecent(RecentComparisonItem? item)
    {
        if (item == null) return;

        if (item.IsBundle && !string.IsNullOrEmpty(item.FilePath))
        {
            App.Comparison.RequestBundleOpen(item.FilePath);
            await Shell.Current.GoToAsync($"//compare?bundle={Uri.EscapeDataString(item.FilePath)}");
        }
        else
        {
            // For run comparisons, navigate to compare tab
            await Shell.Current.GoToAsync("//compare");
        }

        // Move this item to top of recents
        var existing = RecentComparisons.FirstOrDefault(r => r.Id == item.Id);
        if (existing != null)
        {
            existing.Timestamp = DateTimeOffset.UtcNow;
            SaveRecentComparisons();
            RefreshRecentComparisons();
        }
    }

    /// <summary>
    /// Clear all recent comparisons.
    /// </summary>
    [RelayCommand]
    private void ClearRecent()
    {
        RecentComparisons = [];
        HasRecentComparisons = false;
        SaveRecentComparisons();
    }

    /// <summary>
    /// Refresh recent comparisons from storage.
    /// </summary>
    public void RefreshRecentComparisons()
    {
        try
        {
            var json = Preferences.Get(RecentComparisonsKey, "[]");
            var items = JsonSerializer.Deserialize<List<RecentComparisonItem>>(json) ?? [];
            
            // Filter out items with missing files
            items = items.Where(i => 
                string.IsNullOrEmpty(i.FilePath) || 
                File.Exists(i.FilePath))
                .OrderByDescending(i => i.Timestamp)
                .Take(MaxRecentItems)
                .ToList();

            RecentComparisons = items;
            HasRecentComparisons = items.Count > 0;
        }
        catch
        {
            RecentComparisons = [];
            HasRecentComparisons = false;
        }
    }

    /// <summary>
    /// Add a comparison to the recent list.
    /// </summary>
    public void AddRecentComparison(RecentComparisonItem item)
    {
        var items = new List<RecentComparisonItem>(RecentComparisons);
        
        // Remove existing with same ID or path
        items.RemoveAll(i => 
            i.Id == item.Id || 
            (!string.IsNullOrEmpty(i.FilePath) && i.FilePath == item.FilePath));
        
        // Add to front
        items.Insert(0, item);
        
        // Trim to max
        if (items.Count > MaxRecentItems)
        {
            items = items.Take(MaxRecentItems).ToList();
        }

        RecentComparisons = items;
        HasRecentComparisons = items.Count > 0;
        SaveRecentComparisons();
    }

    private void SaveRecentComparisons()
    {
        try
        {
            var json = JsonSerializer.Serialize(RecentComparisons);
            Preferences.Set(RecentComparisonsKey, json);
        }
        catch
        {
            // Ignore save failures
        }
    }
}

/// <summary>
/// Represents a recent comparison or bundle for the welcome page.
/// </summary>
public class RecentComparisonItem
{
    public string Id { get; set; } = "";
    public string Title { get; set; } = "";
    public string Subtitle { get; set; } = "";
    public string Icon { get; set; } = "📊";
    public string? FilePath { get; set; }
    public bool IsBundle { get; set; }
    public DateTimeOffset Timestamp { get; set; }

    /// <summary>
    /// Human-readable time ago string.
    /// </summary>
    public string TimeAgo
    {
        get
        {
            var elapsed = DateTimeOffset.UtcNow - Timestamp;
            if (elapsed.TotalMinutes < 1) return "Just now";
            if (elapsed.TotalMinutes < 60) return $"{(int)elapsed.TotalMinutes}m ago";
            if (elapsed.TotalHours < 24) return $"{(int)elapsed.TotalHours}h ago";
            if (elapsed.TotalDays < 7) return $"{(int)elapsed.TotalDays}d ago";
            return Timestamp.LocalDateTime.ToString("MMM d");
        }
    }
}
