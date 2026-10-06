using ScalarScope.Services;
using ScalarScope.ViewModels;

namespace ScalarScope;

public partial class App : Application
{
    public static VortexSessionViewModel Session { get; } = new();
    public static ComparisonViewModel Comparison { get; } = new();
    public static KeyboardService Keyboard { get; private set; } = null!;
    public static bool NeedsRecovery { get; private set; }

    /// <summary>Shown on Home when recovery navigation itself fails.</summary>
    public static string? RecoveryNotice { get; set; }

    private static long _lastSessionSave;

    public App()
    {
        InitializeComponent();

        // Initialize crash reporting
        CrashReportingService.Initialize();

        // Check if we need to show recovery
        NeedsRecovery = CrashReportingService.DidRecoverFromCrash();

        // Apply saved theme preference
        var savedTheme = UserPreferencesService.GetTheme();
        if (savedTheme != AppTheme.Unspecified)
        {
            UserAppTheme = savedTheme;
        }

        ShellStartup.ApplyAccessibility();

        Keyboard = new KeyboardService(Session);
    }

    protected override Window CreateWindow(IActivationState? activationState)
    {
        var shell = new AppShell();
        shell.BindingContext = Session;

        var window = new Window(shell)
        {
            Title = "ScalarScope",
            Width = 1400,
            Height = 900
        };

        // Initialize demo state service for living empty states
        DemoStateService.Instance.Initialize();

        shell.Navigated += (_, _) => SaveSessionState();
        Session.Player.TimeChanged += SaveSessionThrottled;
        Comparison.Player.TimeChanged += SaveSessionThrottled;

        window.Destroying += OnWindowDestroying;

        return window;
    }

    private static void SaveSessionThrottled()
    {
        var now = Environment.TickCount64;
        if (now - _lastSessionSave < 2000)
            return;
        _lastSessionSave = now;
        SaveSessionState();
    }

    private void OnWindowDestroying(object? sender, EventArgs e)
    {
        try { SaveSessionState(); }
        catch (Exception ex) { ErrorLoggingService.Instance.Log(ex, "session save on close"); }

        try { Session.Player.Dispose(); }
        catch (Exception ex) { ErrorLoggingService.Instance.Log(ex, "session player dispose"); }

        try { Comparison.Player.Dispose(); }
        catch (Exception ex) { ErrorLoggingService.Instance.Log(ex, "comparison player dispose"); }

        CrashReportingService.MarkCleanShutdown();
    }

    /// <summary>
    /// Save current session state (call periodically or on significant changes).
    /// </summary>
    public static void SaveSessionState()
    {
        var state = new SessionState
        {
            LoadedFilePath = Session.LoadedFilePath,
            CurrentPage = Shell.Current?.CurrentState?.Location?.ToString() ?? "Trajectory",
            PlaybackTime = Session.Player?.Time ?? 0,
            IsPlaying = Session.Player?.IsPlaying ?? false,
            Theme = Application.Current?.RequestedTheme == AppTheme.Dark ? "dark" : "light"
        };

        CrashReportingService.SaveSessionState(state);
    }
}
