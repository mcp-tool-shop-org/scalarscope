using System.Diagnostics;
using System.Numerics;
using Microsoft.Extensions.Logging;
using Microsoft.Maui.ApplicationModel;
using ScalarScope.Models;
using ScalarScope.Services;

namespace ScalarScope.SoakTests;

/// <summary>
/// Soak test runner for ScalarScope.
/// Runs the application through various scenarios to prove stability.
/// </summary>
public class SoakTestRunner
{
    private readonly ILogger _logger;
    private readonly SoakTestConfig _config;
    private readonly List<SoakTestResult> _results = new();
    private long _startMemory;
    private readonly Stopwatch _stopwatch = new();
    private const int PlaybackBudgetMs = 1000;
    private const int ExportBudgetMs = 5000;
    private const int ToggleBudgetMs = 1000;

    public SoakTestRunner(ILogger logger, SoakTestConfig config)
    {
        _logger = logger;
        _config = config;
    }

    public async Task<SoakTestReport> RunAsync()
    {
        _logger.LogInformation("Starting soak test with duration: {Duration} minutes", _config.DurationMinutes);

        _startMemory = GC.GetTotalMemory(true);
        _stopwatch.Start();

        var cts = new CancellationTokenSource(TimeSpan.FromMinutes(_config.DurationMinutes));
        var iteration = 0;

        try
        {
            while (!cts.Token.IsCancellationRequested)
            {
                iteration++;
                _logger.LogInformation("Starting iteration {Iteration}", iteration);

                await RunIterationAsync(iteration, cts.Token);

                // Brief pause between iterations
                await Task.Delay(TimeSpan.FromSeconds(_config.PauseBetweenIterationsSeconds), cts.Token);
            }
        }
        catch (OperationCanceledException)
        {
            _logger.LogInformation("Soak test completed normally after {Elapsed}", _stopwatch.Elapsed);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Soak test failed with exception");
            _results.Add(new SoakTestResult
            {
                TestName = "UnhandledException",
                Passed = false,
                Message = ex.Message,
                Timestamp = DateTime.UtcNow
            });
        }

        _stopwatch.Stop();

        var endMemory = GC.GetTotalMemory(true);
        var report = GenerateReport(endMemory, iteration);

        return report;
    }

    private async Task RunIterationAsync(int iteration, CancellationToken ct)
    {
        // Test 1: Memory stability check
        await TestMemoryStabilityAsync(iteration, ct);

        await TestPlaybackAsync(iteration, ct);
        await TestExportAsync(iteration, ct);
        await TestThemeToggleAsync(iteration, ct);
        await TestAnnotationToggleAsync(iteration, ct);
    }

    private async Task TestMemoryStabilityAsync(int iteration, CancellationToken ct)
    {
        var currentMemory = GC.GetTotalMemory(false);
        var memoryGrowthMB = (currentMemory - _startMemory) / (1024.0 * 1024.0);

        var passed = memoryGrowthMB < _config.MaxMemoryGrowthMB;

        _results.Add(new SoakTestResult
        {
            TestName = $"MemoryStability_Iteration{iteration}",
            Passed = passed,
            Message = $"Memory growth: {memoryGrowthMB:F2} MB (limit: {_config.MaxMemoryGrowthMB} MB)",
            Timestamp = DateTime.UtcNow,
            MetricValue = memoryGrowthMB
        });

        if (!passed)
        {
            _logger.LogWarning("Memory growth exceeded threshold: {Growth:F2} MB", memoryGrowthMB);
        }

        await Task.Delay(100, ct);
    }

    private async Task TestPlaybackAsync(int iteration, CancellationToken ct)
    {
        var sw = Stopwatch.StartNew();
        var passed = false;
        var message = "PlaybackController was not exercised";

        try
        {
            using var playback = new PlaybackController { EnableParticles = false, EnableMotionBlur = false };
            var points = new List<Vector2>();
            var times = new List<double>();
            for (var i = 0; i < 5; i++)
            {
                points.Add(new Vector2(i, 0));
                times.Add(i * 0.25);
            }

            playback.SetTrajectory(points, times);
            playback.StepForward();
            var stepped = playback.GetCurrentFrameIndex() > 0 || playback.CurrentTime > 0;
            playback.Play();
            await Task.Delay(40, ct);
            playback.Stop();
            sw.Stop();

            passed = stepped
                && !playback.IsPlaying
                && playback.CurrentTime == 0
                && sw.ElapsedMilliseconds < PlaybackBudgetMs;
            message = stepped
                ? $"PlaybackController step/play/stop in {sw.ElapsedMilliseconds} ms (budget {PlaybackBudgetMs} ms)"
                : "PlaybackController.StepForward did not advance the trajectory";
        }
        catch (Exception ex) when (ex is not OperationCanceledException)
        {
            sw.Stop();
            message = $"PlaybackController failed: {ex.Message}";
        }

        _results.Add(new SoakTestResult
        {
            TestName = $"Playback_Iteration{iteration}",
            Passed = passed,
            Message = message,
            Timestamp = DateTime.UtcNow,
            MetricValue = sw.ElapsedMilliseconds
        });
    }

    private async Task TestExportAsync(int iteration, CancellationToken ct)
    {
        var sw = Stopwatch.StartNew();
        var passed = false;
        var message = "ExportService was not exercised";
        var outputPath = Path.Combine(Path.GetTempPath(), "scalarscope-soak", $"export-{iteration}-{Guid.NewGuid():N}.png");

        try
        {
            var exporter = new ExportService();
            var rejected = exporter.ValidateForExport(null);
            var run = new GeometryRun
            {
                Metadata = new RunMetadata { RunId = "soak-export" },
                Trajectory = new Trajectory
                {
                    Timesteps =
                    [
                        new TrajectoryTimestep { T = 0, State2D = [0, 0], Velocity = [0.1, 0], Curvature = 0 },
                        new TrajectoryTimestep { T = 0.5, State2D = [0.2, 0.1], Velocity = [0.1, 0], Curvature = 0 },
                        new TrajectoryTimestep { T = 1, State2D = [0.4, 0.2], Velocity = [0.05, 0], Curvature = 0 }
                    ]
                }
            };
            var accepted = exporter.ValidateForExport(run);
            Directory.CreateDirectory(Path.GetDirectoryName(outputPath)!);
            var exported = await exporter.ExportStillAsync(run, 1.0, outputPath, new ExportOptions
            {
                Width = 320,
                Height = 240,
                ShowProfessors = false,
                ShowMetrics = false,
                ShowEigenvalues = false,
                IncludeLegend = false,
                IncludeWatermark = false,
                ShowAnnotations = false
            }, ct);
            sw.Stop();

            var bytes = exported.IsSuccess && File.Exists(outputPath) ? new FileInfo(outputPath).Length : 0;
            var withinBudget = sw.ElapsedMilliseconds < ExportBudgetMs;
            passed = !rejected.IsValid && accepted.IsValid && exported.IsSuccess && bytes > 0 && withinBudget;
            message = passed
                ? $"ExportService wrote {bytes} bytes in {sw.ElapsedMilliseconds} ms (budget {ExportBudgetMs} ms)"
                : $"ExportService rejectedNull={rejected.IsValid} accepted={accepted.IsValid} success={exported.IsSuccess} bytes={bytes} elapsed={sw.ElapsedMilliseconds} budget={ExportBudgetMs} {exported.ErrorMessage}";
        }
        catch (Exception ex) when (ex is not OperationCanceledException)
        {
            sw.Stop();
            message = $"ExportService failed: {ex.Message}";
        }
        finally
        {
            if (File.Exists(outputPath))
                File.Delete(outputPath);
        }

        _results.Add(new SoakTestResult
        {
            TestName = $"Export_Iteration{iteration}",
            Passed = passed,
            Message = message,
            Timestamp = DateTime.UtcNow,
            MetricValue = sw.ElapsedMilliseconds
        });
    }

    private async Task TestThemeToggleAsync(int iteration, CancellationToken ct)
    {
        var sw = Stopwatch.StartNew();
        var passed = false;
        var message = "UserPreferencesService.SetTheme was not exercised";
        AppTheme? previous = null;

        try
        {
            previous = UserPreferencesService.GetTheme();
            var next = previous == AppTheme.Dark ? AppTheme.Light : AppTheme.Dark;
            UserPreferencesService.SetTheme(next);
            var readBack = UserPreferencesService.GetTheme();
            sw.Stop();
            passed = readBack == next && sw.ElapsedMilliseconds < ToggleBudgetMs;
            message = passed
                ? $"Theme toggled to {next} in {sw.ElapsedMilliseconds} ms (budget {ToggleBudgetMs} ms)"
                : $"Theme read-back was {readBack}, expected {next}, in {sw.ElapsedMilliseconds} ms";
        }
        catch (Exception ex) when (ex is not OperationCanceledException)
        {
            sw.Stop();
            message = $"Theme path failed: {ex.Message}";
        }
        finally
        {
            if (previous.HasValue)
            {
                try { UserPreferencesService.SetTheme(previous.Value); }
                catch { /* restore is best-effort */ }
            }
        }

        await Task.Delay(1, ct);
        _results.Add(new SoakTestResult
        {
            TestName = $"ThemeToggle_Iteration{iteration}",
            Passed = passed,
            Message = message,
            Timestamp = DateTime.UtcNow,
            MetricValue = sw.ElapsedMilliseconds
        });
    }

    private async Task TestAnnotationToggleAsync(int iteration, CancellationToken ct)
    {
        var sw = Stopwatch.StartNew();
        var passed = false;
        var message = "Annotation preferences were not exercised";
        AnnotationDensity? previous = null;

        try
        {
            DemoAnnotationService.ResetForNewDemo();
            var annotations = DemoAnnotationService.GetAllAnnotations();
            DemoAnnotationService.CheckTimeThreshold(0.5);
            previous = UserPreferencesService.GetAnnotationDensity();
            var next = previous == AnnotationDensity.Full ? AnnotationDensity.Minimal : AnnotationDensity.Full;
            UserPreferencesService.SetAnnotationDensity(next);
            var readBack = UserPreferencesService.GetAnnotationDensity();
            sw.Stop();
            passed = annotations.Count > 0 && readBack == next && sw.ElapsedMilliseconds < ToggleBudgetMs;
            message = passed
                ? $"Annotation density toggled to {next} across {annotations.Count} demo annotations in {sw.ElapsedMilliseconds} ms"
                : $"Annotation toggle failed: count={annotations.Count} readBack={readBack} expected={next} elapsed={sw.ElapsedMilliseconds}";
        }
        catch (Exception ex) when (ex is not OperationCanceledException)
        {
            sw.Stop();
            message = $"Annotation path failed: {ex.Message}";
        }
        finally
        {
            if (previous.HasValue)
            {
                try { UserPreferencesService.SetAnnotationDensity(previous.Value); }
                catch { /* restore is best-effort */ }
            }
        }

        await Task.Delay(1, ct);
        _results.Add(new SoakTestResult
        {
            TestName = $"AnnotationToggle_Iteration{iteration}",
            Passed = passed,
            Message = message,
            Timestamp = DateTime.UtcNow,
            MetricValue = sw.ElapsedMilliseconds
        });
    }

    private SoakTestReport GenerateReport(long endMemory, int iterations)
    {
        var totalTests = _results.Count;
        var passedTests = _results.Count(r => r.Passed);
        var failedTests = totalTests - passedTests;

        var memoryGrowthMB = (endMemory - _startMemory) / (1024.0 * 1024.0);

        var report = new SoakTestReport
        {
            StartTime = DateTime.UtcNow - _stopwatch.Elapsed,
            EndTime = DateTime.UtcNow,
            Duration = _stopwatch.Elapsed,
            TotalIterations = iterations,
            TotalTests = totalTests,
            PassedTests = passedTests,
            FailedTests = failedTests,
            StartMemoryMB = _startMemory / (1024.0 * 1024.0),
            EndMemoryMB = endMemory / (1024.0 * 1024.0),
            MemoryGrowthMB = memoryGrowthMB,
            Results = _results,
            OverallPassed = failedTests == 0 && memoryGrowthMB < _config.MaxMemoryGrowthMB
        };

        return report;
    }
}

public class SoakTestConfig
{
    public int DurationMinutes { get; set; } = 120; // 2 hours default
    public int PauseBetweenIterationsSeconds { get; set; } = 5;
    public double MaxMemoryGrowthMB { get; set; } = 100; // Allow up to 100 MB growth
}

public class SoakTestResult
{
    public required string TestName { get; set; }
    public bool Passed { get; set; }
    public string Message { get; set; } = "";
    public DateTime Timestamp { get; set; }
    public double MetricValue { get; set; }
}

public class SoakTestReport
{
    public DateTime StartTime { get; set; }
    public DateTime EndTime { get; set; }
    public TimeSpan Duration { get; set; }
    public int TotalIterations { get; set; }
    public int TotalTests { get; set; }
    public int PassedTests { get; set; }
    public int FailedTests { get; set; }
    public double StartMemoryMB { get; set; }
    public double EndMemoryMB { get; set; }
    public double MemoryGrowthMB { get; set; }
    public List<SoakTestResult> Results { get; set; } = new();
    public bool OverallPassed { get; set; }

    public void WriteToConsole()
    {
        Console.WriteLine();
        Console.WriteLine("═══════════════════════════════════════════════════════════════");
        Console.WriteLine("                    SOAK TEST REPORT                           ");
        Console.WriteLine("═══════════════════════════════════════════════════════════════");
        Console.WriteLine();
        Console.WriteLine($"  Duration:        {Duration:hh\\:mm\\:ss}");
        Console.WriteLine($"  Iterations:      {TotalIterations}");
        Console.WriteLine($"  Tests Run:       {TotalTests}");
        Console.WriteLine($"  Passed:          {PassedTests}");
        Console.WriteLine($"  Failed:          {FailedTests}");
        Console.WriteLine();
        Console.WriteLine("  Memory:");
        Console.WriteLine($"    Start:         {StartMemoryMB:F2} MB");
        Console.WriteLine($"    End:           {EndMemoryMB:F2} MB");
        Console.WriteLine($"    Growth:        {MemoryGrowthMB:F2} MB");
        Console.WriteLine();
        Console.WriteLine($"  OVERALL:         {(OverallPassed ? "✅ PASSED" : "❌ FAILED")}");
        Console.WriteLine();
        Console.WriteLine("═══════════════════════════════════════════════════════════════");

        if (FailedTests > 0)
        {
            Console.WriteLine();
            Console.WriteLine("  Failed Tests:");
            foreach (var result in Results.Where(r => !r.Passed))
            {
                Console.WriteLine($"    ❌ {result.TestName}: {result.Message}");
            }
        }
    }

    public async Task WriteToFileAsync(string path)
    {
        var json = System.Text.Json.JsonSerializer.Serialize(this, new System.Text.Json.JsonSerializerOptions
        {
            WriteIndented = true
        });
        await File.WriteAllTextAsync(path, json);
    }
}
