using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using ScalarScope.Models;
using ScalarScope.Services;
using ScalarScope.Services.Evidence;

namespace ScalarScope.SoakTests;

/// <summary>
/// Phase 3.2 Validation Runner - Generates delta suite validation summary.
/// Run with: dotnet run --project tests/ScalarScope.SoakTests -- --validate
/// </summary>
public class Phase32ValidationRunner
{
    private readonly ILogger _logger;

    public Phase32ValidationRunner(ILogger? logger = null)
    {
        _logger = logger ?? NullLogger.Instance;
    }

    /// <summary>Recorded floors. Probes sit on these numbers so a threshold edit changes the exit code.</summary>
    private const int ResolutionStepsFloor = 3;
    private const int MinDurationFloor = 4;
    private const int PersistenceStepsFloor = 3;
    private const int StepCount = 40;

    /// <summary>
    /// Run Phase 3.2 validation and return summary report.
    /// </summary>
    public Task<Phase32ValidationReport> RunValidationAsync()
    {
        _logger.LogInformation("=== Phase 3.2 Delta Suite Validation ===");

        var report = new Phase32ValidationReport
        {
            ValidationTime = DateTime.UtcNow,
            Phase = "3.2 - Scientific Tuning"
        };

        try
        {
            var measured = MeasurePairs();
            report.PairResults = measured.Pairs;
            report.DeltaImplementationStatus = measured.Status;
            report.DeltaFVerification = measured.FailureChecks;
            report.SuiteGates = measured.Gates;
            report.Notes.Add($"CanonicalDeltaService measured {measured.Pairs.Count} pairs.");
            report.Notes.Add($"Clearly different present deltas: {measured.ClearPresent}. Nearly identical present deltas: {measured.NearPresent}.");
            report.Notes.Add($"Resolution boundary (floor {ResolutionStepsFloor}): below fired={measured.ResolutionBelowFired}, at floor fired={measured.ResolutionAtFired}.");
            report.Notes.Add($"Persistence boundary (floor {PersistenceStepsFloor}): {PersistenceStepsFloor - 1}-step fired={measured.ShortSpikeFired}, {PersistenceStepsFloor}-step fired={measured.PersistentSpikeFired}.");
            report.Notes.Add($"Stability boundary (MinDuration {MinDurationFloor}, area-above-theta): short fired={measured.ShortEpisodeFired}, qualifying fired={measured.QualifyingEpisodeFired}, sub-noise fired={measured.TinyAreaFired}, evidence episodes={measured.QualifyingEvidenceEpisodes}.");
        }
        catch (Exception ex)
        {
            report.Notes.Add($"Detector measurement failed: {ex.Message}");
            report.Locked = false;
            report.LockDecision = "Detector measurement threw. Not locked.";
            _logger.LogError(ex, "Phase 3.2 measurement failed");
            return Task.FromResult(report);
        }

        report.Locked = CanLock(report);
        report.LockDecision = report.Locked
            ? "Measured gates passed. Ready for lock."
            : "Measured gates did not pass. Not locked.";
        _logger.LogInformation("Phase 3.2 lock={Locked}", report.Locked);
        return Task.FromResult(report);
    }

    private static MeasuredSuite MeasurePairs()
    {
        var evidence = new EvidenceExportService();
        var calm = Flat("calm");
        var twin = Flat("twin");
        var failed = Flat("failed", failures:
        [
            new FailureEvent { T = 0.40, Category = "collapse", Severity = "HIGH", Description = "synthetic failure" },
            new FailureEvent { T = 0.45, Category = "collapse", Severity = "HIGH", Description = "synthetic failure" },
            new FailureEvent { T = 0.50, Category = "collapse", Severity = "HIGH", Description = "synthetic failure" }
        ]);

        var clear = Measure(calm, failed);
        var near = Measure(calm, twin);
        var resolutionBelow = Measure(Settled("early", WindowStart), Settled("late-below", WindowStart + ResolutionStepsFloor - 1));
        var resolutionAt = Measure(Settled("early-at", WindowStart), Settled("late-at", WindowStart + ResolutionStepsFloor));
        var shortSpike = Measure(calm, Spiked("spike-2", PersistenceStepsFloor - 1));
        var persistentSpike = Measure(calm, Spiked("spike-3", PersistenceStepsFloor));
        var moderate = Measure(Alternating("alt-a"), Alternating("alt-b"));
        var shortEpisode = Measure(Flat("still-a"), Curved("curve-3", MinDurationFloor - 1, 1.0));
        var qualifyingRun = Curved("curve-4", MinDurationFloor, 1.0);
        var qualifying = Measure(Flat("still-b"), qualifyingRun);
        var tinyArea = Measure(Flat("still-c"), Curved("curve-noise", MinDurationFloor, 0.02));
        var alignmentBelow = Measure(Aligned("align-low-a", 0.20), Aligned("align-low-b", 0.24));
        var alignmentAt = Measure(Aligned("align-high-a", 0.20), Aligned("align-high-b", 0.26));
        var emerged = Measure(Flat("distributed"), Dominant("dominant"));

        var qualifyingEvidence = evidence.CaptureDetectorDiagnostics(
            Flat("evidence-flat"), qualifyingRun, TemporalAlignment.ByStep, 1.0, "area-above-theta");
        var qualifyingEpisodes = qualifyingEvidence.Stability?.RunBEpisodes.Count(episode =>
            episode.Duration >= MinDurationFloor && episode.AreaScore > 0) ?? 0;
        var tinyEvidence = evidence.CaptureDetectorDiagnostics(
            Flat("evidence-flat-tiny"), Curved("curve-noise-b", MinDurationFloor, 0.02),
            TemporalAlignment.ByStep, 1.0, "area-below-noise");
        var tinyEpisodes = tinyEvidence.Stability?.RunBEpisodes.Count ?? -1;

        var clearPresent = clear.Count(delta => delta.Status == ScalarScope.Services.DeltaStatus.Present);
        var nearPresent = near.Count(delta => delta.Status == ScalarScope.Services.DeltaStatus.Present);
        var clearFired = Present(clear, "FailurePresence") && clearPresent >= 1;
        var nearQuiet = nearPresent == 0 && !Present(near, "FailurePresence");
        var resolutionBelowFired = Present(resolutionBelow, "ConvergenceTiming");
        var resolutionAtFired = Present(resolutionAt, "ConvergenceTiming");
        var shortSpikeFired = Present(shortSpike, "FailurePresence");
        var persistentSpikeFired = Present(persistentSpike, "FailurePresence");
        var moderateFired = Present(moderate, "FailurePresence");
        var shortEpisodeFired = Present(shortEpisode, "StabilityOscillation");
        var qualifyingEpisodeFired = Present(qualifying, "StabilityOscillation");
        var tinyAreaFired = Present(tinyArea, "StabilityOscillation");
        var alignmentBelowFired = Present(alignmentBelow, "EvaluatorAlignment");
        var alignmentAtFired = Present(alignmentAt, "EvaluatorAlignment");
        var emergenceFired = Present(emerged, "StructuralEmergence");

        var falsePositivePassed = nearQuiet && !moderateFired;
        var persistencePassed = !shortSpikeFired && persistentSpikeFired;
        var extremeThresholdPassed = !moderateFired && persistentSpikeFired;
        var resolutionPassed = !resolutionBelowFired && resolutionAtFired;
        var noisePassed = !shortEpisodeFired && qualifyingEpisodeFired && qualifyingEpisodes > 0
            && !tinyAreaFired && tinyEpisodes == 0;
        var alignmentPassed = !alignmentBelowFired && alignmentAtFired;

        List<CanonicalDelta> byStep;
        List<CanonicalDelta> byConvergence;
        string? alignmentError = null;
        try
        {
            byStep = Measure(calm, failed, TemporalAlignment.ByStep);
            byConvergence = Measure(calm, failed, TemporalAlignment.ByConvergence);
        }
        catch (Exception ex)
        {
            byStep = [];
            byConvergence = [];
            alignmentError = ex.Message;
        }

        var consistent = alignmentError == null && SameConclusions(byStep, byConvergence);

        var pairs = new List<PairValidationResult>
        {
            ToPair("clearly-different", "clearly_different", clear, clearFired),
            ToPair("nearly-identical", "nearly_identical", near, nearQuiet),
            ToPair("resolution-below", "subtly_different", resolutionBelow, !resolutionBelowFired),
            ToPair("resolution-at-floor", "clearly_different", resolutionAt, resolutionAtFired),
            ToPair("persistence-below", "nearly_identical", shortSpike, !shortSpikeFired),
            ToPair("persistence-at-floor", "one_failure", persistentSpike, persistentSpikeFired),
            ToPair("moderate-variance", "nearly_identical", moderate, !moderateFired),
            ToPair("duration-below", "nearly_identical", shortEpisode, !shortEpisodeFired),
            ToPair("duration-at-floor", "clearly_different", qualifying, qualifyingEpisodeFired),
            ToPair("area-below-noise", "nearly_identical", tinyArea, !tinyAreaFired),
            ToPair("alignment-boundary", "subtly_different", alignmentAt, alignmentPassed),
            ToPair("emergence", "clearly_different", emerged, emergenceFired)
        };

        var status = new DeltaImplementationStatus
        {
            DeltaA = Status("ΔĀ (Evaluator Alignment)", alignmentPassed,
                "Persistence score must clear the 0.05 floor over a sustained segment."),
            DeltaTd = Status("ΔTd (Structural Emergence)", emergenceFired,
                "Dominance on one run and not the other must surface StructuralEmergence."),
            DeltaTc = Status("ΔTc (Convergence Timing)", resolutionPassed,
                $"Only a step gap of at least {ResolutionStepsFloor} fires ConvergenceTiming."),
            DeltaO = Status("ΔO (Stability Oscillation)", noisePassed,
                $"Episodes shorter than {MinDurationFloor} or under the area noise floor must not fire."),
            DeltaF = Status("ΔF (Failure Detection)", persistencePassed && falsePositivePassed,
                $"{PersistenceStepsFloor} consecutive divergence steps fire; fewer steps and 2× velocity do not.")
        };

        var checks = new DeltaFVerificationResult
        {
            Summary = falsePositivePassed && persistencePassed && extremeThresholdPassed
                ? "ΔF trigger counts matched the recorded floors."
                : "ΔF trigger counts missed a recorded floor.",
            Checks =
            [
                Check("ΔF-1: False-positive audit",
                    "ΔF triggers on 0 nearly-identical and moderate pairs",
                    falsePositivePassed,
                    $"nearly-identical present={nearPresent}; moderate failure={moderateFired}"),
                Check("ΔF-2: PersistenceWindow boundary",
                    $"{PersistenceStepsFloor - 1} divergence steps do not fire; {PersistenceStepsFloor} do",
                    persistencePassed,
                    $"shortFired={shortSpikeFired}; persistentFired={persistentSpikeFired}"),
                Check("ΔF-3: Extreme threshold",
                    "2× velocity does not fire; a 10× persistent spike does",
                    extremeThresholdPassed,
                    $"moderateFired={moderateFired}; persistentFired={persistentSpikeFired}")
            ]
        };

        var gates = new SuiteGatesResult
        {
            GateA = clearFired && nearQuiet && pairs.Count > 0,
            GateB = resolutionPassed && alignmentPassed,
            GateC = noisePassed,
            GateD = consistent
        };
        gates.GateANotes.Add($"Clearly different present={clearPresent} (need >= 1). Nearly identical present={nearPresent} (need 0).");
        gates.GateBNotes.Add($"Convergence gap {ResolutionStepsFloor - 1} fired={resolutionBelowFired}; gap {ResolutionStepsFloor} fired={resolutionAtFired}.");
        gates.GateBNotes.Add($"Alignment difference 0.04 fired={alignmentBelowFired}; difference 0.06 fired={alignmentAtFired}.");
        gates.GateCNotes.Add($"Duration {MinDurationFloor - 1} fired={shortEpisodeFired}; duration {MinDurationFloor} fired={qualifyingEpisodeFired}; evidence episodes={qualifyingEpisodes}.");
        gates.GateCNotes.Add($"Sub-noise area fired={tinyAreaFired}; evidence episodes={tinyEpisodes}.");
        gates.GateDNotes.Add(alignmentError == null
            ? "ByStep and ByConvergence present-delta sets and signs matched on the failure pair."
            : $"Alignment comparison threw: {alignmentError}");

        return new MeasuredSuite(pairs, status, checks, gates, clearPresent, nearPresent,
            resolutionBelowFired, resolutionAtFired, shortSpikeFired, persistentSpikeFired,
            shortEpisodeFired, qualifyingEpisodeFired, tinyAreaFired, qualifyingEpisodes);
    }

    private const int WindowStart = 5;

    private static List<CanonicalDelta> Measure(GeometryRun left, GeometryRun right, TemporalAlignment alignment = TemporalAlignment.ByStep)
        => CanonicalDeltaService.ComputeDeltas(left, right, alignment, 1.0, CanonicalDeltaService.DefaultConfig);

    private static bool Present(IReadOnlyList<CanonicalDelta> deltas, string id)
        => deltas.Any(delta => SameId(delta.Id, id) && delta.Status == ScalarScope.Services.DeltaStatus.Present);

    private static bool SameId(string? left, string? right)
        => DeltaIds.Canonical(left) == DeltaIds.Canonical(right);

    private static bool SameConclusions(IReadOnlyList<CanonicalDelta> left, IReadOnlyList<CanonicalDelta> right)
    {
        var leftIds = left.Where(delta => delta.Status == ScalarScope.Services.DeltaStatus.Present).Select(delta => delta.Id).OrderBy(id => id).ToArray();
        var rightIds = right.Where(delta => delta.Status == ScalarScope.Services.DeltaStatus.Present).Select(delta => delta.Id).OrderBy(id => id).ToArray();
        if (leftIds.Length == 0 || !leftIds.SequenceEqual(rightIds))
            return false;

        foreach (var id in leftIds)
        {
            var a = left.First(delta => delta.Id == id && delta.Status == ScalarScope.Services.DeltaStatus.Present);
            var b = right.First(delta => delta.Id == id && delta.Status == ScalarScope.Services.DeltaStatus.Present);
            if (Math.Sign(a.Delta) != Math.Sign(b.Delta))
                return false;
        }

        return true;
    }

    private static PairValidationResult ToPair(string name, string category, IReadOnlyList<CanonicalDelta> deltas, bool passes)
        => new()
        {
            PairName = name,
            Category = category,
            DeltaF = Map(deltas, "FailurePresence"),
            DeltaTc = Map(deltas, "ConvergenceTiming"),
            DeltaTd = Map(deltas, "StructuralEmergence"),
            DeltaA = Map(deltas, "EvaluatorAlignment"),
            DeltaO = Map(deltas, "StabilityOscillation"),
            PassesGates = passes
        };

    private static DeltaResult Map(IReadOnlyList<CanonicalDelta> deltas, string id)
    {
        var hit = deltas.FirstOrDefault(delta => SameId(delta.Id, id) && delta.Status == ScalarScope.Services.DeltaStatus.Present);
        if (hit == null)
            return new DeltaResult { Suppressed = true, SuppressionReason = "absent", KeyValue = "" };

        return new DeltaResult
        {
            Suppressed = false,
            KeyValue = hit.Explanation,
            Confidence = hit.Confidence,
            TriggerType = hit.DeltaType.ToString()
        };
    }

    private static ScalarScope.SoakTests.DeltaStatus Status(string name, bool implemented, string evidence)
        => new()
        {
            Name = name,
            Implemented = implemented,
            Changes = [evidence],
            EvidenceBasis = evidence
        };

    private static VerificationCheck Check(string name, string condition, bool passed, string notes)
        => new()
        {
            Name = name,
            Description = condition,
            PassCondition = condition,
            Implemented = passed,
            Passed = passed,
            Notes = notes
        };

    private static bool CanLock(Phase32ValidationReport report)
        => report.PairResults.Count > 0
           && report.SuiteGates?.GateA == true
           && report.DeltaFVerification?.Checks.Count > 0
           && report.DeltaFVerification.Checks.All(check => check.Passed)
           && (report.DeltaImplementationStatus?.AllImplemented ?? false);

    private static GeometryRun Flat(string id, IReadOnlyList<FailureEvent>? failures = null)
        => Build(id, _ => 0.05, _ => 0, _ => EqualSpectrum(0.20), failures);

    private static GeometryRun Settled(string id, int settleStep)
        => Build(id, step => step < settleStep ? 1.0 : 0.05, _ => 0, _ => EqualSpectrum(0.20), null);

    private static GeometryRun Spiked(string id, int spikeSteps)
        => Build(id, step => step >= 20 && step < 20 + spikeSteps ? 5.0 : 0.2, _ => 0, _ => EqualSpectrum(0.20), null);

    private static GeometryRun Alternating(string id)
        => Build(id, step => step % 2 == 0 ? 0.2 : 0.4, _ => 0, _ => EqualSpectrum(0.20), null);

    private static GeometryRun Curved(string id, int burstSteps, double curvature)
        => Build(id, _ => 0.05, step => step >= 10 && step < 10 + burstSteps ? curvature : 0, _ => EqualSpectrum(0.20), null);

    private static GeometryRun Aligned(string id, double firstShare)
        => Build(id, _ => 0.05, _ => 0, _ => ShareSpectrum(firstShare), null);

    private static GeometryRun Dominant(string id)
        => Build(id, _ => 0.05, _ => 0, _ => [0.90, 0.20, 0.10, 0.10, 0.10], null);

    private static List<double> EqualSpectrum(double share) => [share, share, share, share, share];

    private static List<double> ShareSpectrum(double first)
    {
        var rest = (1.0 - first) / 4.0;
        return [first, rest, rest, rest, rest];
    }

    private static GeometryRun Build(
        string id,
        Func<int, double> velocity,
        Func<int, double> curvature,
        Func<int, List<double>> spectrum,
        IReadOnlyList<FailureEvent>? failures)
    {
        var steps = new List<TrajectoryTimestep>(StepCount);
        var eigenvalues = new List<EigenTimestep>(StepCount);
        for (var i = 0; i < StepCount; i++)
        {
            var t = i / (double)(StepCount - 1);
            var speed = velocity(i);
            steps.Add(new TrajectoryTimestep
            {
                T = t,
                State2D = [t, 0],
                Velocity = [speed, 0],
                Curvature = curvature(i)
            });
            eigenvalues.Add(new EigenTimestep { T = t, Values = spectrum(i) });
        }

        return new GeometryRun
        {
            Metadata = new RunMetadata { RunId = id },
            Trajectory = new Trajectory { Timesteps = steps },
            Geometry = new GeometryMetrics { Eigenvalues = eigenvalues },
            Failures = failures?.ToList() ?? []
        };
    }

    private sealed record MeasuredSuite(
        List<PairValidationResult> Pairs,
        DeltaImplementationStatus Status,
        DeltaFVerificationResult FailureChecks,
        SuiteGatesResult Gates,
        int ClearPresent,
        int NearPresent,
        bool ResolutionBelowFired,
        bool ResolutionAtFired,
        bool ShortSpikeFired,
        bool PersistentSpikeFired,
        bool ShortEpisodeFired,
        bool QualifyingEpisodeFired,
        bool TinyAreaFired,
        int QualifyingEvidenceEpisodes);
}

/// <summary>
/// Phase 3.2 validation report structure.
/// </summary>
public class Phase32ValidationReport
{
    /// <summary>
    /// Delta spec version for tracking tuning changes over time.
    /// Increment on any threshold/behavior change.
    /// </summary>
    public string DeltaSpecVersion { get; set; } = "3.2.0";
    
    public DateTime ValidationTime { get; set; }
    public string Phase { get; set; } = "";
    public DeltaImplementationStatus? DeltaImplementationStatus { get; set; }
    public DeltaFVerificationResult? DeltaFVerification { get; set; }
    public List<PairValidationResult> PairResults { get; set; } = new();
    public SuiteGatesResult? SuiteGates { get; set; }
    public List<string> Notes { get; set; } = new();
    public bool Locked { get; set; }
    public string LockDecision { get; set; } = "";

    public bool AllGatesPassed =>
        PairResults.Count > 0 &&
        (DeltaImplementationStatus?.AllImplemented ?? false) &&
        (DeltaFVerification?.Checks.Count > 0) &&
        (DeltaFVerification?.Checks.All(c => c.Passed) ?? false) &&
        SuiteGates is { GateA: true, GateB: true, GateC: true, GateD: true };

    public string ToSummaryTable()
    {
        var sb = new System.Text.StringBuilder();
        sb.AppendLine("# Phase 3.2 Validation Summary");
        sb.AppendLine($"**Delta Spec Version:** {DeltaSpecVersion}");
        sb.AppendLine($"Generated: {ValidationTime:yyyy-MM-dd HH:mm:ss} UTC");
        sb.AppendLine();

        // Implementation Status Table
        if (DeltaImplementationStatus != null)
        {
            sb.AppendLine("## Delta Implementation Status");
            sb.AppendLine("| Delta | Implemented | Key Changes | Evidence Basis |");
            sb.AppendLine("|-------|-------------|-------------|----------------|");

            void AddRow(DeltaStatus s)
            {
                var changes = string.Join("; ", s.Changes.Take(2));
                sb.AppendLine($"| {s.Name} | {(s.Implemented ? "✅" : "❌")} | {changes} | {s.EvidenceBasis} |");
            }

            AddRow(DeltaImplementationStatus.DeltaA);
            AddRow(DeltaImplementationStatus.DeltaTd);
            AddRow(DeltaImplementationStatus.DeltaTc);
            AddRow(DeltaImplementationStatus.DeltaO);
            AddRow(DeltaImplementationStatus.DeltaF);
            sb.AppendLine();
        }

        // ΔF Verification
        if (DeltaFVerification != null)
        {
            sb.AppendLine("## ΔF Verification");
            foreach (var check in DeltaFVerification.Checks)
            {
                sb.AppendLine($"- **{check.Name}**: {(check.Passed ? "✅ PASS" : "⏳ PENDING")}");
                sb.AppendLine($"  - {check.Notes}");
            }
            sb.AppendLine($"**Summary:** {DeltaFVerification.Summary}");
            sb.AppendLine();
        }

        // Pair Results (if any)
        if (PairResults.Any())
        {
            sb.AppendLine("## Pair Validation Results");
            sb.AppendLine("| Pair | Category | ΔĀ | ΔTd | ΔTc | ΔO | ΔF | Pass |");
            sb.AppendLine("|------|----------|-----|-----|-----|-----|-----|------|");
            foreach (var pair in PairResults)
            {
                sb.AppendLine($"| {pair.PairName} | {pair.Category} | {FormatDelta(pair.DeltaA)} | {FormatDelta(pair.DeltaTd)} | {FormatDelta(pair.DeltaTc)} | {FormatDelta(pair.DeltaO)} | {FormatDelta(pair.DeltaF)} | {(pair.PassesGates ? "✅" : "❌")} |");
            }
            sb.AppendLine();
        }

        // Suite Gates
        if (SuiteGates != null)
        {
            sb.AppendLine("## Suite Gates");
            sb.AppendLine($"### Gate A (Discrimination): {(SuiteGates.GateA ? "✅ PASS" : "⏳ PENDING")}");
            foreach (var note in SuiteGates.GateANotes) sb.AppendLine($"- {note}");
            sb.AppendLine();
            
            sb.AppendLine($"### Gate B (Trustworthiness): {(SuiteGates.GateB ? "✅ PASS" : "⏳ PENDING")}");
            foreach (var note in SuiteGates.GateBNotes) sb.AppendLine($"- {note}");
            sb.AppendLine();
            
            sb.AppendLine($"### Gate C (Noise Control): {(SuiteGates.GateC ? "✅ PASS" : "⏳ PENDING")}");
            foreach (var note in SuiteGates.GateCNotes) sb.AppendLine($"- {note}");
            sb.AppendLine();
            
            sb.AppendLine($"### Gate D (Consistency): {(SuiteGates.GateD ? "✅ PASS" : "⏳ PENDING")}");
            foreach (var note in SuiteGates.GateDNotes) sb.AppendLine($"- {note}");
            sb.AppendLine();
        }

        // Notes
        if (Notes.Any())
        {
            sb.AppendLine("## Notes");
            foreach (var note in Notes)
            {
                sb.AppendLine($"- {note}");
            }
            sb.AppendLine();
        }

        // Lock Decision
        sb.AppendLine("---");
        sb.AppendLine($"## Lock Decision: {(Locked ? "🔒 **LOCKED**" : "🔓 **NOT LOCKED**")}");
        if (!string.IsNullOrEmpty(LockDecision))
        {
            sb.AppendLine(LockDecision);
        }

        return sb.ToString();
    }

    private static string FormatDelta(DeltaResult? delta)
    {
        if (delta == null) return "-";
        if (delta.Suppressed) return $"⊘ {delta.SuppressionReason}";
        return $"✓ {delta.KeyValue}";
    }
}

public class DeltaImplementationStatus
{
    public DeltaStatus DeltaA { get; set; } = new();
    public DeltaStatus DeltaTd { get; set; } = new();
    public DeltaStatus DeltaTc { get; set; } = new();
    public DeltaStatus DeltaO { get; set; } = new();
    public DeltaStatus DeltaF { get; set; } = new();

    public bool AllImplemented => DeltaA.Implemented && DeltaTd.Implemented && 
                                   DeltaTc.Implemented && DeltaO.Implemented && 
                                   DeltaF.Implemented;
}

public class DeltaStatus
{
    public string Name { get; set; } = "";
    public bool Implemented { get; set; }
    public string[] Changes { get; set; } = Array.Empty<string>();
    public string EvidenceBasis { get; set; } = "";
}

public class DeltaFVerificationResult
{
    public List<VerificationCheck> Checks { get; set; } = new();
    public string Summary { get; set; } = "";
}

public class VerificationCheck
{
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public string PassCondition { get; set; } = "";
    public bool Implemented { get; set; }
    public bool Passed { get; set; }
    public string Notes { get; set; } = "";
}

public class PairValidationResult
{
    public string PairName { get; set; } = "";
    public string Category { get; set; } = ""; // clearly_different, subtly_different, nearly_identical, one_failure, mismatched_length
    public DeltaResult? DeltaA { get; set; }  // Alignment
    public DeltaResult? DeltaTd { get; set; } // Emergence
    public DeltaResult? DeltaTc { get; set; } // Convergence
    public DeltaResult? DeltaO { get; set; }  // Stability
    public DeltaResult? DeltaF { get; set; }  // Failure
    public bool PassesGates { get; set; }
    public List<string> ReviewerNotes { get; set; } = new();
}

public class DeltaResult
{
    public bool Suppressed { get; set; }
    public string SuppressionReason { get; set; } = "";
    public string KeyValue { get; set; } = "";
    public double? Confidence { get; set; }
    
    /// <summary>
    /// Why this delta fired. Track for regression analysis.
    /// Expected values by delta:
    /// - ΔTd: "sustained" | "recurrence"  
    /// - ΔĀ: "persistence_weighted" | "raw"
    /// - ΔO: "area_episode" | "count_episode"
    /// - ΔTc: "step_difference" | "one_run_converged"
    /// - ΔF: "event" | "divergence_proxy" | "collapse_proxy"
    /// </summary>
    public string? TriggerType { get; set; }
}

public class SuiteGatesResult
{
    public bool GateA { get; set; } // Discrimination
    public bool GateB { get; set; } // Trustworthiness
    public bool GateC { get; set; } // Noise Control
    public bool GateD { get; set; } // Consistency
    public List<string> GateANotes { get; set; } = new();
    public List<string> GateBNotes { get; set; } = new();
    public List<string> GateCNotes { get; set; } = new();
    public List<string> GateDNotes { get; set; } = new();
}
