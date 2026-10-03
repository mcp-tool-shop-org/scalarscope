using System.Globalization;
using System.Text.Json;

namespace ScalarScope.Services.Connectors;

/// <summary>
/// A geometry export has a trajectory. Anything else is offered to a trace connector.
/// </summary>
public static class TraceOpen
{
    public static bool IsGeometryRun(string text)
    {
        if (string.IsNullOrWhiteSpace(text))
            return false;

        try
        {
            using var document = JsonDocument.Parse(text);
            if (!document.RootElement.TryGetProperty("trajectory", out var trajectory))
                return false;
            if (!trajectory.TryGetProperty("timesteps", out var steps) || steps.ValueKind != JsonValueKind.Array)
                return false;
            return steps.GetArrayLength() > 0;
        }
        catch (JsonException)
        {
            return false;
        }
    }
}

/// <summary>
/// One point on an empirical CDF. Probability is rank / n, so the last point is 1.
/// </summary>
public readonly record struct TraceDistributionPoint(double Value, double Probability);

public sealed record TraceFinding
{
    public required string Symbol { get; init; }
    public required string Signal { get; init; }
    public required string Text { get; init; }
}

/// <summary>
/// Two readings of one aligned window: the series, and the distribution of those same samples.
/// </summary>
public sealed record TraceReview
{
    public required string LeftLabel { get; init; }
    public required string RightLabel { get; init; }
    public required string LeftPath { get; init; }
    public required string RightPath { get; init; }
    public required string Signal { get; init; }
    public required string Unit { get; init; }
    public required int SkippedLeft { get; init; }
    public required int SkippedRight { get; init; }
    public required IReadOnlyList<double?> LeftValues { get; init; }
    public required IReadOnlyList<double?> RightValues { get; init; }
    public required IReadOnlyList<TraceDistributionPoint> LeftDistribution { get; init; }
    public required IReadOnlyList<TraceDistributionPoint> RightDistribution { get; init; }
    public double? LeftP50 { get; init; }
    public double? LeftP95 { get; init; }
    public double? RightP50 { get; init; }
    public double? RightP95 { get; init; }
    public required string Caption { get; init; }
    public required string Verdict { get; init; }
    public required IReadOnlyList<TraceFinding> Findings { get; init; }
    public required IReadOnlyList<string> Withheld { get; init; }
    public required IReadOnlyList<string> FiredSymbols { get; init; }
}

public static class TraceReviewBuilder
{
    private static readonly string[] PreferredSignals = ["latency_ms", "latency", "throughput_items_per_sec"];
    private static readonly string[] SymbolOrder = ["ΔF", "ΔTc", "ΔTd", "ΔĀ", "ΔO"];

    public static TraceReview Build(
        RuntimeRunTrace left,
        RuntimeRunTrace right,
        ComparisonResult comparison,
        string leftPath,
        string rightPath)
    {
        var signal = ChooseSignal(left, right, comparison);
        var leftSeries = left.Scalars.GetByName(signal);
        var rightSeries = right.Scalars.GetByName(signal);
        var alignment = comparison.Alignment;
        var leftValues = Window(leftSeries, alignment.SkippedStepsA, alignment.AlignedStepCount);
        var rightValues = Window(rightSeries, alignment.SkippedStepsB, alignment.AlignedStepCount);
        var leftFinite = Finite(leftValues);
        var rightFinite = Finite(rightValues);
        var findings = new List<TraceFinding>();
        var withheld = new List<string>();
        var steady = left.Milestones.SteadyStateStartStep.HasValue && right.Milestones.SteadyStateStartStep.HasValue;

        foreach (var delta in comparison.Deltas.Where(item => item.Fired && !item.IsSuppressed))
        {
            if (delta.DeltaType == "ΔTc" && !steady)
            {
                withheld.Add("ΔTc is withheld. A steady-state milestone is missing, so the last step is not a stabilization time.");
                continue;
            }

            findings.Add(new TraceFinding
            {
                Symbol = delta.DeltaType,
                Signal = delta.Signal,
                Text = string.IsNullOrWhiteSpace(delta.Interpretation)
                    ? delta.DeltaType
                    : delta.Interpretation
            });
        }

        var fired = SymbolOrder.Where(symbol => findings.Any(finding => finding.Symbol == symbol)).ToList();
        foreach (var finding in findings)
        {
            if (!fired.Contains(finding.Symbol, StringComparer.Ordinal))
                fired.Add(finding.Symbol);
        }

        var unit = UnitLabel(leftSeries?.Unit ?? rightSeries?.Unit ?? ScalarUnit.None);
        var caption = string.Create(CultureInfo.InvariantCulture, $"{signal} ({unit}). {alignment.Summary}. p50 and p95 are nearest-rank on this window. The distribution is the empirical CDF of these same samples.");
        var verdictParts = findings.Select(finding => $"{finding.Symbol} {finding.Text}").ToList();
        if (verdictParts.Count == 0)
            verdictParts.Add($"No delta fired on {signal}.");
        verdictParts.AddRange(withheld);

        return new TraceReview
        {
            LeftLabel = left.Label ?? Path.GetFileNameWithoutExtension(leftPath),
            RightLabel = right.Label ?? Path.GetFileNameWithoutExtension(rightPath),
            LeftPath = leftPath,
            RightPath = rightPath,
            Signal = signal,
            Unit = unit,
            SkippedLeft = alignment.SkippedStepsA,
            SkippedRight = alignment.SkippedStepsB,
            LeftValues = leftValues,
            RightValues = rightValues,
            LeftDistribution = EmpiricalCdf(leftFinite),
            RightDistribution = EmpiricalCdf(rightFinite),
            LeftP50 = Percentile(leftFinite, 0.50),
            LeftP95 = Percentile(leftFinite, 0.95),
            RightP50 = Percentile(rightFinite, 0.50),
            RightP95 = Percentile(rightFinite, 0.95),
            Caption = caption,
            Verdict = string.Join(" ", verdictParts),
            Findings = findings,
            Withheld = withheld,
            FiredSymbols = fired
        };
    }

    /// <summary>
    /// Nearest-rank percentile. The index is ceil(p * n) - 1 on the sorted samples.
    /// </summary>
    public static double? Percentile(IReadOnlyList<double> sortedAscending, double probability)
    {
        if (sortedAscending.Count == 0)
            return null;

        var rank = (int)Math.Ceiling(probability * sortedAscending.Count) - 1;
        if (rank < 0)
            rank = 0;
        if (rank >= sortedAscending.Count)
            rank = sortedAscending.Count - 1;
        return sortedAscending[rank];
    }

    public static IReadOnlyList<TraceDistributionPoint> EmpiricalCdf(IReadOnlyList<double> sortedAscending)
    {
        var count = sortedAscending.Count;
        var points = new TraceDistributionPoint[count];
        for (var index = 0; index < count; index++)
            points[index] = new TraceDistributionPoint(sortedAscending[index], (index + 1) / (double)count);
        return points;
    }

    private static string ChooseSignal(RuntimeRunTrace left, RuntimeRunTrace right, ComparisonResult comparison)
    {
        var names = left.Scalars.Series.Select(series => series.Name)
            .Intersect(right.Scalars.Series.Select(series => series.Name), StringComparer.OrdinalIgnoreCase)
            .ToList();

        foreach (var preferred in PreferredSignals)
        {
            var match = names.FirstOrDefault(name => name.Equals(preferred, StringComparison.OrdinalIgnoreCase));
            if (match != null)
                return match;
        }

        var fromDelta = comparison.Deltas.FirstOrDefault(delta => !string.IsNullOrWhiteSpace(delta.Signal))?.Signal;
        if (fromDelta != null && names.Contains(fromDelta, StringComparer.OrdinalIgnoreCase))
            return names.First(name => name.Equals(fromDelta, StringComparison.OrdinalIgnoreCase));

        return names.FirstOrDefault() ?? "latency_ms";
    }

    private static IReadOnlyList<double?> Window(RuntimeScalarSeries? series, int skip, int count)
    {
        if (series == null || count <= 0)
            return [];
        return series.Values.Skip(Math.Max(0, skip)).Take(count).ToList();
    }

    private static IReadOnlyList<double> Finite(IReadOnlyList<double?> values)
    {
        return values.Where(value => value.HasValue && double.IsFinite(value.Value))
            .Select(value => value!.Value)
            .OrderBy(value => value)
            .ToList();
    }

    private static string UnitLabel(ScalarUnit unit)
    {
        return unit switch
        {
            ScalarUnit.Milliseconds => "ms",
            ScalarUnit.Seconds => "s",
            ScalarUnit.Microseconds => "µs",
            ScalarUnit.ItemsPerSecond => "items/s",
            ScalarUnit.Bytes => "bytes",
            ScalarUnit.Percent => "%",
            _ => unit.ToString().ToLowerInvariant()
        };
    }
}
