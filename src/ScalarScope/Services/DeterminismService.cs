using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using ScalarScope.Models;

namespace ScalarScope.Services;

/// <summary>
/// Phase 6.1: Determinism service for reproducible analysis.
/// Ensures identical inputs produce identical outputs.
/// </summary>
public static class DeterminismService
{
    /// <summary>
    /// Current determinism seed. Set to null for non-deterministic behavior.
    /// </summary>
    public static int? Seed { get; private set; }
    
    /// <summary>
    /// Whether deterministic mode is enabled.
    /// </summary>
    public static bool IsDeterministic => Seed.HasValue;
    
    /// <summary>
    /// Not a source of truth. Fingerprints are carried on the comparison result.
    /// Kept so older readers do not fail to compile; new comparisons do not write it.
    /// </summary>
    public static string? LastFingerprint { get; private set; }
    
    /// <summary>
    /// Enable deterministic mode with a specific seed.
    /// </summary>
    public static void EnableDeterminism(int seed)
    {
        Seed = seed;
        _random = new Random(seed);
    }
    
    /// <summary>
    /// Enable deterministic mode with seed derived from input data.
    /// This ensures same inputs always produce same seed.
    /// </summary>
    public static void EnableDeterminismFromInput(string inputFingerprint)
    {
        var hash = ComputeHash(inputFingerprint);
        Seed = BitConverter.ToInt32(hash, 0);
        _random = new Random(Seed.Value);
    }
    
    /// <summary>
    /// Disable deterministic mode (use system randomness).
    /// </summary>
    public static void DisableDeterminism()
    {
        Seed = null;
        _random = null;
    }
    
    private static Random? _random;
    
    /// <summary>
    /// Get a deterministic random value (0.0 to 1.0).
    /// Falls back to system random if not in deterministic mode.
    /// </summary>
    public static double NextDouble()
    {
        return (_random ?? Random.Shared).NextDouble();
    }
    
    /// <summary>
    /// Get a deterministic random integer.
    /// </summary>
    public static int NextInt(int maxExclusive)
    {
        return (_random ?? Random.Shared).Next(maxExclusive);
    }
    
    /// <summary>
    /// Compute a deterministic fingerprint for comparison inputs.
    /// Same inputs will always produce same fingerprint.
    /// </summary>
    public static string ComputeInputFingerprint(
        string? leftRunId,
        string? rightRunId,
        int alignmentMode,
        int leftTimestepCount,
        int rightTimestepCount)
    {
        // Phase 6.1: Use normalized inputs for consistent fingerprinting
        var normalized = InputNormalizer.NormalizeComparisonInput(
            leftRunId,
            rightRunId,
            leftTimestepCount,
            rightTimestepCount,
            alignmentMode);
        
        return ComputeHashString(normalized.CanonicalForm);
    }

    /// <summary>
    /// SHA-256 of one run's canonical measurements. Null means the measurements are absent.
    /// </summary>
    public static string HashRun(GeometryRun? run)
    {
        return ComputeHashString(run is null ? "absent" : CanonicalRun(run));
    }

    /// <summary>
    /// Combine two per-run fingerprints without collapsing them into one shared value.
    /// </summary>
    public static string CombineRunFingerprints(string leftFingerprint, string rightFingerprint)
    {
        return ComputeHashString(leftFingerprint + "\n" + rightFingerprint);
    }
    
    /// <summary>
    /// Hash the numeric delta fields the detector actually produces.
    /// </summary>
    public static string ComputeDeltaHash(IEnumerable<CanonicalDelta> deltas)
    {
        var sb = new StringBuilder();
        foreach (var delta in deltas)
        {
            sb.Append("id=").Append(DeltaIds.Canonical(delta.Id));
            sb.Append(";status=").Append(delta.Status);
            sb.Append(";left=").Append(FormatMeasurement(delta.LeftValue));
            sb.Append(";right=").Append(FormatMeasurement(delta.RightValue));
            sb.Append(";delta=").Append(FormatMeasurement(delta.Delta));
            sb.Append(";magnitude=").Append(FormatMeasurement(delta.Magnitude));
            sb.Append(";confidence=").Append(FormatMeasurement(delta.Confidence));
            sb.Append('\n');
        }
        return ComputeHashString(sb.ToString());
    }

    /// <summary>
    /// Compute a deterministic hash for delta outputs.
    /// Used to verify reproducibility.
    /// </summary>
    public static string ComputeDeltaHash(IEnumerable<object> deltas)
    {
        var serialized = JsonSerializer.Serialize(deltas, new JsonSerializerOptions
        {
            WriteIndented = false,
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase
        });
        return ComputeHashString(serialized);
    }
    
    /// <summary>
    /// Verify that current outputs match expected hash.
    /// </summary>
    public static DeterminismVerification VerifyDeterminism(
        string expectedHash,
        IEnumerable<object> actualDeltas)
    {
        var actualHash = ComputeDeltaHash(actualDeltas);
        var matches = string.Equals(expectedHash, actualHash, StringComparison.OrdinalIgnoreCase);
        
        return new DeterminismVerification
        {
            ExpectedHash = expectedHash,
            ActualHash = actualHash,
            IsMatch = matches,
            Seed = Seed,
            Timestamp = DateTime.UtcNow
        };
    }
    
    /// <summary>
    /// Get reproducibility metadata for exports.
    /// </summary>
    public static ReproducibilityMetadata GetReproducibilityMetadata(string? inputFingerprint = null)
    {
        return new ReproducibilityMetadata
        {
            IsDeterministic = IsDeterministic,
            Seed = Seed,
            InputFingerprint = inputFingerprint,
            Version = VersionInfo.Version,
            Timestamp = DateTime.UtcNow
        };
    }

    private static string CanonicalRun(GeometryRun run)
    {
        var sb = new StringBuilder();
        sb.Append("id=").Append(InputNormalizer.NormalizeRunId(run.Metadata?.RunId));
        var steps = run.Trajectory?.Timesteps;
        sb.Append(";n=").Append(steps?.Count ?? 0);
        if (steps != null)
        {
            for (int i = 0; i < steps.Count; i++)
            {
                var step = steps[i];
                sb.Append("|t=").Append(i.ToString(CultureInfo.InvariantCulture));
                sb.Append(";x=").Append(FormatMeasurement(step.State2D.Count > 0 ? step.State2D[0] : double.NaN));
                sb.Append(";y=").Append(FormatMeasurement(step.State2D.Count > 1 ? step.State2D[1] : double.NaN));
                sb.Append(";v=").Append(FormatMeasurement(step.VelocityMagnitude));
                sb.Append(";c=").Append(FormatMeasurement(step.Curvature));
            }
        }

        var eigenvalues = run.Geometry?.Eigenvalues;
        if (eigenvalues != null)
        {
            for (int i = 0; i < eigenvalues.Count; i++)
            {
                sb.Append("|e=").Append(i.ToString(CultureInfo.InvariantCulture));
                var values = eigenvalues[i].Values;
                if (values == null || values.Count == 0)
                {
                    sb.Append(":missing");
                    continue;
                }
                foreach (var value in values)
                    sb.Append(':').Append(FormatMeasurement(value));
            }
        }

        return sb.ToString();
    }

    private static string FormatMeasurement(double value)
    {
        if (double.IsNaN(value))
            return "missing";
        if (double.IsPositiveInfinity(value))
            return "inf";
        if (double.IsNegativeInfinity(value))
            return "-inf";
        return value.ToString("R", CultureInfo.InvariantCulture);
    }
    
    private static byte[] ComputeHash(string input)
    {
        return SHA256.HashData(Encoding.UTF8.GetBytes(input));
    }
    
    private static string ComputeHashString(string input)
    {
        var hash = ComputeHash(input);
        return Convert.ToHexString(hash).ToLowerInvariant();
    }
}

/// <summary>
/// Result of determinism verification.
/// </summary>
public record DeterminismVerification
{
    public required string ExpectedHash { get; init; }
    public required string ActualHash { get; init; }
    public required bool IsMatch { get; init; }
    public int? Seed { get; init; }
    public DateTime Timestamp { get; init; }
    
    public override string ToString() => 
        IsMatch 
            ? $"✓ Determinism verified (hash: {ActualHash})"
            : $"✗ Determinism mismatch: expected {ExpectedHash}, got {ActualHash}";
}

/// <summary>
/// Metadata for reproducibility in exports.
/// </summary>
public record ReproducibilityMetadata
{
    public bool IsDeterministic { get; init; }
    public int? Seed { get; init; }
    public string? InputFingerprint { get; init; }
    public string? Version { get; init; }
    public DateTime Timestamp { get; init; }
}
