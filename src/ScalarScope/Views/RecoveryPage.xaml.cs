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
        var loadSucceeded = false;
        var fileWasPresent = false;
        string? loadError = null;
        if (sessionState != null && !string.IsNullOrEmpty(sessionState.LoadedFilePath))
        {
            fileWasPresent = File.Exists(sessionState.LoadedFilePath);
            if (fileWasPresent)
            {
                try
                {
                    await App.Session.LoadFromFileAsync(sessionState.LoadedFilePath);
                    loadSucceeded = App.Session.HasRun;
                    if (!loadSucceeded)
                        loadError = string.IsNullOrEmpty(App.Session.LoadError)
                            ? "the file did not load"
                            : App.Session.LoadError;
                    else
                    {
                        App.Session.Player.JumpToTimeCommand.Execute(sessionState.PlaybackTime);
                        if (sessionState.IsPlaying)
                            App.Session.Player.PlayPauseCommand.Execute(null);
                    }
                }
                catch (Exception ex)
                {
                    loadError = ex.Message;
                }
            }
        }

        if (RecoveryResume.ShouldClearSession(loadSucceeded))
            CrashReportingService.ClearSessionState();

        var hasFile = fileWasPresent && loadSucceeded;
        var route = ShellDestinations.Resume(sessionState?.CurrentPage, hasFile);
        var alert = RecoveryResume.Alert(loadSucceeded, fileWasPresent, loadError, route);
        if (!string.IsNullOrEmpty(alert))
            await DisplayAlert("Could not resume", alert, "OK");

        await GoToTab(route, hasFile);
    }

    private async void OnStartFreshClicked(object sender, EventArgs e)
    {
        // Acknowledge and clear everything
        CrashReportingService.AcknowledgeCrash();
        CrashReportingService.ClearSessionState();

        await GoToTab(ShellDestinations.Welcome, fileStillOnDisk: false);
    }

    /// <summary>
    /// Only welcome, compare, help, and settings are registered tabs.
    /// A failed route falls back to one of those. If that also fails, stay here.
    /// </summary>
    private async Task GoToTab(string route, bool fileStillOnDisk)
    {
        if (!ShellDestinations.IsTab(route))
            route = fileStillOnDisk ? ShellDestinations.Compare : ShellDestinations.Welcome;

        try
        {
            await Shell.Current.GoToAsync(route);
        }
        catch (Exception ex)
        {
            var fallback = fileStillOnDisk ? ShellDestinations.Compare : ShellDestinations.Welcome;
            if (!string.Equals(route, fallback, StringComparison.Ordinal))
            {
                try
                {
                    await Shell.Current.GoToAsync(fallback);
                    return;
                }
                catch (Exception fallbackEx)
                {
                    ex = fallbackEx;
                }
            }

            await DisplayAlert("Could not leave recovery", ex.Message, "OK");
        }
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
