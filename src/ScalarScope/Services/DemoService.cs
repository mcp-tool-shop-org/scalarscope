using System.Text.Json;
using ScalarScope.Models;

namespace ScalarScope.Services;

/// <summary>
/// Manages the bundled demo runs and demo session state.
/// Provides access to Path A (orthogonal) and Path B (correlated) sample data.
/// </summary>
public static class DemoService
{
    /// <summary>
    /// Bundled sample file for Path A (orthogonal professors).
    /// </summary>
    public const string PathAFileName = "orthogonal_professors.json";

    /// <summary>
    /// Bundled sample file for Path B (correlated professors).
    /// </summary>
    public const string PathBFileName = "correlated_professors.json";

    /// <summary>
    /// Is a demo session currently active?
    /// </summary>
    public static bool IsDemoActive { get; private set; }

    /// <summary>
    /// Why the last sample load failed. Null when both samples loaded.
    /// </summary>
    public static string? LastFailure { get; private set; }

    /// <summary>
    /// Mark a demo active so a threshold check can be tried without the sample files.
    /// </summary>
    public static void ArmForChecks()
    {
        IsDemoActive = true;
    }

    /// <summary>Name the sample file and the last error. Does not log.</summary>
    public static string DescribeLoadFailure(string fileName, Exception? ex)
    {
        var reason = string.IsNullOrWhiteSpace(ex?.Message)
            ? "No matching file was found in the app package."
            : ex.Message;
        return $"The sample file {fileName} could not be loaded. {reason}";
    }

    /// <summary>
    /// The currently loaded demo runs (if demo is active).
    /// </summary>
    public static GeometryRun? DemoPathA { get; private set; }
    public static GeometryRun? DemoPathB { get; private set; }

    /// <summary>
    /// Load Path A (orthogonal professors) from bundled resources.
    /// </summary>
    public static async Task<GeometryRun?> LoadPathAAsync()
    {
        return await LoadBundledRunAsync(PathAFileName);
    }

    /// <summary>
    /// Load Path B (correlated professors) from bundled resources.
    /// </summary>
    public static async Task<GeometryRun?> LoadPathBAsync()
    {
        return await LoadBundledRunAsync(PathBFileName);
    }

    /// <summary>
    /// Start a demo session by loading both Path A and Path B.
    /// </summary>
    public static async Task<(GeometryRun? PathA, GeometryRun? PathB)> StartDemoAsync()
    {
        LastFailure = null;
        DemoPathA = await LoadPathAAsync();
        DemoPathB = await LoadPathBAsync();

        if (DemoPathA != null && DemoPathB != null)
        {
            LastFailure = null;
            IsDemoActive = true;
            UserPreferencesService.MarkDemoSeen();

            // Reset annotation state for new demo
            DemoAnnotationService.ResetForNewDemo();
        }
        else
        {
            IsDemoActive = false;
        }

        return (DemoPathA, DemoPathB);
    }

    /// <summary>
    /// End the current demo session.
    /// </summary>
    public static void EndDemo(bool completed = false)
    {
        IsDemoActive = false;
        DemoPathA = null;
        DemoPathB = null;

        if (completed)
        {
            UserPreferencesService.MarkDemoCompleted();
        }
    }

    /// <summary>
    /// Load a bundled sample run from app package resources.
    /// Tries multiple paths for compatibility across different MAUI deployments.
    /// </summary>
    private static async Task<GeometryRun?> LoadBundledRunAsync(string fileName)
    {
        Exception? last = null;
        var pathsToTry = new[]
        {
            $"Samples/{fileName}",
            $"Samples\\{fileName}",
            fileName,
        };

        foreach (var path in pathsToTry)
        {
            try
            {
                using var stream = await FileSystem.OpenAppPackageFileAsync(path);
                using var reader = new StreamReader(stream);
                var json = await reader.ReadToEndAsync();
                var run = JsonSerializer.Deserialize<GeometryRun>(json);
                if (run != null)
                    return run;
                last = new InvalidDataException("The sample file did not contain a run.");
            }
            catch (Exception ex)
            {
                last = ex;
            }
        }

        var sourceDir = AppContext.BaseDirectory;
        var devPaths = new[]
        {
            Path.Combine(sourceDir, "Resources", "Raw", "Samples", fileName),
            Path.Combine(sourceDir, "..", "..", "..", "..", "Resources", "Raw", "Samples", fileName),
        };

        foreach (var devPath in devPaths)
        {
            try
            {
                if (File.Exists(devPath))
                {
                    var json = await File.ReadAllTextAsync(devPath);
                    var run = JsonSerializer.Deserialize<GeometryRun>(json);
                    if (run != null)
                        return run;
                    last = new InvalidDataException("The sample file did not contain a run.");
                }
            }
            catch (Exception ex)
            {
                last = ex;
            }
        }

        RememberFailure(fileName, last);
        return null;
    }

    private static void RememberFailure(string fileName, Exception? ex)
    {
        var sentence = DescribeLoadFailure(fileName, ex);
        LastFailure = string.IsNullOrEmpty(LastFailure) ? sentence : $"{LastFailure} {sentence}";
        try
        {
            ErrorLoggingService.Instance.Log(ErrorSeverity.Error, sentence, nameof(DemoService));
        }
        catch (Exception logEx)
        {
            System.Diagnostics.Debug.WriteLine(logEx.Message);
        }
    }

    /// <summary>
    /// Get the display name for a demo path.
    /// </summary>
    public static string GetPathDisplayName(bool isPathA)
    {
        return isPathA
            ? "Path A: Orthogonal Professors"
            : "Path B: Correlated Professors";
    }

    /// <summary>
    /// Get a brief description for demo annotations.
    /// </summary>
    public static string GetPathDescription(bool isPathA)
    {
        return isPathA
            ? "Professors have independent evaluation criteria (r ≈ 0). No shared latent structure to converge on."
            : "Professors share a latent quality axis (r ≈ 0.87). Unified representation emerges naturally.";
    }
}
