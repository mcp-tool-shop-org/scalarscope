using System.Globalization;
using System.Text.Json;
using ScalarScope.Models;
using ScalarScope.Services;
using ScalarScope.Services.Connectors;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace ScalarScope.ViewModels;

/// <summary>
/// ViewModel for side-by-side run comparison.
/// Manages two runs with synchronized playback.
/// </summary>
public partial class ComparisonViewModel : ObservableObject
{
    // Left run (typically Path A / orthogonal)
    [ObservableProperty]
    private GeometryRun? _leftRun;

    [ObservableProperty]
    private string _leftRunName = "Load Path A";

    [ObservableProperty]
    private bool _hasLeftRun;

    // Right run (typically Path B / correlated)
    [ObservableProperty]
    private GeometryRun? _rightRun;

    [ObservableProperty]
    private string _rightRunName = "Load Path B";

    [ObservableProperty]
    private bool _hasRightRun;

    private string? _leftSourcePath;
    private string? _rightSourcePath;
    private RuntimeRunTrace? _leftTrace;
    private RuntimeRunTrace? _rightTrace;

    // Comparison state
    [ObservableProperty]
    private bool _hasBothRuns;

    [ObservableProperty]
    private string _frameworkName = "TFRT";

    [ObservableProperty]
    private bool _hasPreset = true;

    [ObservableProperty]
    private string _presetName = "TFRT Runtime";

    [ObservableProperty]
    private int _deltaCount;

    [ObservableProperty]
    private string _comparisonSummary = "";

    [ObservableProperty]
    private string _interpretationVerdict = "";

    [ObservableProperty]
    private string _leftDescription = "";

    [ObservableProperty]
    private string _rightDescription = "";

    // Visual dominance indicators for canvas dimming
    [ObservableProperty]
    private bool? _isLeftDominant;

    [ObservableProperty]
    private bool? _isRightDominant;

    // Demo mode state
    [ObservableProperty]
    private bool _isDemoMode;

    [ObservableProperty]
    private bool _isDemoComplete;

    // Overlay mode - Phase 3.2 Comparative Analysis
    [ObservableProperty]
    private bool _isOverlayMode;

    [ObservableProperty]
    private bool _showTraceReview;

    [ObservableProperty]
    private bool _showTraceDistribution;

    [ObservableProperty]
    private bool _showGeometryChrome;

    [ObservableProperty]
    private bool _showSideBySide;

    [ObservableProperty]
    private bool _showOverlay;

    [ObservableProperty]
    private TraceReview? _traceReview;

    [ObservableProperty]
    private string _traceNote = "";

    [ObservableProperty]
    private bool _hasTraceNote;

    [ObservableProperty]
    private bool _showDeviation = true;

    [ObservableProperty]
    private bool _showStatisticalBands;

    [ObservableProperty]
    private bool _showDistanceMetrics = true;

    // === Phase 3: Make Comparison the Star ===

    // Temporal Alignment
    [ObservableProperty]
    private TemporalAlignment _selectedAlignment = TemporalAlignment.ByStep;

    [ObservableProperty]
    private string _alignmentDescription = "Aligned by training step";

    // Canonical Deltas
    [ObservableProperty]
    private List<CanonicalDelta> _canonicalDeltas = [];

    [ObservableProperty]
    private string _autoSummary = "";

    // Visual emphasis state
    [ObservableProperty]
    private double _highlightedAnchorTime = -1;

    [ObservableProperty]
    private string? _highlightedDeltaId;

    // Compare mode state
    [ObservableProperty]
    private bool _isCompareMode;

    // Phase 7.2: Review mode state (viewing bundle, not live data)
    [ObservableProperty]
    private bool _isReviewMode;

    [ObservableProperty]
    private LoadedBundle? _reviewBundle;

    /// <summary>
    /// Phase 7.2: Enter review mode with a loaded bundle.
    /// Disables compute buttons and shows review mode banner.
    /// </summary>
    public void EnterReviewMode(LoadedBundle bundle)
    {
        _reviewBundle = bundle;
        IsReviewMode = true;
        
        // Set deltas from bundle
        CanonicalDeltas = bundle.Deltas;
        TryRecordBundle(bundle);
        
        // Set summary text from bundle
        AutoSummary = bundle.SummaryMarkdown ?? "";
        
        // Disable playback (bundle data is static)
        if (Player.IsPlaying)
        {
            Player.IsPlaying = false;
        }
        
        OnPropertyChanged(nameof(ReviewBundle));
    }

    /// <summary>
    /// Phase 7.2: Exit review mode and return to live comparisons.
    /// </summary>
    public void ExitReviewMode()
    {
        _reviewBundle = null;
        IsReviewMode = false;
        
        // Clear bundled data
        CanonicalDeltas = [];
        AutoSummary = "";
        
        // Unload the bundle
        BundleImportService.Instance.Unload();
        
        OnPropertyChanged(nameof(ReviewBundle));
    }

    // Phase 5.2: State preservation for mode continuity
    private double _preservedTime;
    private string? _preservedHighlightedDeltaId;
    private string? _preservedLeftRunName;
    private string? _preservedRightRunName;

    /// <summary>
    /// Phase 5.2: Preserve current focus state before mode transition.
    /// </summary>
    public void PreserveFocusState()
    {
        _preservedTime = Player.Time;
        _preservedHighlightedDeltaId = HighlightedDeltaId;
        _preservedLeftRunName = LeftRunName;
        _preservedRightRunName = RightRunName;
    }

    /// <summary>
    /// Phase 5.2: Restore focus state after mode transition.
    /// Run labels persist; last focus region preserved.
    /// </summary>
    public void RestoreFocusState()
    {
        // Restore playback position if valid
        if (_preservedTime > 0 && _preservedTime <= Player.Duration)
        {
            Player.JumpToTimeCommand.Execute(_preservedTime);
        }
        
        // Restore highlighted delta if still valid
        if (!string.IsNullOrEmpty(_preservedHighlightedDeltaId))
        {
            var stillExists = CanonicalDeltas?.Any(d => d.Id == _preservedHighlightedDeltaId) ?? false;
            if (stillExists)
            {
                HighlightedDeltaId = _preservedHighlightedDeltaId;
            }
        }
    }

    // Collection of runs for overlay view
    public List<GeometryRun> OverlayRuns => [.. (new[] { LeftRun, RightRun }).OfType<GeometryRun>()];

    // Shared playback controller
    public TrajectoryPlayerViewModel Player { get; } = new();

    // Home sets one of these, then navigates here. The page takes it once.
    private string? _pendingBundlePath;
    private string? _pendingLeftPath;
    private string? _pendingRightPath;
    private bool _pendingDemo;

    /// <summary>
    /// Ask Compare to import this bundle the next time the page appears.
    /// </summary>
    public void RequestBundleOpen(string path)
    {
        _pendingBundlePath = path;
        _pendingLeftPath = null;
        _pendingRightPath = null;
        _pendingDemo = false;
    }

    /// <summary>
    /// Ask Compare to load the built-in example the next time the page appears.
    /// </summary>
    public void RequestDemoOpen()
    {
        _pendingBundlePath = null;
        _pendingLeftPath = null;
        _pendingRightPath = null;
        _pendingDemo = true;
    }

    /// <summary>
    /// Ask Compare to reload these two run files the next time the page appears.
    /// </summary>
    public void RequestRunsOpen(string leftPath, string rightPath)
    {
        _pendingBundlePath = null;
        _pendingLeftPath = leftPath;
        _pendingRightPath = rightPath;
        _pendingDemo = false;
    }

    /// <summary>
    /// Take the one open Home asked for. A second call returns nothing.
    /// </summary>
    public bool TryTakeOpenRequest(out string? bundlePath, out bool demo, out string? leftPath, out string? rightPath)
    {
        bundlePath = _pendingBundlePath;
        leftPath = _pendingLeftPath;
        rightPath = _pendingRightPath;
        demo = _pendingDemo && string.IsNullOrEmpty(bundlePath);
        var hasRequest = !string.IsNullOrEmpty(bundlePath)
            || _pendingDemo
            || (!string.IsNullOrEmpty(leftPath) && !string.IsNullOrEmpty(rightPath));
        _pendingBundlePath = null;
        _pendingLeftPath = null;
        _pendingRightPath = null;
        _pendingDemo = false;
        return hasRequest;
    }

    // Computed properties for current time - Left
    public TrajectoryTimestep? LeftCurrentTrajectory => GetTrajectoryAtTime(LeftRun, Player.Time);
    public ScalarTimestep? LeftCurrentScalars => GetScalarsAtTime(LeftRun, Player.Time);
    public EigenTimestep? LeftCurrentEigenvalues => GetEigenvaluesAtTime(LeftRun, Player.Time);

    // Computed properties for current time - Right
    public TrajectoryTimestep? RightCurrentTrajectory => GetTrajectoryAtTime(RightRun, Player.Time);
    public ScalarTimestep? RightCurrentScalars => GetScalarsAtTime(RightRun, Player.Time);
    public EigenTimestep? RightCurrentEigenvalues => GetEigenvaluesAtTime(RightRun, Player.Time);

    public ComparisonViewModel()
    {
        Player.TimeChanged += OnTimeChanged;
    }

    private void OnTimeChanged()
    {
        // Invariant check: time must always be valid
        var clampedTime = InvariantGuard.ClampTime(Player.Time, "ComparisonViewModel.OnTimeChanged");
        if (Math.Abs(clampedTime - Player.Time) > 0.001)
        {
            // Time was out of bounds - this should never happen but we handle it
            Player.JumpToTimeCommand.Execute(clampedTime);
        }

        NotifyComputedPropertiesChanged();
        
        // Phase 3: Update deltas with current time. A trace review keeps the playhead and does not recompute geometry deltas.
        if (HasBothRuns && !ShowTraceReview)
        {
            UpdateCanonicalDeltas();
        }
    }

    partial void OnIsOverlayModeChanged(bool value) => SyncChrome();

    /// <summary>
    /// Called when SelectedAlignment property changes.
    /// Recomputes deltas and updates alignment description.
    /// </summary>
    partial void OnSelectedAlignmentChanged(TemporalAlignment value)
    {
        if (ShowTraceReview)
            return;

        // Update alignment description
        var anchors = TemporalAlignmentService.GetAnchors(LeftRun, RightRun, value);
        AlignmentDescription = anchors.AnchorDescription;

        // Recompute deltas with new alignment
        if (HasBothRuns)
        {
            UpdateCanonicalDeltas();
            TryRecordFinishedComparison();
        }
    }

    /// <summary>
    /// Update canonical deltas based on current runs, alignment, and time.
    /// </summary>
    private void UpdateCanonicalDeltas()
    {
        // Phase 5.5: Track delta computation performance
        using (PerformanceProfiler.BeginScope("DeltaComputation"))
        {
            CanonicalDeltas = CanonicalDeltaService.ComputeDeltas(
                LeftRun, RightRun, SelectedAlignment, Player.Time);
        }
        
        DeltaCount = CanonicalDeltas.Count(d => d.Status == DeltaStatus.Present);
        AutoSummary = CanonicalDeltaService.GenerateAutoSummary(CanonicalDeltas);
    }

    /// <summary>
    /// Jump to the visual anchor time of a delta.
    /// Phase 3: Clicking delta scrolls/focuses source region.
    /// </summary>
    [RelayCommand]
    public void JumpToDeltaAnchor(CanonicalDelta delta)
    {
        if (delta == null) return;
        
        HighlightedDeltaId = delta.Id;
        HighlightedAnchorTime = delta.VisualAnchorTime;
        Player.JumpToTimeCommand.Execute(delta.VisualAnchorTime);
    }

    /// <summary>
    /// Highlight a delta (for hover interaction).
    /// </summary>
    public void HighlightDelta(CanonicalDelta? delta)
    {
        HighlightedDeltaId = delta?.Id;
        HighlightedAnchorTime = delta?.VisualAnchorTime ?? -1;
    }

    private void NotifyComputedPropertiesChanged()
    {
        OnPropertyChanged(nameof(LeftCurrentTrajectory));
        OnPropertyChanged(nameof(LeftCurrentScalars));
        OnPropertyChanged(nameof(LeftCurrentEigenvalues));
        OnPropertyChanged(nameof(RightCurrentTrajectory));
        OnPropertyChanged(nameof(RightCurrentScalars));
        OnPropertyChanged(nameof(RightCurrentEigenvalues));
        OnPropertyChanged(nameof(OverlayRuns));
    }

    /// <summary>
    /// Reset the left run to initial state.
    /// </summary>
    [RelayCommand]
    public void ResetLeftRun()
    {
        LeftRun = null;
        _leftTrace = null;
        LeftRunName = "Load Path A";
        HasLeftRun = false;
        _leftSourcePath = null;
        RefreshPair();
        NotifyComputedPropertiesChanged();
    }

    /// <summary>
    /// Reset the right run to initial state.
    /// </summary>
    [RelayCommand]
    public void ResetRightRun()
    {
        RightRun = null;
        _rightTrace = null;
        RightRunName = "Load Path B";
        HasRightRun = false;
        _rightSourcePath = null;
        RefreshPair();
        NotifyComputedPropertiesChanged();
    }

    /// <summary>
    /// Reset both runs to initial state.
    /// </summary>
    [RelayCommand]
    public void ResetAll()
    {
        if (Player.IsPlaying)
        {
            Player.PlayPauseCommand.Execute(null);
        }

        LeftRun = null;
        _leftTrace = null;
        LeftRunName = "Load Path A";
        HasLeftRun = false;
        _leftSourcePath = null;

        RightRun = null;
        _rightTrace = null;
        RightRunName = "Load Path B";
        HasRightRun = false;
        _rightSourcePath = null;

        ShowTraceReview = false;
        TraceReview = null;
        TraceNote = "";
        HasTraceNote = false;
        HasBothRuns = false;
        ComparisonSummary = "";
        InterpretationVerdict = "";
        LeftDescription = "";
        RightDescription = "";
        IsLeftDominant = null;
        IsRightDominant = null;

        Player.JumpToTimeCommand.Execute(0.0);
        SyncChrome();
        NotifyComputedPropertiesChanged();
    }

    /// <summary>
    /// Load demo runs directly without file picker.
    /// Called by the demo flow to initialize comparison view.
    /// Phase 5.2: Includes transition smoothing.
    /// </summary>
    public async void LoadDemoRuns(GeometryRun pathA, GeometryRun pathB)
    {
        // Phase 5.2: Preserve camera state for continuity
        TransitionService.PreserveCameraState(1f, 0, 0);
        
        // Reset any existing state
        if (Player.IsPlaying)
        {
            Player.PlayPauseCommand.Execute(null);
        }

        // Set demo mode
        IsDemoMode = true;
        IsDemoComplete = false;

        // Subscribe to demo completion
        DemoAnnotationService.DemoCompleted += OnDemoCompleted;

        // Load Path A (orthogonal) on the left
        _leftSourcePath = null;
        _rightSourcePath = null;
        _leftTrace = null;
        _rightTrace = null;
        ShowTraceReview = false;
        TraceReview = null;
        LeftRun = pathA;
        LeftRunName = pathA.Metadata?.Condition ?? "Path A: Orthogonal";
        HasLeftRun = true;

        // Load Path B (correlated) on the right
        RightRun = pathB;
        RightRunName = pathB.Metadata?.Condition ?? "Path B: Correlated";
        HasRightRun = true;

        // Update comparison state
        UpdateComparisonState();

        // Reset player to start
        Player.JumpToTimeCommand.Execute(0.0);
        NotifyComputedPropertiesChanged();
        
        // Phase 5.2: Smooth transition if coming from demo state
        if (DemoStateService.Instance.IsDemo)
        {
            await TransitionService.StartDemoToRealTransition();
            DemoStateService.Instance.TransitionToRealData();
        }
    }

    private void OnDemoCompleted()
    {
        IsDemoComplete = true;
        DemoService.EndDemo(completed: true);
    }

    /// <summary>
    /// End the demo early (user requested).
    /// </summary>
    [RelayCommand]
    public void EndDemo()
    {
        if (!IsDemoMode) return;

        // Stop playback
        if (Player.IsPlaying)
        {
            Player.PlayPauseCommand.Execute(null);
        }

        // Clean up demo state
        IsDemoMode = false;
        IsDemoComplete = false;
        DemoAnnotationService.DemoCompleted -= OnDemoCompleted;
        DemoService.EndDemo(completed: false);

        // Reset the comparison view
        ResetAll();
    }

    /// <summary>
    /// Silently end demo mode when user takes over with their own file.
    /// Does not reset the view - just cleans up demo state.
    /// </summary>
    private void EndDemoModeIfActive()
    {
        if (!IsDemoMode) return;

        IsDemoMode = false;
        IsDemoComplete = false;
        DemoAnnotationService.DemoCompleted -= OnDemoCompleted;
        DemoService.EndDemo(completed: false);
    }

    [RelayCommand]
    private async Task LoadLeftRunAsync()
    {
        var result = await FilePicker.Default.PickAsync(new PickOptions
        {
            PickerTitle = "Open path A",
            FileTypes = RunFileTypes()
        });

        if (result != null)
            await LoadLeftFromFileAsync(result.FullPath);
    }

    [RelayCommand]
    private async Task LoadRightRunAsync()
    {
        var result = await FilePicker.Default.PickAsync(new PickOptions
        {
            PickerTitle = "Open path B",
            FileTypes = RunFileTypes()
        });

        if (result != null)
            await LoadRightFromFileAsync(result.FullPath);
    }

    public Task LoadLeftFromFileAsync(string path) => LoadSideAsync(left: true, path);

    public Task LoadRightFromFileAsync(string path) => LoadSideAsync(left: false, path);

    private static FilePickerFileType RunFileTypes() => new(new Dictionary<DevicePlatform, IEnumerable<string>>
    {
        { DevicePlatform.WinUI, new[] { ".json", ".csv", ".log", ".gz" } },
        { DevicePlatform.macOS, new[] { "json", "csv", "log" } }
    });

    private async Task LoadSideAsync(bool left, string path)
    {
        EndDemoModeIfActive();

        string text;
        try
        {
            text = await File.ReadAllTextAsync(path);
        }
        catch (Exception ex)
        {
            FailSide(left, ex.Message);
            return;
        }

        if (TraceOpen.IsGeometryRun(text))
        {
            LoadGeometrySide(left, path, text);
            return;
        }

        try
        {
            var trace = await new TensorFlowRTOfflineConnector().ImportRuntimeAsync(path);
            SetTraceSide(left, path, trace);
        }
        catch (Exception ex)
        {
            FailSide(left, ex.Message);
        }
    }

    private void LoadGeometrySide(bool left, string path, string json)
    {
        try
        {
            var run = JsonSerializer.Deserialize<GeometryRun>(json);
            if (run == null)
            {
                FailSide(left, "The file did not contain a run.");
                return;
            }

            var where = left ? "LoadLeftFromFileAsync" : "LoadRightFromFileAsync";
            if (!InvariantGuard.AssertTrajectoryNonEmpty(run, $"{where}({Path.GetFileName(path)})"))
            {
                FailSide(left, "No trajectory data");
                return;
            }

            InvariantGuard.AssertDataConsistentLengths(run, $"{where}({Path.GetFileName(path)})");
            if (left)
            {
                _leftTrace = null;
                LeftRun = run;
                LeftRunName = Path.GetFileNameWithoutExtension(path);
                _leftSourcePath = path;
                HasLeftRun = true;
            }
            else
            {
                _rightTrace = null;
                RightRun = run;
                RightRunName = Path.GetFileNameWithoutExtension(path);
                _rightSourcePath = path;
                HasRightRun = true;
            }

            TraceNote = "";
            HasTraceNote = false;
            RefreshPair();
        }
        catch (Exception ex)
        {
            FailSide(left, ex.Message);
        }
    }

    private void SetTraceSide(bool left, string path, RuntimeRunTrace trace)
    {
        var label = string.IsNullOrWhiteSpace(trace.Label)
            ? Path.GetFileNameWithoutExtension(path)
            : trace.Label;

        if (left)
        {
            LeftRun = null;
            _leftTrace = trace;
            LeftRunName = label;
            _leftSourcePath = path;
            HasLeftRun = true;
        }
        else
        {
            RightRun = null;
            _rightTrace = trace;
            RightRunName = label;
            _rightSourcePath = path;
            HasRightRun = true;
        }

        TraceNote = "";
        HasTraceNote = false;
        RefreshPair();
    }

    private void FailSide(bool left, string message)
    {
        if (left)
        {
            LeftRun = null;
            _leftTrace = null;
            LeftRunName = "Could not read";
            HasLeftRun = false;
            _leftSourcePath = null;
        }
        else
        {
            RightRun = null;
            _rightTrace = null;
            RightRunName = "Could not read";
            HasRightRun = false;
            _rightSourcePath = null;
        }

        TraceNote = message;
        HasTraceNote = true;
        RefreshPair();
    }

    private void RefreshPair()
    {
        var mixed = (_leftTrace != null && RightRun != null) || (_rightTrace != null && LeftRun != null);
        if (mixed)
        {
            ShowTraceReview = false;
            TraceReview = null;
            HasBothRuns = false;
            IsCompareMode = false;
            CanonicalDeltas = [];
            AutoSummary = "";
            TraceNote = "One side is a geometry run and the other is an inference trace. Load two of the same kind.";
            HasTraceNote = true;
            SyncChrome();
            return;
        }

        if (_leftTrace != null && _rightTrace != null)
        {
            PublishTraceReview();
            return;
        }

        ShowTraceReview = false;
        TraceReview = null;
        UpdateComparisonState();
    }

    private void PublishTraceReview()
    {
        var left = _leftTrace!;
        var right = _rightTrace!;
        var comparison = new RunTraceComparer().Compare(
            left,
            right,
            ComparisonIntent.TfrtOptimization(LeftRunName, RightRunName));
        var review = TraceReviewBuilder.Build(
            left,
            right,
            comparison,
            _leftSourcePath ?? "",
            _rightSourcePath ?? "");

        TraceReview = review;
        ShowTraceReview = true;
        HasBothRuns = true;
        IsCompareMode = true;
        CanonicalDeltas = [];
        DeltaCount = review.FiredSymbols.Count;
        AutoSummary = review.Verdict;
        ComparisonSummary = review.Caption;
        InterpretationVerdict = review.Verdict;
        LeftDescription = DescribeSide(review.LeftLabel, review, review.LeftP50, review.LeftP95, review.LeftValues.Count);
        RightDescription = DescribeSide(review.RightLabel, review, review.RightP50, review.RightP95, review.RightValues.Count);
        FrameworkName = "TFRT";
        PresetName = "TFRT Runtime";
        HasPreset = true;
        IsLeftDominant = null;
        IsRightDominant = null;
        TryRecordTrace(review);
        SyncChrome();
    }

    private static string DescribeSide(string label, TraceReview review, double? p50, double? p95, int count)
    {
        var mid = p50 is double median ? median.ToString("0.###", CultureInfo.InvariantCulture) : "none";
        var tail = p95 is double high ? high.ToString("0.###", CultureInfo.InvariantCulture) : "none";
        return $"{label} · {review.Signal} · {count} samples · p50 {mid} {review.Unit} · p95 {tail} {review.Unit}";
    }

    private void TryRecordTrace(TraceReview review)
    {
        try
        {
            ComparisonLog.Record(new ComparisonLogEntry
            {
                Kind = "trace",
                LeftName = review.LeftLabel,
                RightName = review.RightLabel,
                LeftPath = review.LeftPath,
                RightPath = review.RightPath,
                Alignment = review.Caption,
                DeltasFired = review.FiredSymbols.ToList()
            }, ComparisonLog.DefaultDirectory);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Comparison log: {ex.Message}");
        }
    }

    private void SyncChrome()
    {
        ShowGeometryChrome = HasBothRuns && !ShowTraceReview;
        ShowSideBySide = ShowGeometryChrome && !IsOverlayMode;
        ShowOverlay = ShowGeometryChrome && IsOverlayMode;
    }

    /// <summary>
    /// Write the full-run review. Playback ticks recompute the playhead and do not call this.
    /// </summary>
    private void TryRecordFinishedComparison()
    {
        if (!HasBothRuns || LeftRun == null || RightRun == null)
            return;

        try
        {
            var fullRun = CanonicalDeltaService.ComputeDeltas(
                LeftRun, RightRun, SelectedAlignment, currentTime: 1.0);

            ComparisonLog.Record(new ComparisonLogEntry
            {
                Kind = IsDemoMode ? "example" : "compare",
                LeftName = LeftRunName,
                RightName = RightRunName,
                LeftPath = _leftSourcePath,
                RightPath = _rightSourcePath,
                LeftRunId = LeftRun.Metadata?.RunId,
                RightRunId = RightRun.Metadata?.RunId,
                Alignment = SelectedAlignment.ToString(),
                DeltasFired = ComparisonLog.SymbolsFor(fullRun).ToList()
            }, ComparisonLog.DefaultDirectory);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Comparison log: {ex.Message}");
        }
    }

    private static void TryRecordBundle(LoadedBundle bundle)
    {
        try
        {
            ComparisonLog.Record(new ComparisonLogEntry
            {
                Kind = "bundle",
                LeftName = Path.GetFileNameWithoutExtension(bundle.FilePath),
                BundlePath = bundle.FilePath,
                BundleHash = bundle.BundleHash,
                Alignment = bundle.Repro?.AlignmentMode ?? "",
                DeltasFired = ComparisonLog.SymbolsFor(bundle.Deltas).ToList()
            }, ComparisonLog.DefaultDirectory);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Comparison log: {ex.Message}");
        }
    }

    private void UpdateComparisonState()
    {
        // Phase 5.2: Preserve focus state before mode change
        var wasCompareMode = IsCompareMode;
        PreserveFocusState();
        
        HasBothRuns = HasLeftRun && HasRightRun;
        IsCompareMode = HasBothRuns;

        if (HasBothRuns)
        {
            GenerateComparisonSummary();
            
            // Phase 3: Compute canonical deltas
            UpdateCanonicalDeltas();
            TryRecordFinishedComparison();
            
            // Update alignment description
            var anchors = TemporalAlignmentService.GetAnchors(LeftRun, RightRun, SelectedAlignment);
            AlignmentDescription = anchors.AnchorDescription;
            
            // Phase 5.2: Restore focus state after mode transition
            RestoreFocusState();
        }
        else
        {
            // Clear Phase 3 state when not in compare mode
            CanonicalDeltas = [];
            AutoSummary = "";
            HighlightedDeltaId = null;
            HighlightedAnchorTime = -1;
        }

        SyncChrome();
    }

    private void GenerateComparisonSummary()
    {
        if (LeftRun == null || RightRun == null) return;

        // Invariant check: both runs must be valid for comparison
        if (!InvariantGuard.AssertCompareRunsValid(LeftRun, RightRun, "ComparisonViewModel.GenerateComparisonSummary"))
        {
            ComparisonSummary = "Error: One or both runs have invalid data.";
            InterpretationVerdict = "Cannot compare runs with missing trajectory data.";
            return;
        }

        // Invariant check: data consistency
        InvariantGuard.AssertDataConsistentLengths(LeftRun, "ComparisonViewModel.LeftRun");
        InvariantGuard.AssertDataConsistentLengths(RightRun, "ComparisonViewModel.RightRun");

        var leftCondition = LeftRun.Metadata?.Condition ?? "Unknown";
        var rightCondition = RightRun.Metadata?.Condition ?? "Unknown";
        var leftTier = LeftRun.Metadata?.ConscienceTier ?? "?";
        var rightTier = RightRun.Metadata?.ConscienceTier ?? "?";
        var leftFailures = LeftRun.Failures?.Count ?? 0;
        var rightFailures = RightRun.Failures?.Count ?? 0;

        // Compute final eigenvalue comparison with null safety
        var leftFinalEigen = LeftRun.Geometry?.Eigenvalues?.LastOrDefault()?.Values;
        var rightFinalEigen = RightRun.Geometry?.Eigenvalues?.LastOrDefault()?.Values;

        var leftSum = leftFinalEigen?.Sum() ?? 0;
        var rightSum = rightFinalEigen?.Sum() ?? 0;

        var leftFirstFactor = (leftFinalEigen?.Count > 0 && leftSum > 0.001)
            ? leftFinalEigen[0] / leftSum
            : 0;
        var rightFirstFactor = (rightFinalEigen?.Count > 0 && rightSum > 0.001)
            ? rightFinalEigen[0] / rightSum
            : 0;

        ComparisonSummary = $"Left: {leftCondition} ({leftTier}, {leftFailures} failures, λ₁={leftFirstFactor:P0})\n" +
                          $"Right: {rightCondition} ({rightTier}, {rightFailures} failures, λ₁={rightFirstFactor:P0})";

        // Generate descriptions for interpretation strip
        LeftDescription = GenerateRunDescription(LeftRun, leftFirstFactor, "A");
        RightDescription = GenerateRunDescription(RightRun, rightFirstFactor, "B");
        InterpretationVerdict = GenerateVerdict(leftFirstFactor, rightFirstFactor, leftFailures, rightFailures);

        // Determine visual dominance for canvas dimming - use centralized threshold
        var delta = rightFirstFactor - leftFirstFactor;
        if (Math.Abs(delta) < ConsistencyCheckService.FirstFactorDeltaThreshold)
        {
            // Too close to call - no dimming
            IsLeftDominant = null;
            IsRightDominant = null;
        }
        else if (delta > 0)
        {
            // Right (Path B) is dominant
            IsLeftDominant = false;
            IsRightDominant = true;
        }
        else
        {
            // Left (Path A) is dominant
            IsLeftDominant = true;
            IsRightDominant = false;
        }
    }

    private static string GenerateRunDescription(GeometryRun run, double firstFactor, string pathLabel)
    {
        var tier = run.Metadata.ConscienceTier;
        var failures = run.Failures.Count;

        // Use centralized interpretation for consistency
        var interpretation = ConsistencyCheckService.GetEigenInterpretation(firstFactor);

        return interpretation switch
        {
            EigenInterpretation.StrongSharedAxis =>
                $"Path {pathLabel}: Strong eigenvalue dominance ({firstFactor:P0}). Evaluators share structure. {tier} tier with {failures} failure(s).",
            EigenInterpretation.ModerateUnification =>
                $"Path {pathLabel}: Mixed eigenvalue distribution ({firstFactor:P0}). Partial structure sharing. {tier} tier with {failures} failure(s).",
            _ =>
                $"Path {pathLabel}: Distributed eigenvalues ({firstFactor:P0}). Evaluators are orthogonal. {tier} tier with {failures} failure(s)."
        };
    }

    private static string GenerateVerdict(double leftFirst, double rightFirst, int leftFail, int rightFail)
    {
        var delta = rightFirst - leftFirst;

        // Use centralized threshold for consistency
        if (Math.Abs(delta) < ConsistencyCheckService.FirstFactorDeltaThreshold)
            return "Both paths show similar eigenvalue structure. No clear convergence advantage.";
        else if (delta > 0.2)
            return $"Path B shows stronger convergence (Δλ₁ = +{delta:P0}). Correlated professors enable unified representation.";
        else if (delta > 0)
            return $"Path B shows slightly better convergence (Δλ₁ = +{delta:P0}).";
        else if (delta < -0.2)
            return $"Path A maintains diversity (Δλ₁ = {delta:P0}). Orthogonal professors prevent collapse.";
        else
            return $"Path A shows slightly more distributed learning (Δλ₁ = {delta:P0}).";
    }

    // Helper methods for getting data at time t
    private static TrajectoryTimestep? GetTrajectoryAtTime(GeometryRun? run, double t)
    {
        try
        {
            if (run?.Trajectory?.Timesteps is not { Count: > 0 } steps)
                return null;

            var idx = (int)(t * Math.Max(0, steps.Count - 1));
            idx = Math.Clamp(idx, 0, steps.Count - 1);
            return steps[idx];
        }
        catch
        {
            return null;
        }
    }

    private static ScalarTimestep? GetScalarsAtTime(GeometryRun? run, double t)
    {
        try
        {
            if (run?.Scalars?.Values is not { Count: > 0 } values)
                return null;

            var idx = (int)(t * Math.Max(0, values.Count - 1));
            idx = Math.Clamp(idx, 0, values.Count - 1);
            return values[idx];
        }
        catch
        {
            return null;
        }
    }

    private static EigenTimestep? GetEigenvaluesAtTime(GeometryRun? run, double t)
    {
        try
        {
            if (run?.Geometry?.Eigenvalues is not { Count: > 0 } values)
                return null;

            var idx = (int)(t * Math.Max(0, values.Count - 1));
            idx = Math.Clamp(idx, 0, values.Count - 1);
            return values[idx];
        }
        catch
        {
            return null;
        }
    }

    public IEnumerable<TrajectoryTimestep> GetLeftTrajectoryUpToTime(double t)
    {
        if (LeftRun?.Trajectory?.Timesteps is not { Count: > 0 } steps)
            yield break;

        var maxIdx = (int)(t * Math.Max(0, steps.Count - 1));
        maxIdx = Math.Clamp(maxIdx, 0, steps.Count - 1);

        for (int i = 0; i <= maxIdx && i < steps.Count; i++)
        {
            yield return steps[i];
        }
    }

    public IEnumerable<TrajectoryTimestep> GetRightTrajectoryUpToTime(double t)
    {
        if (RightRun?.Trajectory?.Timesteps is not { Count: > 0 } steps)
            yield break;

        var maxIdx = (int)(t * Math.Max(0, steps.Count - 1));
        maxIdx = Math.Clamp(maxIdx, 0, steps.Count - 1);

        for (int i = 0; i <= maxIdx && i < steps.Count; i++)
        {
            yield return steps[i];
        }
    }

    /// <summary>
    /// Compute comparison metrics at current time.
    /// </summary>
    public ComparisonMetrics GetCurrentMetrics()
    {
        var leftEigen = LeftCurrentEigenvalues?.Values ?? [];
        var rightEigen = RightCurrentEigenvalues?.Values ?? [];

        var leftFirstFactor = leftEigen.Count > 0 ? leftEigen[0] / Math.Max(0.001, leftEigen.Sum()) : 0;
        var rightFirstFactor = rightEigen.Count > 0 ? rightEigen[0] / Math.Max(0.001, rightEigen.Sum()) : 0;

        var leftEffDim = LeftCurrentTrajectory?.EffectiveDim ?? 0;
        var rightEffDim = RightCurrentTrajectory?.EffectiveDim ?? 0;

        var leftCurvature = LeftCurrentTrajectory?.Curvature ?? 0;
        var rightCurvature = RightCurrentTrajectory?.Curvature ?? 0;

        return new ComparisonMetrics
        {
            LeftFirstFactorVariance = leftFirstFactor,
            RightFirstFactorVariance = rightFirstFactor,
            LeftEffectiveDim = leftEffDim,
            RightEffectiveDim = rightEffDim,
            LeftCurvature = leftCurvature,
            RightCurvature = rightCurvature,
            FirstFactorDelta = rightFirstFactor - leftFirstFactor,
            EffectiveDimDelta = leftEffDim - rightEffDim, // Lower is more unified
        };
    }
}

public record ComparisonMetrics
{
    public double LeftFirstFactorVariance { get; init; }
    public double RightFirstFactorVariance { get; init; }
    public double LeftEffectiveDim { get; init; }
    public double RightEffectiveDim { get; init; }
    public double LeftCurvature { get; init; }
    public double RightCurvature { get; init; }
    public double FirstFactorDelta { get; init; }
    public double EffectiveDimDelta { get; init; }
}
