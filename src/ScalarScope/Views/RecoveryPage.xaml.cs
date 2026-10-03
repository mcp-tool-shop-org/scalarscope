using ScalarScope.Services;

namespace ScalarScope.Views;

public partial class RecoveryPage : ContentPage
{
    public RecoveryPage()
    {
        InitializeComponent();
        LoadRecoveryInfo();
    }

    private void LoadRecoveryInfo()
    {
        // Load session state
        var sessionState = CrashReportingService.GetLastSessionState();
        if (sessionState != null)
        {
            LastFileLabel.Text = string.IsNullOrEmpty(sessionState.LoadedFilePath)
                ? "(No file loaded)"
                : Path.GetFileName(sessionState.LoadedFilePath);
            LastPageLabel.Text = sessionState.CurrentPage;
            LastTimeLabel.Text = $"{sessionState.PlaybackTime:P0} through playback";
        }
        else
        {
            LastFileLabel.Text = "(Unknown)";
            LastPageLabel.Text = "(Unknown)";
            LastTimeLabel.Text = "(Unknown)";
        }

        // Load crash info
        var crashInfo = CrashReportingService.GetLastCrashInfo();
        if (crashInfo != null)
        {
            var timeSince = DateTime.UtcNow - crashInfo.Timestamp;
            var timeAgo = timeSince.TotalMinutes < 60
                ? $"{(int)timeSince.TotalMinutes} minutes ago"
                : timeSince.TotalHours < 24
                    ? $"{(int)timeSince.TotalHours} hours ago"
                    : $"{(int)timeSince.TotalDays} days ago";

            CrashInfoLabel.Text = $"The app stopped unexpectedly {timeAgo}.\n" +
                                   $"Error type: {crashInfo.ExceptionType?.Split('.').LastOrDefault() ?? "Unknown"}";
        }
        else
        {
            CrashInfoLabel.Text = "The app stopped unexpectedly during your last session.";
        }
    }

    private async void OnResumeClicked(object sender, EventArgs e)
    {
        // Acknowledge the crash
        CrashReportingService.AcknowledgeCrash();

        // Load the session state and restore
        var sessionState = CrashReportingService.GetLastSessionState();
        if (sessionState != null && !string.IsNullOrEmpty(sessionState.LoadedFilePath))
        {
            try
            {
                // Check if file still exists
                if (File.Exists(sessionState.LoadedFilePath))
                {
                    // Load the file
                    await App.Session.LoadFromFileAsync(sessionState.LoadedFilePath);

                    // Restore playback position
                    if (App.Session.HasRun)
                    {
                        App.Session.Player.JumpToTimeCommand.Execute(sessionState.PlaybackTime);

                        // Resume playback if it was playing
                        if (sessionState.IsPlaying)
                        {
                            App.Session.Player.PlayPauseCommand.Execute(null);
                        }
                    }
                }
            }
            catch
            {
                // Failed to restore - continue to normal navigation
            }
        }

        CrashReportingService.ClearSessionState();

        var hasFile = sessionState != null
            && !string.IsNullOrEmpty(sessionState.LoadedFilePath)
            && File.Exists(sessionState.LoadedFilePath);

        await Shell.Current.GoToAsync(ResumeDestination(sessionState?.CurrentPage, hasFile));
    }

    /// <summary>
    /// Only the four shell tabs exist. A restored file opens Compare. Anything else opens Home.
    /// </summary>
    private static string ResumeDestination(string? currentPage, bool hasFile)
    {
        return currentPage?.Trim().ToLowerInvariant() switch
        {
            "welcome" => "//welcome",
            "compare" => "//compare",
            "help" => "//help",
            "settings" => "//settings",
            _ => hasFile ? "//compare" : "//welcome"
        };
    }

    private async void OnStartFreshClicked(object sender, EventArgs e)
    {
        // Acknowledge and clear everything
        CrashReportingService.AcknowledgeCrash();
        CrashReportingService.ClearSessionState();

        await Shell.Current.GoToAsync("//welcome");
    }

    private async void OnSupportBundleClicked(object sender, EventArgs e)
    {
        try
        {
            var bundlePath = await CrashReportingService.GenerateSupportBundleAsync();

            await DisplayAlert(
                "Support Bundle Created",
                $"A support bundle has been saved to:\n\n{bundlePath}\n\nPlease include this file when reporting issues.",
                "OK");
        }
        catch (Exception ex)
        {
            await DisplayAlert(
                "Error",
                $"Failed to create support bundle: {ex.Message}",
                "OK");
        }
    }
}
