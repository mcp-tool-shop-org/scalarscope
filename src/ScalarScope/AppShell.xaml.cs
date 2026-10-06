using ScalarScope.Services;
using ScalarScope.Views;

namespace ScalarScope;

public partial class AppShell : Shell
{
    public AppShell()
    {
        InitializeComponent();

        // Register routes
        Routing.RegisterRoute("recovery", typeof(RecoveryPage));

        var snapshotPath = CrashReportingService.GetLastSessionState()?.LoadedFilePath;
        if (App.NeedsRecovery || ShellStartup.ShouldOpenSavedSession(
                UserPreferencesService.GetAutoLoadLastSession(),
                snapshotPath))
            Loaded += NavigateToRecovery;

        Navigated += (_, _) => AccessibleShell.Apply(this);
        AccessibilityService.Instance.SettingsChanged += () =>
        {
            try { MainThread.BeginInvokeOnMainThread(() => AccessibleShell.Apply(this)); }
            catch (InvalidOperationException) { AccessibleShell.Apply(this); }
        };
    }

    private async void NavigateToRecovery(object? sender, EventArgs e)
    {
        Loaded -= NavigateToRecovery;
        try
        {
            await GoToAsync("recovery");
        }
        catch (Exception ex)
        {
            App.RecoveryNotice = "Recovery could not open. " + ex.Message;
            try { await GoToAsync("//welcome"); }
            catch { /* Stay on the page that is already showing. */ }
        }
    }
}
