using System.Globalization;
using System.Text.Json;
using ScalarScope.Models;

namespace ScalarScope.Services;

/// <summary>
/// Phase 6.1: Determinism test suite for verifying reproducibility.
/// Can be run from the app or as part of CI.
/// </summary>
public static class DeterminismTestSuite
{
    /// <summary>
    /// Run all determinism tests and return results.
    /// </summary>
    public static DeterminismTestResults RunAllTests()
    {
        var results = new DeterminismTestResults();
        
        results.Add(TestInputNormalization());
        results.Add(TestFingerprintDeterminism());
        results.Add(TestDeltaHashDeterminism());
        results.Add(TestDoubleNormalization());
        results.Add(TestRunIdNormalization());
        results.Add(TestInstabilityAlignmentExport());
        results.Add(TestSingleStepAlignment());
        results.Add(TestEnumSchemaRoundTrip());
        results.Add(TestFiveDeltaIdRoundTrip());
        
        return results;
    }
    
    /// <summary>
    /// Test that input normalization is consistent.
    /// </summary>
    public static DeterminismTestResult TestInputNormalization()
    {
        try
        {
            // Same logical input with different formatting
            var input1 = InputNormalizer.NormalizeComparisonInput(
                "Run_A ", "  run-b", 100, 100, 0);
            var input2 = InputNormalizer.NormalizeComparisonInput(
                "run_a", "RUN-B", 100, 100, 0);
            
            var passed = input1.CanonicalForm == input2.CanonicalForm;
            
            return new DeterminismTestResult
            {
                TestName = "InputNormalization",
                Passed = passed,
                Message = passed 
                    ? "Normalized inputs match" 
                    : $"Mismatch: '{input1.CanonicalForm}' vs '{input2.CanonicalForm}'"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "InputNormalization",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }
    
    /// <summary>
    /// Test that fingerprints are deterministic across calls.
    /// </summary>
    public static DeterminismTestResult TestFingerprintDeterminism()
    {
        try
        {
            var fp1 = DeterminismService.ComputeInputFingerprint("run_a", "run_b", 0, 100, 100);
            var fp2 = DeterminismService.ComputeInputFingerprint("run_a", "run_b", 0, 100, 100);
            var fp3 = DeterminismService.ComputeInputFingerprint("run_a", "run_b", 0, 100, 100);

            var baseRun = SampleRun("alpha", 1.0);
            var renamed = SampleRun("beta", 1.0);
            var moved = SampleRun("alpha", 9.0);
            var same = DeterminismService.HashRun(baseRun) == DeterminismService.HashRun(SampleRun("alpha", 1.0));
            var idChanges = DeterminismService.HashRun(baseRun) != DeterminismService.HashRun(renamed);
            var measurementChanges = DeterminismService.HashRun(baseRun) != DeterminismService.HashRun(moved);
            var fullLength = fp1.Length == 64 && DeterminismService.HashRun(baseRun).Length == 64;
            
            var passed = fp1 == fp2 && fp2 == fp3 && same && idChanges && measurementChanges && fullLength;
            
            return new DeterminismTestResult
            {
                TestName = "FingerprintDeterminism",
                Passed = passed,
                Message = passed 
                    ? $"Fingerprint consistent: {fp1}" 
                    : $"Fingerprint varied: {fp1}, {fp2}, {fp3}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "FingerprintDeterminism",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }
    
    /// <summary>
    /// Test that delta hashes are deterministic for same input.
    /// </summary>
    public static DeterminismTestResult TestDeltaHashDeterminism()
    {
        try
        {
            var testDeltas = new[]
            {
                new CanonicalDelta
                {
                    Id = DeltaIds.FailurePresence,
                    Status = DeltaStatus.Present,
                    Confidence = 0.95,
                    LeftValue = 1.25,
                    RightValue = 2.5,
                    Delta = 1.25,
                    Explanation = "Test"
                },
                new CanonicalDelta
                {
                    Id = DeltaIds.ConvergenceTiming,
                    Status = DeltaStatus.Suppressed,
                    Confidence = 0.0,
                    Delta = 0,
                    Explanation = ""
                }
            };

            var originalCulture = CultureInfo.CurrentCulture;
            string hash1;
            string hash2;
            string hash3;
            string changed;
            try
            {
                CultureInfo.CurrentCulture = CultureInfo.GetCultureInfo("en-US");
                hash1 = DeterminismService.ComputeDeltaHash(testDeltas);
                hash2 = DeterminismService.ComputeDeltaHash(testDeltas);
                CultureInfo.CurrentCulture = CultureInfo.GetCultureInfo("de-DE");
                hash3 = DeterminismService.ComputeDeltaHash(testDeltas);
                var moved = new[] { testDeltas[0] with { Delta = 9.5 }, testDeltas[1] };
                changed = DeterminismService.ComputeDeltaHash(moved);
            }
            finally
            {
                CultureInfo.CurrentCulture = originalCulture;
            }
            
            var passed = hash1 == hash2 && hash2 == hash3
                && hash1 != changed
                && hash1.Length == 64;
            
            return new DeterminismTestResult
            {
                TestName = "DeltaHashDeterminism",
                Passed = passed,
                Message = passed 
                    ? $"Delta hash consistent: {hash1}" 
                    : $"Delta hash varied: {hash1}, {hash2}, {hash3}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "DeltaHashDeterminism",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }
    
    /// <summary>
    /// Test double normalization handles edge cases.
    /// </summary>
    public static DeterminismTestResult TestDoubleNormalization()
    {
        try
        {
            var nan = InputNormalizer.NormalizeDouble(double.NaN);
            if (!double.IsNaN(nan))
            {
                return new DeterminismTestResult
                {
                    TestName = "DoubleNormalization",
                    Passed = false,
                    Message = "NaN must stay missing, not become zero"
                };
            }

            var tests = new (double input, double expected)[]
            {
                (0.123456789012345, 0.1234567890),
                (double.PositiveInfinity, double.MaxValue),
                (double.NegativeInfinity, double.MinValue),
                (1.0 / 3.0, 0.3333333333)
            };
            
            var failures = new List<string>();
            foreach (var (input, expected) in tests)
            {
                var actual = InputNormalizer.NormalizeDouble(input);
                if (Math.Abs(actual - expected) > 1e-12)
                {
                    failures.Add($"{input} -> {actual} (expected {expected})");
                }
            }
            
            var passed = failures.Count == 0;
            
            return new DeterminismTestResult
            {
                TestName = "DoubleNormalization",
                Passed = passed,
                Message = passed 
                    ? "All double normalizations correct" 
                    : $"Failures: {string.Join(", ", failures)}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "DoubleNormalization",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }
    
    /// <summary>
    /// Test run ID normalization handles edge cases.
    /// </summary>
    public static DeterminismTestResult TestRunIdNormalization()
    {
        try
        {
            var tests = new (string? input, string expected)[]
            {
                (null, "UNKNOWN"),
                ("", "UNKNOWN"),
                ("  ", "UNKNOWN"),
                ("Run_A", "run_a"),
                ("  Run-B  ", "run-b"),
                ("RUN C", "run_c")
            };
            
            var failures = new List<string>();
            foreach (var (input, expected) in tests)
            {
                var actual = InputNormalizer.NormalizeRunId(input);
                if (actual != expected)
                {
                    failures.Add($"'{input}' -> '{actual}' (expected '{expected}')");
                }
            }
            
            var passed = failures.Count == 0;
            
            return new DeterminismTestResult
            {
                TestName = "RunIdNormalization",
                Passed = passed,
                Message = passed 
                    ? "All run ID normalizations correct" 
                    : $"Failures: {string.Join(", ", failures)}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "RunIdNormalization",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }

    /// <summary>
    /// ByFirstInstability stays that mode, and the bundle token is firstInstability.
    /// </summary>
    public static DeterminismTestResult TestInstabilityAlignmentExport()
    {
        try
        {
            var left = SampleRun("left", 0, 0, 0, 1, 0);
            var right = SampleRun("right", 0, 0, 0, 1, 0);
            var map = AlignmentMapper.CreateAlignmentMap(left, right, TemporalAlignment.ByFirstInstability);
            var token = JsonSerializer.Serialize(Bundles.AlignmentMode.FirstInstability);
            var passed = map.Mode == TemporalAlignment.ByFirstInstability
                && token == "\"firstInstability\"";

            return new DeterminismTestResult
            {
                TestName = "InstabilityAlignmentExport",
                Passed = passed,
                Message = passed
                    ? "Instability alignment keeps firstInstability"
                    : $"Mode {map.Mode}, token {token}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "InstabilityAlignmentExport",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }

    /// <summary>
    /// A one-step trajectory maps without dividing by zero.
    /// </summary>
    public static DeterminismTestResult TestSingleStepAlignment()
    {
        try
        {
            var left = SampleRun("left", 0.2);
            var right = SampleRun("right", 0.4);
            var map = AlignmentMapper.CreateAlignmentMap(left, right, TemporalAlignment.ByStep);
            var passed = map.IdxToStepA.Length == 1
                && map.IdxToStepA[0] == 0
                && map.IdxToStepB[0] == 0;

            return new DeterminismTestResult
            {
                TestName = "SingleStepAlignment",
                Passed = passed,
                Message = passed ? "One-step runs map to step 0" : "One-step alignment did not map both sides to 0"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "SingleStepAlignment",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }

    /// <summary>
    /// Enum wire values are the manifest schema tokens.
    /// </summary>
    public static DeterminismTestResult TestEnumSchemaRoundTrip()
    {
        try
        {
            var options = Bundles.BundleHashAlgorithm.GetCanonicalOptions();
            var share = JsonSerializer.Serialize(Bundles.BundleProfile.Share, options);
            var algorithm = JsonSerializer.Serialize(Bundles.HashAlgorithm.Sha256, options);
            var changed = JsonSerializer.Serialize(Bundles.ReproReason.InputsChanged, options);
            var alignment = JsonSerializer.Serialize(Bundles.AlignmentMode.FirstInstability, options);
            var back = JsonSerializer.Deserialize<Bundles.AlignmentMode>(alignment, options);

            var passed = share == "\"share\""
                && algorithm == "\"SHA-256\""
                && changed == "\"inputs_changed\""
                && alignment == "\"firstInstability\""
                && back == Bundles.AlignmentMode.FirstInstability;

            return new DeterminismTestResult
            {
                TestName = "EnumSchemaRoundTrip",
                Passed = passed,
                Message = passed
                    ? "Schema enum tokens round-trip"
                    : $"share={share} hash={algorithm} reason={changed} align={alignment} back={back}"
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "EnumSchemaRoundTrip",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }

    /// <summary>
    /// Each of the five detector ids survives an alias round trip with its delta type.
    /// </summary>
    public static DeterminismTestResult TestFiveDeltaIdRoundTrip()
    {
        try
        {
            (string alias, string canonical, DeltaType type)[] rows =
            [
                ("FailurePresence", DeltaIds.FailurePresence, DeltaType.Event),
                ("delta_tc", DeltaIds.ConvergenceTiming, DeltaType.Timing),
                ("structuralEmergence", DeltaIds.StructuralEmergence, DeltaType.Timing),
                ("delta_a", DeltaIds.EvaluatorAlignment, DeltaType.Structure),
                ("StabilityOscillation", DeltaIds.StabilityOscillation, DeltaType.Behavior)
            ];

            var failures = new List<string>();
            foreach (var (alias, canonical, type) in rows)
            {
                var id = DeltaIds.Canonical(alias);
                var mapped = DeltaIds.ToDeltaType(alias);
                if (id != canonical || mapped != type)
                    failures.Add($"{alias} -> {id}/{mapped}");
            }

            var passed = failures.Count == 0;
            return new DeterminismTestResult
            {
                TestName = "FiveDeltaIdRoundTrip",
                Passed = passed,
                Message = passed
                    ? "Five delta ids round-trip"
                    : string.Join("; ", failures)
            };
        }
        catch (Exception ex)
        {
            return new DeterminismTestResult
            {
                TestName = "FiveDeltaIdRoundTrip",
                Passed = false,
                Message = $"Exception: {ex.Message}"
            };
        }
    }

    private static GeometryRun SampleRun(string runId, params double[] measurements)
    {
        var steps = new List<TrajectoryTimestep>();
        if (measurements.Length == 0)
            measurements = [0];

        foreach (var value in measurements)
        {
            steps.Add(new TrajectoryTimestep
            {
                State2D = [value, 0],
                Velocity = [0.1, 0],
                Curvature = value
            });
        }

        return new GeometryRun
        {
            Metadata = new RunMetadata { RunId = runId },
            Trajectory = new Trajectory { Timesteps = steps }
        };
    }
}

/// <summary>
/// Result of a single determinism test.
/// </summary>
public record DeterminismTestResult
{
    public required string TestName { get; init; }
    public bool Passed { get; init; }
    public required string Message { get; init; }
}

/// <summary>
/// Collection of determinism test results.
/// </summary>
public class DeterminismTestResults
{
    private readonly List<DeterminismTestResult> _results = new();
    
    public IReadOnlyList<DeterminismTestResult> Results => _results;
    public int TotalTests => _results.Count;
    public int PassedTests => _results.Count(r => r.Passed);
    public int FailedTests => _results.Count(r => !r.Passed);
    public bool AllPassed => _results.All(r => r.Passed);
    
    public void Add(DeterminismTestResult result) => _results.Add(result);
    
    public override string ToString()
    {
        var sb = new System.Text.StringBuilder();
        sb.AppendLine($"Determinism Test Results: {PassedTests}/{TotalTests} passed");
        sb.AppendLine(new string('=', 50));
        
        foreach (var result in _results)
        {
            var status = result.Passed ? "✓" : "✗";
            sb.AppendLine($"{status} {result.TestName}: {result.Message}");
        }
        
        return sb.ToString();
    }
}
