using ScalarScope.Models;

namespace ScalarScope.Services;

/// <summary>
/// Handles temporal alignment between two runs.
/// Comparison without alignment is noise.
/// Phase 3: Make Comparison the Star
/// </summary>
public static class TemporalAlignmentService
{
    /// <summary>
    /// Get the default alignment mode.
    /// </summary>
    public static TemporalAlignment DefaultAlignment => TemporalAlignment.ByStep;

    /// <summary>
    /// Get all supported alignment modes with descriptions.
    /// </summary>
    public static IReadOnlyList<AlignmentOption> GetAlignmentOptions() =>
    [
        new("ByStep", "By Step", "Align by training step (epoch)"),
        new("ByConvergence", "By Convergence", "Align when paths stabilize"),
        new("ByFirstInstability", "By First Change", "Align at first major shift")
    ];

    /// <summary>
    /// Map a time from one run to aligned time in another run.
    /// </summary>
    public static double MapTime(
        double sourceTime,
        GeometryRun? sourceRun,
        GeometryRun? targetRun,
        TemporalAlignment alignment)
    {
        if (sourceRun == null || targetRun == null)
            return sourceTime;

        return alignment switch
        {
            TemporalAlignment.ByStep => MapByStep(sourceTime, sourceRun, targetRun),
            TemporalAlignment.ByConvergence => MapByConvergence(sourceTime, sourceRun, targetRun),
            TemporalAlignment.ByFirstInstability => MapByFirstInstability(sourceTime, sourceRun, targetRun),
            _ => sourceTime
        };
    }

    /// <summary>
    /// Get alignment anchors for visualization.
    /// </summary>
    public static AlignmentAnchors GetAnchors(
        GeometryRun? leftRun,
        GeometryRun? rightRun,
        TemporalAlignment alignment)
    {
        if (leftRun == null || rightRun == null)
        {
            return new AlignmentAnchors
            {
                LeftAnchor = 0,
                RightAnchor = 0,
                AnchorDescription = "No data"
            };
        }

        return alignment switch
        {
            TemporalAlignment.ByStep => new AlignmentAnchors
            {
                LeftAnchor = 0,
                RightAnchor = 0,
                AnchorDescription = "Aligned by training step"
            },
            TemporalAlignment.ByConvergence => GetConvergenceAnchors(leftRun, rightRun),
            TemporalAlignment.ByFirstInstability => GetInstabilityAnchors(leftRun, rightRun),
            _ => new AlignmentAnchors
            {
                LeftAnchor = 0,
                RightAnchor = 0,
                AnchorDescription = "Unknown alignment"
            }
        };
    }

    // === Private Methods ===

    /// <summary>
    /// Direct step-to-step mapping (normalized time).
    /// </summary>
    private static double MapByStep(double time, GeometryRun source, GeometryRun target)
    {
        // Both runs use normalized time [0, 1], so no transformation needed
        return Math.Clamp(time, 0, 1);
    }

    /// <summary>
    /// Map times relative to convergence point.
    /// </summary>
    private static double MapByConvergence(double time, GeometryRun source, GeometryRun target)
    {
        var sourceConvergence = FindConvergenceTime(source);
        var targetConvergence = FindConvergenceTime(target);

        // Fall back only when both anchors are missing. One missing side is not aligned.
        if (sourceConvergence < 0 && targetConvergence < 0)
            return time;
        if (sourceConvergence < 0 || targetConvergence < 0)
            return time;

        // Map time relative to convergence
        // If source time is at convergence, target time should be at target convergence
        var relativeToConvergence = time - sourceConvergence;
        var mappedTime = targetConvergence + relativeToConvergence;

        return Math.Clamp(mappedTime, 0, 1);
    }

    /// <summary>
    /// Map times relative to first instability.
    /// </summary>
    private static double MapByFirstInstability(double time, GeometryRun source, GeometryRun target)
    {
        var sourceInstability = FindFirstInstabilityTime(source);
        var targetInstability = FindFirstInstabilityTime(target);

        // Fall back only when both anchors are missing. One missing side is not aligned.
        if (sourceInstability < 0 && targetInstability < 0)
            return time;
        if (sourceInstability < 0 || targetInstability < 0)
            return time;

        // Map time relative to first instability
        var relativeToInstability = time - sourceInstability;
        var mappedTime = targetInstability + relativeToInstability;

        return Math.Clamp(mappedTime, 0, 1);
    }

    private static AlignmentAnchors GetConvergenceAnchors(GeometryRun left, GeometryRun right)
    {
        var leftConvergence = FindConvergenceTime(left);
        var rightConvergence = FindConvergenceTime(right);

        if (leftConvergence < 0 && rightConvergence < 0)
        {
            return new AlignmentAnchors
            {
                LeftAnchor = 0.5,
                RightAnchor = 0.5,
                AnchorDescription = "Neither path converged; using midpoint"
            };
        }

        if (leftConvergence < 0 || rightConvergence < 0)
        {
            var missing = leftConvergence < 0 ? "Path A did not converge" : "Path B did not converge";
            return new AlignmentAnchors
            {
                LeftAnchor = leftConvergence < 0 ? 0 : leftConvergence,
                RightAnchor = rightConvergence < 0 ? 0 : rightConvergence,
                AnchorDescription = $"{missing}; times are not aligned"
            };
        }

        return new AlignmentAnchors
        {
            LeftAnchor = leftConvergence,
            RightAnchor = rightConvergence,
            AnchorDescription = $"Aligned at convergence (A: {leftConvergence:P0}, B: {rightConvergence:P0})"
        };
    }

    private static AlignmentAnchors GetInstabilityAnchors(GeometryRun left, GeometryRun right)
    {
        var leftInstability = FindFirstInstabilityTime(left);
        var rightInstability = FindFirstInstabilityTime(right);

        if (leftInstability < 0 && rightInstability < 0)
        {
            return new AlignmentAnchors
            {
                LeftAnchor = 0.1,
                RightAnchor = 0.1,
                AnchorDescription = "Neither path showed instability; using early point"
            };
        }

        if (leftInstability < 0 || rightInstability < 0)
        {
            var missing = leftInstability < 0 ? "Path A remained stable" : "Path B remained stable";
            return new AlignmentAnchors
            {
                LeftAnchor = leftInstability < 0 ? 0 : leftInstability,
                RightAnchor = rightInstability < 0 ? 0 : rightInstability,
                AnchorDescription = $"{missing}; times are not aligned"
            };
        }

        return new AlignmentAnchors
        {
            LeftAnchor = leftInstability,
            RightAnchor = rightInstability,
            AnchorDescription = $"Aligned at first shift (A: {leftInstability:P0}, B: {rightInstability:P0})"
        };
    }

    /// <summary>
    /// Find when trajectory velocity stabilizes below threshold.
    /// </summary>
    private static double FindConvergenceTime(GeometryRun run)
    {
        // Same detector that produces ΔTc. A fixed 0.05 band is a different event.
        return CanonicalDeltaService.DetectConvergenceAnchor(run).Time;
    }

    /// <summary>
    /// Find first significant curvature spike.
    /// </summary>
    private static double FindFirstInstabilityTime(GeometryRun run)
    {
        var steps = run.Trajectory?.Timesteps;
        if (steps == null || steps.Count < 3)
            return -1;

        // Compute mean curvature for threshold
        var meanCurvature = steps.Average(s => s.Curvature);
        var threshold = Math.Max(meanCurvature * 2, 0.3);

        for (int i = 1; i < steps.Count; i++)
        {
            if (steps[i].Curvature > threshold)
            {
                return i / (double)(steps.Count - 1);
            }
        }

        return -1;
    }
}

/// <summary>
/// Temporal alignment modes for comparison.
/// </summary>
public enum TemporalAlignment
{
    /// <summary>Align by training step (epoch). Default.</summary>
    ByStep,
    
    /// <summary>Align when paths stabilize (convergence onset).</summary>
    ByConvergence,
    
    /// <summary>Align at first major direction change.</summary>
    ByFirstInstability
}

/// <summary>
/// Describes an alignment option for the UI.
/// </summary>
public record AlignmentOption(string Value, string Label, string Description);

/// <summary>
/// Anchor points for alignment visualization.
/// </summary>
public record AlignmentAnchors
{
    public double LeftAnchor { get; init; }
    public double RightAnchor { get; init; }
    public string AnchorDescription { get; init; } = "";
}
