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

/// <summary>
/// Rolling mean ± population standard deviation. This is a spread band, not a confidence interval.
/// </summary>
public readonly record struct TraceBandPoint(double Low, double High);

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
    public required IReadOnlyList<TraceBandPoint?> LeftBand { get; init; }
    public required IReadOnlyList<TraceBandPoint?> RightBand { get; init; }
    public required IReadOnlyList<int> LeftAnomalies { get; init; }
    public required IReadOnlyList<int> RightAnomalies { get; init; }
    public int? LeftSteadyIndex { get; init; }
    public int? RightSteadyIndex { get; init; }
    public required IReadOnlyList<double?> LeftThroughput { get; init; }
    public required IReadOnlyList<double?> RightThroughput { get; init; }
    public double? LeftP50 { get; init; }
    public double? LeftP95 { get; init; }
    public double? LeftP99 { get; init; }
    public double? RightP50 { get; init; }
    public double? RightP95 { get; init; }
    public double? RightP99 { get; init; }
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

        var throughput = ThroughputWindows(left, right, signal, alignment);
        var unit = UnitLabel(leftSeries?.Unit ?? rightSeries?.Unit ?? ScalarUnit.None);
        var caption = string.Create(CultureInfo.InvariantCulture, $"{signal} ({unit}). {alignment.Summary}. The band is a centered 5-sample rolling mean ± population standard deviation, not a confidence interval. Marks are 3-sigma on this window. p50, p95, and p99 are nearest-rank. The distribution is the empirical CDF of these same samples.");
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
            LeftBand = DeviationBand(leftValues),
            RightBand = DeviationBand(rightValues),
            LeftAnomalies = ThreeSigmaIndices(leftValues),
            RightAnomalies = ThreeSigmaIndices(rightValues),
            LeftSteadyIndex = SteadyIndex(left.Timeline.Steps, alignment.SkippedStepsA, alignment.AlignedStepCount, left.Milestones.SteadyStateStartStep),
            RightSteadyIndex = SteadyIndex(right.Timeline.Steps, alignment.SkippedStepsB, alignment.AlignedStepCount, right.Milestones.SteadyStateStartStep),
            LeftThroughput = throughput.Left,
            RightThroughput = throughput.Right,
            LeftP50 = Percentile(leftFinite, 0.50),
            LeftP95 = Percentile(leftFinite, 0.95),
            LeftP99 = Percentile(leftFinite, 0.99),
            RightP50 = Percentile(rightFinite, 0.50),
            RightP95 = Percentile(rightFinite, 0.95),
            RightP99 = Percentile(rightFinite, 0.99),
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

    public const int DeviationRadius = 2;

    /// <summary>
    /// Centered window of five samples, clipped at the ends. Population standard deviation, divide by the count.
    /// A window with fewer than two numbers has no band.
    /// </summary>
    public static IReadOnlyList<TraceBandPoint?> DeviationBand(IReadOnlyList<double?> values)
    {
        var result = new TraceBandPoint?[values.Count];
        for (var index = 0; index < values.Count; index++)
        {
            var window = new List<double>();
            var start = Math.Max(0, index - DeviationRadius);
            var end = Math.Min(values.Count - 1, index + DeviationRadius);
            for (var cursor = start; cursor <= end; cursor++)
            {
                if (values[cursor] is double number && double.IsFinite(number))
                    window.Add(number);
            }

            if (window.Count < 2)
                continue;

            var mean = window.Average();
            var standardDeviation = Math.Sqrt(window.Sum(value => Math.Pow(value - mean, 2)) / window.Count);
            result[index] = new TraceBandPoint(mean - standardDeviation, mean + standardDeviation);
        }

        return result;
    }

    /// <summary>
    /// Indices whose value sits more than three population standard deviations from the window mean.
    /// Fewer than three numbers produces no marks. This is the same rule ΔF uses.
    /// </summary>
    public static IReadOnlyList<int> ThreeSigmaIndices(IReadOnlyList<double?> values)
    {
        var finite = new List<(int Index, double Value)>();
        for (var index = 0; index < values.Count; index++)
        {
            if (values[index] is double number && double.IsFinite(number))
                finite.Add((index, number));
        }

        if (finite.Count < 3)
            return [];

        var mean = finite.Average(sample => sample.Value);
        var standardDeviation = Math.Sqrt(finite.Sum(sample => Math.Pow(sample.Value - mean, 2)) / finite.Count);
        var threshold = 3 * standardDeviation;
        return finite.Where(sample => Math.Abs(sample.Value - mean) > threshold)
            .Select(sample => sample.Index)
            .ToList();
    }

    /// <summary>
    /// Index of the first aligned step at or after the steady-state milestone. Missing milestone means no span.
    /// </summary>
    public static int? SteadyIndex(IReadOnlyList<int> steps, int skip, int count, int? steadyStep)
    {
        if (steadyStep is not int steady || count <= 0)
            return null;

        var window = steps.Skip(Math.Max(0, skip)).Take(count).ToList();
        for (var index = 0; index < window.Count; index++)
        {
            if (window[index] >= steady)
                return index;
        }

        return null;
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

    private static (IReadOnlyList<double?> Left, IReadOnlyList<double?> Right) ThroughputWindows(
        RuntimeRunTrace left,
        RuntimeRunTrace right,
        string signal,
        AlignmentResult alignment)
    {
        if (signal.Equals("throughput_items_per_sec", StringComparison.OrdinalIgnoreCase))
            return ([], []);

        var leftSeries = left.Scalars.GetByName("throughput_items_per_sec");
        var rightSeries = right.Scalars.GetByName("throughput_items_per_sec");
        if (leftSeries == null || rightSeries == null)
            return ([], []);

        return (
            Window(leftSeries, alignment.SkippedStepsA, alignment.AlignedStepCount),
            Window(rightSeries, alignment.SkippedStepsB, alignment.AlignedStepCount));
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
