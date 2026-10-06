using ScalarScope.Models;
using ScalarScope.Services;
using System.Text;

namespace ScalarScope.Services;

/// <summary>
/// Phase 5.4: Generate copy-ready cards for sharing deltas.
/// </summary>
public static class DeltaCopyService
{
    /// <summary>
    /// Generate a plain-text copy card for a delta.
    /// </summary>
    public static string GeneratePlainTextCard(CanonicalDelta delta)
    {
        var sb = new StringBuilder();
        
        // Header with emoji based on delta type
        var emoji = GetDeltaEmoji(delta.DeltaType);
        sb.AppendLine($"{emoji} {delta.Name}");
        sb.AppendLine(new string('─', 30));
        
        // Main explanation
        sb.AppendLine(delta.Explanation);
        sb.AppendLine();
        
        // Values
        sb.AppendLine($"📊 Path A: {FormatValue(delta.LeftValue, delta.Units)}");
        sb.AppendLine($"📊 Path B: {FormatValue(delta.RightValue, delta.Units)}");
        sb.AppendLine($"📐 Δ: {FormatValue(delta.Delta, delta.Units)} ({GetDirection(delta.Delta)})");
        sb.AppendLine();
        
        // Confidence
        var tier = ConfidenceTokens.GetTierFromConfidence(delta.Confidence);
        var confidenceLabel = ConfidenceTokens.GetLabel(tier);
        sb.AppendLine($"🎯 Confidence: {confidenceLabel} ({delta.Confidence:P0})");
        
        // Visual anchor
        sb.AppendLine($"📍 Visible at: {delta.VisualAnchorTime:P0} of timeline");
        
        sb.AppendLine();
        sb.AppendLine("─ ScalarScope Analysis");
        
        return sb.ToString();
    }
    
    /// <summary>
    /// Generate a Markdown copy card for a delta.
    /// </summary>
    public static string GenerateMarkdownCard(CanonicalDelta delta)
    {
        var sb = new StringBuilder();
        
        // Header
        var emoji = GetDeltaEmoji(delta.DeltaType);
        sb.AppendLine($"### {emoji} {delta.Name}");
        sb.AppendLine();
        
        // Main explanation
        sb.AppendLine($"> {delta.Explanation}");
        sb.AppendLine();
        
        // Table of values
        sb.AppendLine("| Metric | Value |");
        sb.AppendLine("|--------|-------|");
        sb.AppendLine($"| Path A | {FormatValue(delta.LeftValue, delta.Units)} |");
        sb.AppendLine($"| Path B | {FormatValue(delta.RightValue, delta.Units)} |");
        sb.AppendLine($"| Delta | {FormatValue(delta.Delta, delta.Units)} ({GetDirection(delta.Delta)}) |");
        sb.AppendLine($"| Confidence | {ConfidenceTokens.GetLabel(ConfidenceTokens.GetTierFromConfidence(delta.Confidence))} |");
        sb.AppendLine();
        
        // Footer
        sb.AppendLine($"*Visible at {delta.VisualAnchorTime:P0} of timeline • ScalarScope*");
        
        return sb.ToString();
    }
    
    /// <summary>
    /// Phase 5.4: Generate plain-language executive summary.
    /// Non-technical stakeholders can understand this.
    /// </summary>
    public static string GeneratePlainLanguageSummary(IEnumerable<CanonicalDelta> deltas, string? leftRunName = null, string? rightRunName = null)
    {
        var deltaList = deltas.Where(d => d.Status == DeltaStatus.Present).ToList();
        var pathA = leftRunName ?? "Path A";
        var pathB = rightRunName ?? "Path B";
        
        if (deltaList.Count == 0)
        {
            return $"**Bottom Line:** {pathA} and {pathB} performed similarly. " +
                   "No significant differences were detected in timing, stability, or outcome.";
        }
        
        var sb = new StringBuilder();
        sb.AppendLine("## What We Found");
        sb.AppendLine();
        
        // Group by confidence for priority ordering
        var highConfidence = deltaList.Where(d => d.Confidence >= 0.99).ToList();
        var mediumConfidence = deltaList.Where(d => d.Confidence >= 0.95 && d.Confidence < 0.99).ToList();
        var lowConfidence = deltaList.Where(d => d.Confidence < 0.95).ToList();
        
        // Summary opening
        sb.AppendLine($"Comparing **{pathA}** and **{pathB}**, we found {deltaList.Count} notable difference{(deltaList.Count > 1 ? "s" : "")}:");
        sb.AppendLine();
        
        // High confidence findings (definite)
        if (highConfidence.Any())
        {
            sb.AppendLine("### Clear Findings (High Confidence)");
            foreach (var delta in highConfidence)
            {
                sb.AppendLine($"- {TranslateToPlainLanguage(delta, pathA, pathB)}");
            }
            sb.AppendLine();
        }
        
        // Medium confidence findings (likely)
        if (mediumConfidence.Any())
        {
            sb.AppendLine("### Likely Findings (Moderate Confidence)");
            foreach (var delta in mediumConfidence)
            {
                sb.AppendLine($"- {TranslateToPlainLanguage(delta, pathA, pathB)}");
            }
            sb.AppendLine();
        }
        
        // Low confidence findings (possible)
        if (lowConfidence.Any())
        {
            sb.AppendLine("### Possible Findings (Lower Confidence)");
            foreach (var delta in lowConfidence)
            {
                sb.AppendLine($"- {TranslateToPlainLanguage(delta, pathA, pathB)}");
            }
            sb.AppendLine();
        }
        
        // Bottom line
        sb.AppendLine("---");
        sb.AppendLine("**Bottom Line:** " + BottomLine(deltaList, pathA, pathB));
        
        return sb.ToString();
    }
    
    private static string TranslateToPlainLanguage(CanonicalDelta delta, string pathA, string pathB)
        => Describe(delta, pathA, pathB);

    /// <summary>
    /// The Why panel heading follows the row status. A withheld row is not "why this fired."
    /// </summary>
    public static string WhyHeading(DeltaStatus? status) => status switch
    {
        DeltaStatus.Suppressed => "Why this is withheld:",
        DeltaStatus.Indeterminate => "Why this is not settled:",
        DeltaStatus.Present => "Why this fired:",
        _ => "What this row says:"
    };

    /// <summary>
    /// The card uses the detector's own sentence. A missing sentence falls back
    /// to the step difference or the failure flags, never a normalized time times 100.
    /// </summary>
    public static string Describe(CanonicalDelta delta, string pathA, string pathB)
    {
        if (!string.IsNullOrWhiteSpace(delta.Explanation))
            return delta.Explanation;

        return DeltaIds.Canonical(delta.Id) switch
        {
            DeltaIds.ConvergenceTiming => DescribeConvergence(delta, pathA, pathB),
            DeltaIds.FailurePresence => DescribeFailure(delta, pathA, pathB),
            _ => ""
        };
    }

    private static string DescribeConvergence(CanonicalDelta delta, string pathA, string pathB)
    {
        if (delta.DeltaTcSteps is not int steps)
            return "";
        if (steps < 0)
            return $"{pathA} reached steady state {Math.Abs(steps)} steps sooner than {pathB}.";
        if (steps > 0)
            return $"{pathB} reached steady state {steps} steps sooner than {pathA}.";
        return $"{pathA} and {pathB} reached steady state together.";
    }

    private static string DescribeFailure(CanonicalDelta delta, string pathA, string pathB)
    {
        if (delta.FailedA == true && delta.FailedB == true)
            return $"Both {pathA} and {pathB} failed.";
        if (delta.FailedA == true)
            return $"{pathA} failed. {pathB} did not.";
        if (delta.FailedB == true)
            return $"{pathB} failed. {pathA} did not.";
        return "";
    }

    /// <summary>
    /// The closing sentence. Both failures stay both failures.
    /// </summary>
    public static string BottomLine(IReadOnlyList<CanonicalDelta> deltas, string pathA, string pathB)
    {
        var failure = deltas.FirstOrDefault(d => DeltaIds.Canonical(d.Id) == DeltaIds.FailurePresence);
        if (failure != null)
        {
            if (failure.FailedA == true && failure.FailedB == true)
                return $"Both {pathA} and {pathB} failed. Neither approach completed successfully.";
            if (failure.FailedA == true && failure.FailedB != true)
                return $"{pathB} completed successfully while {pathA} failed. The successful approach should be preferred.";
            if (failure.FailedB == true && failure.FailedA != true)
                return $"{pathA} completed successfully while {pathB} failed. The successful approach should be preferred.";
        }
        
        // Check dominance (emergence) delta
        var dominance = deltas.FirstOrDefault(d => DeltaIds.Canonical(d.Id) == DeltaIds.StructuralEmergence);
        var convergence = deltas.FirstOrDefault(d => DeltaIds.Canonical(d.Id) == DeltaIds.ConvergenceTiming);
        
        if (dominance != null && dominance.Confidence >= 0.95)
        {
            var better = dominance.Delta > 0 ? pathB : pathA;
            return $"{better} showed clearer emergence of shared evaluative structure, " +
                   "suggesting better generalization potential.";
        }
        
        if (convergence != null && convergence.Confidence >= 0.95)
        {
            var faster = convergence.Delta > 0 ? pathA : pathB;
            return $"{faster} converged faster, which may indicate more efficient learning.";
        }
        
        return "The differences detected are relatively minor. " +
               "Either approach appears viable, but review individual findings for specific trade-offs.";
    }
    
    /// <summary>
    /// Generate a summary card for multiple deltas.
    /// </summary>
    public static string GenerateSummaryCard(IEnumerable<CanonicalDelta> deltas)
    {
        var deltaList = deltas.Where(d => d.Status == DeltaStatus.Present).ToList();
        if (deltaList.Count == 0)
            return "No significant differences detected between the runs.";
        
        var sb = new StringBuilder();
        sb.AppendLine("## ScalarScope Comparison Summary");
        sb.AppendLine();
        sb.AppendLine($"**{deltaList.Count} differences detected:**");
        sb.AppendLine();
        
        foreach (var delta in deltaList)
        {
            var emoji = GetDeltaEmoji(delta.DeltaType);
            var tier = ConfidenceTokens.GetTierFromConfidence(delta.Confidence);
            var badge = tier switch
            {
                ConfidenceTokens.ConfidenceTier.High => "🟢",
                ConfidenceTokens.ConfidenceTier.Medium => "🟡",
                ConfidenceTokens.ConfidenceTier.Low => "🟠",
                _ => "⚪"
            };
            sb.AppendLine($"- {emoji} **{delta.Name}** {badge}");
            sb.AppendLine($"  {delta.Explanation}");
            sb.AppendLine();
        }
        
        sb.AppendLine("---");
        sb.AppendLine("*Generated by ScalarScope*");
        
        return sb.ToString();
    }
    
    /// <summary>
    /// Copy delta card to clipboard.
    /// </summary>
    public static async Task CopyToClipboardAsync(CanonicalDelta delta, bool useMarkdown = false)
    {
        var text = useMarkdown 
            ? GenerateMarkdownCard(delta) 
            : GeneratePlainTextCard(delta);
        
        await Clipboard.Default.SetTextAsync(text);
    }
    
    /// <summary>
    /// Copy summary card to clipboard.
    /// </summary>
    public static async Task CopySummaryToClipboardAsync(IEnumerable<CanonicalDelta> deltas)
    {
        var text = GenerateSummaryCard(deltas);
        await Clipboard.Default.SetTextAsync(text);
    }
    
    private static string GetDeltaEmoji(DeltaType type) => type switch
    {
        DeltaType.Event => "⚠️",
        DeltaType.Timing => "⏱️",
        DeltaType.Structure => "🔷",
        DeltaType.Behavior => "📈",
        _ => "🔹"
    };
    
    private static string FormatValue(double value, string? units)
    {
        var formatted = Math.Abs(value) < 0.01 || Math.Abs(value) > 1000
            ? value.ToString("G3")
            : value.ToString("F2");
        
        return string.IsNullOrEmpty(units) ? formatted : $"{formatted} {units}";
    }
    
    private static string GetDirection(double delta) => delta switch
    {
        > 0 => "↑",
        < 0 => "↓",
        _ => "="
    };
}
