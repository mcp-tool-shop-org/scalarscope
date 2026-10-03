using ScalarScope.Services;
using ScalarScope.ViewModels;

namespace ScalarScope.Views;

public partial class ComparisonPage : ContentPage, IQueryAttributable
{
    // Use the shared comparison instance from App
    private ComparisonViewModel ViewModel => App.Comparison;

    private int _openSerial;

    public ComparisonPage()
    {
        InitializeComponent();
        BindingContext = App.Comparison;
        
        // Wire up Phase 4 insight events
        SetupInsightHandlers();
        
        // Wire up Phase 7.2 bundle import events
        SetupBundleHandlers();
    }

    /// <summary>
    /// Phase 7.2: Open Bundle file picker and import.
    /// </summary>
    private async void OnOpenBundleClicked(object? sender, EventArgs e)
    {
        await ErrorBoundary.TrySafeAsync(async () =>
        {
            var customFileType = new FilePickerFileType(new Dictionary<DevicePlatform, IEnumerable<string>>
            {
                { DevicePlatform.WinUI, new[] { ".scbundle" } },
                { DevicePlatform.macOS, new[] { "scbundle" } },
                { DevicePlatform.iOS, new[] { "public.data" } },
                { DevicePlatform.Android, new[] { "application/octet-stream" } }
            });

            var options = new PickOptions
            {
                PickerTitle = "Open Comparison Bundle",
                FileTypes = customFileType
            };

            var result = await FilePicker.PickAsync(options);
            if (result == null) return;

            await LoadBundleFromPathAsync(result.FullPath);
        }, "Bundle open");
    }

    /// <summary>
    /// Shell delivers the same request Home stored on the comparison.
    /// </summary>
    public void ApplyQueryAttributes(IDictionary<string, object> query)
    {
        if (query.TryGetValue("bundle", out var bundleValue) && bundleValue != null)
        {
            var path = bundleValue.ToString();
            if (!string.IsNullOrWhiteSpace(path))
            {
                ViewModel.RequestBundleOpen(DecodeBundlePath(path));
                ScheduleOpenRequest();
                return;
            }
        }

        if (query.TryGetValue("demo", out var demoValue) && IsDemoQuery(demoValue))
        {
            ViewModel.RequestDemoOpen();
            ScheduleOpenRequest();
        }
    }

    protected override void OnAppearing()
    {
        base.OnAppearing();
        // Update delta label when time changes
        ViewModel.Player.TimeChanged += UpdateDeltaLabel;
        // Trigger initial update if runs are loaded
        UpdateDeltaLabel();

        // Re-subscribe to MessagingCenter (may have been unsubscribed)
        MessagingCenter.Subscribe<HelpPage, string>(this, "HighlightDelta", OnHighlightDeltaRequested);

        ScheduleOpenRequest();
    }

    private void ScheduleOpenRequest()
    {
        var serial = ++_openSerial;
        MainThread.BeginInvokeOnMainThread(async () =>
        {
            if (serial != _openSerial)
                return;

            if (!ViewModel.TryTakeOpenRequest(out var bundlePath, out var demo, out var leftPath, out var rightPath))
                return;

            if (!string.IsNullOrEmpty(bundlePath))
                await LoadBundleFromPathAsync(bundlePath);
            else if (!string.IsNullOrEmpty(leftPath) && !string.IsNullOrEmpty(rightPath))
                await LoadRunPairAsync(leftPath, rightPath);
            else if (demo)
                await LoadDemoFromQueryAsync();
        });
    }

    private async Task LoadBundleFromPathAsync(string path)
    {
        openBundleButton.IsEnabled = false;
        openBundleButton.Text = "Loading...";

        try
        {
            if (!File.Exists(path))
            {
                await DisplayAlert("Import Failed", $"File not found:\n{path}", "OK");
                return;
            }

            var importResult = await BundleImportService.Instance.ImportAsync(path);

            if (importResult.Success && importResult.LoadedBundle != null)
            {
                await HydrateBundleAsync(importResult.LoadedBundle);
            }
            else
            {
                var errorMessage = importResult.ErrorMessage ?? "Failed to import bundle";
                if (importResult.ErrorExplanation != null)
                {
                    errorMessage += $"\n\n{importResult.ErrorExplanation.Summary}";
                }

                await DisplayAlert("Import Failed", errorMessage, "OK");
            }
        }
        catch (Exception ex)
        {
            await DisplayAlert("Import Failed", ex.Message, "OK");
        }
        finally
        {
            openBundleButton.IsEnabled = true;
            openBundleButton.Text = "📦 Open Bundle...";
        }
    }

    private async Task LoadDemoFromQueryAsync()
    {
        openBundleButton.IsEnabled = false;

        try
        {
            var (pathA, pathB) = await DemoService.StartDemoAsync();
            if (pathA == null || pathB == null)
            {
                await DisplayAlert("Example unavailable", "The built-in example runs could not be loaded.", "OK");
                return;
            }

            ViewModel.LoadDemoRuns(pathA, pathB);
            await Task.Delay(500);
            if (!ViewModel.Player.IsPlaying)
            {
                ViewModel.Player.PlayPauseCommand.Execute(null);
            }
        }
        catch (Exception ex)
        {
            await DisplayAlert("Example unavailable", ex.Message, "OK");
        }
        finally
        {
            openBundleButton.IsEnabled = true;
            openBundleButton.Text = "📦 Open Bundle...";
        }
    }

    private static string DecodeBundlePath(string value)
    {
        return value.Contains('%', StringComparison.Ordinal)
            ? Uri.UnescapeDataString(value)
            : value;
    }

    private static bool IsDemoQuery(object? value)
    {
        return value is bool flag
            ? flag
            : string.Equals(value?.ToString(), "true", StringComparison.OrdinalIgnoreCase);
    }

    /// <summary>
    /// Phase 7.2: Hydrate UI with bundle data.
    /// </summary>
    private async Task HydrateBundleAsync(LoadedBundle bundle)
    {
        // Set review mode in ViewModel
        ViewModel.EnterReviewMode(bundle);
        
        // Update delta zone with bundled deltas
        deltaZone.Deltas = bundle.Deltas;
        
        // Update insights tray with bundled insights
        if (bundle.Insights != null)
        {
            insightsTray.SetInsights(bundle.Insights);
        }
        
        // Show review mode banner
        await DisplayAlert(
            "Bundle Loaded",
            $"Loaded {bundle.Manifest.Profile} bundle\n" +
            $"Created: {bundle.Manifest.CreatedAt:g}\n" +
            $"Deltas: {bundle.Deltas.Count}\n" +
            $"Badge: {bundle.ReproducibilityBadge}",
            "OK");
    }

    private async Task LoadRunPairAsync(string leftPath, string rightPath)
    {
        openBundleButton.IsEnabled = false;
        openBundleButton.Text = "Loading...";

        try
        {
            await ViewModel.LoadLeftFromFileAsync(leftPath);
            await ViewModel.LoadRightFromFileAsync(rightPath);
            if (!ViewModel.HasBothRuns)
            {
                await DisplayAlert("Could not reopen", "One of the runs failed to load.", "OK");
            }
        }
        catch (Exception ex)
        {
            await DisplayAlert("Could not reopen", ex.Message, "OK");
        }
        finally
        {
            openBundleButton.IsEnabled = true;
            openBundleButton.Text = "📦 Open Bundle...";
        }
    }

    private void SetupBundleHandlers()
    {
        // Listen for bundle unload
        BundleImportService.Instance.BundleUnloaded += (s, e) =>
        {
            if (ViewModel.IsReviewMode)
            {
                ViewModel.ExitReviewMode();
            }
        };
        
        // Wire up review mode banner exit
        reviewModeBanner.ExitRequested += OnExitReviewModeRequested;

        deltaZone.ExportBundleRequested += OnExportBundleRequested;
        deltaZone.NotifyExportHandler();
    }

    private async void OnExportBundleRequested()
    {
        if (ViewModel.LeftRun == null || ViewModel.RightRun == null || !ViewModel.CanExportBundle)
            return;

        try
        {
            var result = CanonicalDeltaService.ComputeDeltasWithAlignment(
                ViewModel.LeftRun,
                ViewModel.RightRun,
                ViewModel.SelectedAlignment,
                currentTime: 1.0);
            bundleExportPanel.Initialize(
                result,
                ViewModel.LeftSourcePath,
                ViewModel.RightSourcePath,
                insights: insightsTray.CopyInsights());
            bundleExportPanel.Show();
        }
        catch (Exception ex)
        {
            await DisplayAlert("Export unavailable", ex.Message, "OK");
        }
    }

    /// <summary>
    /// Phase 7.2: Handle exit review mode request.
    /// </summary>
    private void OnExitReviewModeRequested(object? sender, EventArgs e)
    {
        ViewModel.ExitReviewMode();
        
        // Clear insights tray bundle mode
        insightsTray.ClearBundleInsights();
    }

    private void OnHighlightDeltaRequested(HelpPage sender, string deltaType)
    {
        // Map deltaType string to delta ID
        var deltaId = deltaType switch
        {
            "failure" => "delta_f",
            "convergence" => "delta_tc",
            "dominance" => "delta_td",
            "alignment" => "delta_a",
            "oscillation" => "delta_o",
            _ => null
        };

        if (deltaId != null)
        {
            // Highlight the delta in DeltaZone
            ViewModel.HighlightedDeltaId = deltaId;
            
            // Expand the Why? panel for this delta
            var delta = ViewModel.CanonicalDeltas?.FirstOrDefault(d => d.Id == deltaId);
            if (delta != null)
            {
                deltaZone.SelectedDelta = delta;
                deltaZone.IsWhyPanelExpanded = true;
            }
        }
    }

    private void SetupInsightHandlers()
    {
        // DeltaZone "Show me" navigates to anchor
        deltaZone.ShowMeRequested += OnShowMeRequested;
        
        // InsightsTray "Show me" navigates to insight source
        insightsTray.ShowMeRequested += OnInsightShowMeRequested;
        
        // Subscribe to deltas changes to publish insights and set idle calming
        ViewModel.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(ViewModel.CanonicalDeltas))
            {
                PublishDeltaInsights();
                
                // Phase 5.1: Slow down demo animations when deltas are visible
                var hasDeltasVisible = ViewModel.CanonicalDeltas?.Any(d => d.Status == DeltaStatus.Present) ?? false;
                DemoStateService.Instance.SetIdleCalming(hasDeltasVisible);
            }
        };
    }

    private async void OnShowMeRequested(CanonicalDelta delta)
    {
        try
        {
            var startTime = ViewModel.Player.Time;
            var targetTime = double.IsFinite(delta.VisualAnchorTime)
                ? Math.Clamp(delta.VisualAnchorTime, 0, 1)
                : startTime;

            await TransitionService.NavigateToAnchor(
                targetTime: targetTime,
                highlightElementId: delta.Id,
                onSeek: t => SeekToward(startTime, targetTime, t),
                onHighlight: id => ViewModel.HighlightedDeltaId = id
            );
        }
        catch (Exception ex)
        {
            await DisplayAlert("Could not show that moment", ex.Message, "OK");
        }
    }

    private async void OnInsightShowMeRequested(Models.InsightEvent insight)
    {
        try
        {
            var target = ShellDestinations.Name(insight.TargetView);
            if (!string.IsNullOrEmpty(target) && target != "compare")
            {
                if (!ShellDestinations.IsTab(target))
                {
                    await DisplayAlert(
                        "That view is not open",
                        $"Compare does not have a {insight.TargetView} page. Showing the moment here.",
                        "OK");
                }
                else
                {
                    try
                    {
                        await Shell.Current.GoToAsync($"//{target}");
                        return;
                    }
                    catch (Exception ex)
                    {
                        await DisplayAlert("Could not open that page", ex.Message, "OK");
                        return;
                    }
                }
            }

            var startTime = ViewModel.Player.Time;
            var anchor = insight.AnchorTime ?? startTime;
            var targetTime = double.IsFinite(anchor) ? Math.Clamp(anchor, 0, 1) : startTime;
            await TransitionService.NavigateToAnchor(
                targetTime: targetTime,
                highlightElementId: insight.DeltaId,
                onSeek: t => SeekToward(startTime, targetTime, t),
                onHighlight: id =>
                {
                    if (id != null) ViewModel.HighlightedDeltaId = id;
                }
            );
        }
        catch (Exception ex)
        {
            await DisplayAlert("Could not show that moment", ex.Message, "OK");
        }
    }

    /// <summary>
    /// NavigateToAnchor reports easedProgress * targetTime. Recover the ease and lerp, including backward.
    /// An anchor at 0 is valid. The reported value is 0 for that anchor, so land on it instead of dividing.
    /// </summary>
    private void SeekToward(double startTime, double targetTime, double reported)
    {
        if (!double.IsFinite(targetTime))
            targetTime = 0;
        targetTime = Math.Clamp(targetTime, 0, 1);
        if (!double.IsFinite(startTime))
            startTime = 0;
        startTime = Math.Clamp(startTime, 0, 1);

        double eased;
        if (targetTime <= double.Epsilon)
        {
            eased = 1;
        }
        else
        {
            eased = reported / targetTime;
            if (!double.IsFinite(eased))
                eased = 1;
            eased = Math.Clamp(eased, 0, 1);
        }

        var next = startTime + ((targetTime - startTime) * eased);
        if (!double.IsFinite(next))
            next = targetTime;
        ViewModel.Player.Time = Math.Clamp(next, 0, 1);
    }

    private void PublishDeltaInsights()
    {
        if (ViewModel.CanonicalDeltas == null) return;

        foreach (var delta in ViewModel.CanonicalDeltas)
        {
            if (delta.Status == DeltaStatus.Present)
            {
                // Determine trigger type based on delta type
                var triggerType = DetermineTriggerType(delta);
                InsightFeedService.Instance.PublishDelta(delta, triggerType);
            }
        }
    }

    private static string? DetermineTriggerType(CanonicalDelta delta)
    {
        return delta.Id switch
        {
            "delta_td" => delta.DominanceRatioK > 0.5 ? "sustained" : "recurrence",
            "delta_a" => "persistence_weighted",
            "delta_o" => "area_episode",
            "delta_tc" => delta.ConvergenceConfidence.HasValue ? "step_difference" : "one_run_converged",
            "delta_f" => (delta.FailedA == true || delta.FailedB == true) ? "event" : "proxy",
            _ => null
        };
    }

    private void UpdateDeltaLabel()
    {
        if (!ViewModel.HasBothRuns) return;

        var metrics = ViewModel.GetCurrentMetrics();
        var deltaSign = metrics.FirstFactorDelta >= 0 ? "+" : "";

        deltaLabel.Text = $"Δλ₁: {deltaSign}{metrics.FirstFactorDelta:P0}";

        // Color based on whether Path B shows improvement
        deltaLabel.TextColor = metrics.FirstFactorDelta > 0.1
            ? Color.FromArgb("#4ecdc4")  // Green - Path B better
            : metrics.FirstFactorDelta < -0.1
                ? Color.FromArgb("#ff6b6b")  // Red - Path A better
                : Color.FromArgb("#ffd93d"); // Yellow - similar
    }

    protected override void OnDisappearing()
    {
        base.OnDisappearing();
        ViewModel.Player.TimeChanged -= UpdateDeltaLabel;
        
        // Unsubscribe from MessagingCenter to prevent memory leaks
        MessagingCenter.Unsubscribe<HelpPage, string>(this, "HighlightDelta");
    }
}
