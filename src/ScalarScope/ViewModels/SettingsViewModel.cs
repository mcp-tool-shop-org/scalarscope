using CommunityToolkit.Maui.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ScalarScope.Services;

namespace ScalarScope.ViewModels;

/// <summary>
/// ViewModel for the Settings page.
/// Uses field-based ObservableProperty for broad C# version compatibility.
/// </summary>
public partial class SettingsViewModel : ObservableObject
{
    public SettingsViewModel()
    {
        LoadSettings();
    }

    // --- Theme Settings ---

    [ObservableProperty]
    private int _themeIndex;

    public string[] ThemeOptions { get; } = ["System", "Light", "Dark"];

    partial void OnThemeIndexChanged(int value)
    {
        var theme = value switch
        {
            1 => AppTheme.Light,
            2 => AppTheme.Dark,
            _ => AppTheme.Unspecified
        };
        UserPreferencesService.SetTheme(theme);
        if (Application.Current != null)
            Application.Current.UserAppTheme = theme;
        NoteStorage();
    }

    // --- Animation Settings ---

    [ObservableProperty]
    private bool _reduceAnimations;

    partial void OnReduceAnimationsChanged(bool value)
    {
        UserPreferencesService.SetReduceAnimations(value);
        NoteStorage();
    }

    // --- Playback Settings ---

    [ObservableProperty]
    private int _defaultSpeedIndex;

    public string[] SpeedOptions { get; } = ["0.25x", "0.5x", "1x", "2x", "4x"];

    public float[] SpeedValues { get; } = [0.25f, 0.5f, 1f, 2f, 4f];

    partial void OnDefaultSpeedIndexChanged(int value)
    {
        if (value >= 0 && value < SpeedValues.Length)
        {
            UserPreferencesService.SetDefaultPlaybackSpeed(SpeedValues[value]);
        }
        NoteStorage();
    }

    [ObservableProperty]
    private bool _autoPlayOnLoad;

    partial void OnAutoPlayOnLoadChanged(bool value)
    {
        UserPreferencesService.SetAutoPlayOnLoad(value);
        NoteStorage();
    }

    // --- Session Settings ---

    [ObservableProperty]
    private bool _autoLoadLastSession;

    partial void OnAutoLoadLastSessionChanged(bool value)
    {
        UserPreferencesService.SetAutoLoadLastSession(value);
        NoteStorage();
    }

    [ObservableProperty]
    private int _recentFilesLimitIndex;

    public string[] RecentFilesOptions { get; } = ["5 files", "10 files", "20 files"];

    public int[] RecentFilesValues { get; } = [5, 10, 20];

    partial void OnRecentFilesLimitIndexChanged(int value)
    {
        if (value >= 0 && value < RecentFilesValues.Length)
        {
            UserPreferencesService.SetRecentFilesLimit(RecentFilesValues[value]);
        }
        NoteStorage();
    }

    // --- Export Settings ---

    [ObservableProperty]
    private string _defaultExportPath = "";

    [ObservableProperty]
    private int _defaultExportWidth = 1920;

    [ObservableProperty]
    private int _defaultExportHeight = 1080;

    partial void OnDefaultExportWidthChanged(int value)
    {
        UserPreferencesService.SetDefaultExportResolution(value, DefaultExportHeight);
        NoteStorage();
    }

    partial void OnDefaultExportHeightChanged(int value)
    {
        UserPreferencesService.SetDefaultExportResolution(DefaultExportWidth, value);
        NoteStorage();
    }

    // --- Accessibility Settings ---

    [ObservableProperty]
    private bool _highContrastMode;

    partial void OnHighContrastModeChanged(bool value)
    {
        UserPreferencesService.SetHighContrastMode(value);
        AccessibilityService.Instance.Settings = AccessibilityService.Instance.Settings with
        {
            HighContrastEnabled = value
        };
        NoteStorage();
    }

    [ObservableProperty]
    private int _annotationDensityIndex;

    public string[] AnnotationOptions { get; } = ["Minimal", "Standard", "Full"];

    partial void OnAnnotationDensityIndexChanged(int value)
    {
        var density = value switch
        {
            0 => AnnotationDensity.Minimal,
            2 => AnnotationDensity.Full,
            _ => AnnotationDensity.Standard
        };
        UserPreferencesService.SetAnnotationDensity(density);
        NoteStorage();
    }

    // Phase 4.3: Enhanced Accessibility

    [ObservableProperty]
    private int _colorVisionIndex;

    public string[] ColorVisionOptions { get; } =
    [
        "Default",
        "Deuteranopia (Green-blind)",
        "Protanopia (Red-blind)",
        "Tritanopia (Blue-blind)",
        "High Contrast",
        "Monochrome"
    ];

    partial void OnColorVisionIndexChanged(int value)
    {
        var index = ShellStartup.ClampColorVisionIndex(value);
        if (index != value)
        {
            ColorVisionIndex = index;
            return;
        }

        var mode = (ColorPaletteMode)index;
        UserPreferencesService.SetColorVisionMode(index);
        AccessibilityService.Instance.Settings = AccessibilityService.Instance.Settings with
        {
            ColorPaletteMode = mode
        };
        NoteStorage();
    }

    [ObservableProperty]
    private bool _screenReaderMode;

    partial void OnScreenReaderModeChanged(bool value)
    {
        UserPreferencesService.SetScreenReaderMode(value);
        AccessibilityService.Instance.Settings = AccessibilityService.Instance.Settings with
        {
            ScreenReaderEnabled = value
        };
        NoteStorage();
    }

    [ObservableProperty]
    private bool _largePointer;

    partial void OnLargePointerChanged(bool value)
    {
        UserPreferencesService.SetLargePointer(value);
        AccessibilityService.Instance.Settings = AccessibilityService.Instance.Settings with
        {
            LargePointer = value
        };
        NoteStorage();
    }

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(TextScaleDisplay))]
    private float _textScale = 1.0f;

    public string TextScaleDisplay => $"{TextScale:P0}";

    partial void OnTextScaleChanged(float value)
    {
        var clamped = ShellStartup.ClampTextScale(value);
        if (Math.Abs(clamped - value) > 0.001f)
        {
            TextScale = clamped;
            return;
        }

        UserPreferencesService.SetTextScale(clamped);
        AccessibilityService.Instance.Settings = AccessibilityService.Instance.Settings with
        {
            TextScale = clamped
        };
        NoteStorage();
    }

    // --- Commands ---

    [RelayCommand]
    private async Task BrowseExportPathAsync()
    {
        try
        {
            var result = await FolderPicker.Default.PickAsync(new CancellationToken());
            if (result.IsSuccessful && !string.IsNullOrEmpty(result.Folder?.Path))
            {
                DefaultExportPath = result.Folder.Path;
                UserPreferencesService.SetDefaultExportPath(DefaultExportPath);
            }
        }
        catch
        {
            // Folder picker not supported or canceled
        }
    }

    [RelayCommand]
    private void ResetAllSettings()
    {
        UserPreferencesService.ResetAllSettings();
        LoadSettings();

        // Reset theme to system
        Application.Current!.UserAppTheme = AppTheme.Unspecified;
    }

    [RelayCommand]
    private void ClearRecentFiles()
    {
        UserPreferencesService.ClearRecentFiles();
    }

    [RelayCommand]
    private void ResetDemoState()
    {
        UserPreferencesService.ResetFirstRunState();
    }

    // --- Load/Save ---

    private void LoadSettings()
    {
        // Theme
        ThemeIndex = UserPreferencesService.GetTheme() switch
        {
            AppTheme.Light => 1,
            AppTheme.Dark => 2,
            _ => 0
        };

        // Animations
        ReduceAnimations = UserPreferencesService.GetReduceAnimations();

        // Playback
        var speed = UserPreferencesService.GetDefaultPlaybackSpeed();
        DefaultSpeedIndex = Array.IndexOf(SpeedValues, speed);
        if (DefaultSpeedIndex < 0) DefaultSpeedIndex = 2; // Default to 1x

        AutoPlayOnLoad = UserPreferencesService.GetAutoPlayOnLoad();

        // Session
        AutoLoadLastSession = UserPreferencesService.GetAutoLoadLastSession();

        var limit = UserPreferencesService.GetRecentFilesLimit();
        RecentFilesLimitIndex = Array.IndexOf(RecentFilesValues, limit);
        if (RecentFilesLimitIndex < 0) RecentFilesLimitIndex = 1; // Default to 10

        // Export
        DefaultExportPath = UserPreferencesService.GetDefaultExportPath();
        var (width, height) = UserPreferencesService.GetDefaultExportResolution();
        DefaultExportWidth = width;
        DefaultExportHeight = height;

        // Accessibility
        HighContrastMode = UserPreferencesService.GetHighContrastMode();
        AnnotationDensityIndex = UserPreferencesService.GetAnnotationDensity() switch
        {
            AnnotationDensity.Minimal => 0,
            AnnotationDensity.Full => 2,
            _ => 1
        };

        // Phase 4.3: Enhanced Accessibility
        ColorVisionIndex = ShellStartup.ClampColorVisionIndex(UserPreferencesService.GetColorVisionMode());
        ScreenReaderMode = UserPreferencesService.GetScreenReaderMode();
        LargePointer = UserPreferencesService.GetLargePointer();
        TextScale = ShellStartup.ClampTextScale(UserPreferencesService.GetTextScale());

        // Update AccessibilityService with loaded settings
        AccessibilityService.Instance.Settings = new AccessibilitySettings
        {
            ColorPaletteMode = (ColorPaletteMode)ColorVisionIndex,
            HighContrastEnabled = HighContrastMode,
            ReducedMotionEnabled = ReduceAnimations,
            ScreenReaderEnabled = ScreenReaderMode,
            TextScale = TextScale,
            LargePointer = LargePointer
        };
        NoteStorage();
    }

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(HasStorageNotice))]
    private string _storageNotice = "";

    public bool HasStorageNotice => !string.IsNullOrWhiteSpace(StorageNotice);

    private void NoteStorage()
    {
        StorageNotice = UserPreferencesService.StorageNotice ?? "";
    }

    // --- About Section Commands ---

    [RelayCommand]
    private Task OpenDocumentation() => ProductLinks.OpenDocumentation();

    [RelayCommand]
    private Task OpenGitHub() => ProductLinks.OpenRepository();

    [RelayCommand]
    private Task OpenIssue() => ProductLinks.OpenNewIssue();
}
