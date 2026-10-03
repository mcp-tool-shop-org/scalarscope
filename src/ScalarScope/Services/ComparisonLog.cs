using System.Text.Json;
using System.Text.Json.Serialization;

namespace ScalarScope.Services;

/// <summary>
/// Local history of finished comparisons. The file lives in app data.
/// It is not sent anywhere.
/// </summary>
public static class ComparisonLog
{
    public const int MaxEntries = 40;
    public const string FileName = "comparison-log.json";

    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    private static readonly string[] SymbolOrder = ["ΔF", "ΔTc", "ΔTd", "ΔĀ", "ΔO"];

    public static string DefaultDirectory => FileSystem.AppDataDirectory;

    public static string FilePathFor(string directory) => Path.Combine(directory, FileName);

    /// <summary>
    /// Insert a finished review at the front. The same two runs, or the same bundle, update one entry.
    /// </summary>
    public static ComparisonLogEntry Record(ComparisonLogEntry incoming, string directory)
    {
        ArgumentNullException.ThrowIfNull(incoming);
        ArgumentException.ThrowIfNullOrWhiteSpace(directory);

        var entries = Read(directory).ToList();
        var match = entries.FirstOrDefault(existing => SameSitting(existing, incoming));
        var saved = Copy(incoming);
        saved.Id = string.IsNullOrWhiteSpace(match?.Id) ? Guid.NewGuid().ToString("n") : match.Id;
        saved.FinishedAt = DateTimeOffset.UtcNow;
        saved.DeltasFired ??= [];
        saved.Kind = string.IsNullOrWhiteSpace(saved.Kind) ? "compare" : saved.Kind;

        if (match != null)
        {
            if (string.IsNullOrWhiteSpace(saved.BundlePath))
            {
                saved.BundlePath = match.BundlePath;
                saved.BundleHash = match.BundleHash;
            }

            entries.Remove(match);
        }

        entries.Insert(0, saved);
        if (entries.Count > MaxEntries)
            entries = entries.Take(MaxEntries).ToList();

        Write(directory, entries);
        return saved;
    }

    /// <summary>
    /// Stamp the newest live review with the bundle that was just written.
    /// A review that is already a different bundle is left alone, and this export is appended.
    /// </summary>
    public static ComparisonLogEntry AttachBundle(string bundlePath, string? bundleHash, string directory)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(bundlePath);
        var entries = Read(directory).ToList();
        var top = entries.FirstOrDefault();

        if (top != null && !string.Equals(top.Kind, "bundle", StringComparison.Ordinal))
        {
            top.BundlePath = bundlePath;
            top.BundleHash = bundleHash;
            top.FinishedAt = DateTimeOffset.UtcNow;
            Write(directory, entries);
            return top;
        }

        if (top != null
            && string.Equals(top.Kind, "bundle", StringComparison.Ordinal)
            && string.Equals(top.BundlePath, bundlePath, StringComparison.OrdinalIgnoreCase))
        {
            top.BundleHash = bundleHash;
            top.FinishedAt = DateTimeOffset.UtcNow;
            Write(directory, entries);
            return top;
        }

        return Record(new ComparisonLogEntry
        {
            Kind = "bundle",
            LeftName = Path.GetFileNameWithoutExtension(bundlePath),
            BundlePath = bundlePath,
            BundleHash = bundleHash
        }, directory);
    }

    public static IReadOnlyList<ComparisonLogEntry> Read(string directory)
    {
        var path = FilePathFor(directory);
        if (!File.Exists(path))
            return [];

        try
        {
            var json = File.ReadAllText(path);
            return JsonSerializer.Deserialize<List<ComparisonLogEntry>>(json, Options) ?? [];
        }
        catch (Exception ex) when (ex is JsonException or IOException)
        {
            return [];
        }
    }

    public static void Clear(string directory)
    {
        var path = FilePathFor(directory);
        if (File.Exists(path))
            File.Delete(path);
    }

    /// <summary>
    /// Short symbols for deltas that are present, in canonical order.
    /// </summary>
    public static IReadOnlyList<string> SymbolsFor(IEnumerable<CanonicalDelta>? deltas)
    {
        if (deltas == null)
            return [];

        var present = new HashSet<string>(StringComparer.Ordinal);
        foreach (var delta in deltas)
        {
            if (delta.Status != DeltaStatus.Present)
                continue;

            var symbol = SymbolForId(delta.Id);
            if (symbol != null)
                present.Add(symbol);
        }

        return SymbolOrder.Where(present.Contains).ToList();
    }

    public static string? SymbolForId(string? id)
    {
        return DeltaIds.Canonical(id) switch
        {
            DeltaIds.FailurePresence => "ΔF",
            DeltaIds.ConvergenceTiming => "ΔTc",
            DeltaIds.StructuralEmergence => "ΔTd",
            DeltaIds.EvaluatorAlignment => "ΔĀ",
            DeltaIds.StabilityOscillation => "ΔO",
            _ => null
        };
    }

    private static bool SameSitting(ComparisonLogEntry existing, ComparisonLogEntry incoming)
    {
        if (string.Equals(incoming.Kind, "bundle", StringComparison.Ordinal))
        {
            return string.Equals(existing.Kind, "bundle", StringComparison.Ordinal)
                && !string.IsNullOrWhiteSpace(incoming.BundlePath)
                && string.Equals(existing.BundlePath, incoming.BundlePath, StringComparison.OrdinalIgnoreCase);
        }

        if (!string.IsNullOrWhiteSpace(incoming.LeftPath) && !string.IsNullOrWhiteSpace(incoming.RightPath))
        {
            return string.Equals(existing.LeftPath, incoming.LeftPath, StringComparison.OrdinalIgnoreCase)
                && string.Equals(existing.RightPath, incoming.RightPath, StringComparison.OrdinalIgnoreCase)
                && !string.Equals(existing.Kind, "bundle", StringComparison.Ordinal);
        }

        return string.Equals(existing.Kind, incoming.Kind, StringComparison.Ordinal)
            && string.Equals(existing.LeftName, incoming.LeftName, StringComparison.Ordinal)
            && string.Equals(existing.RightName, incoming.RightName, StringComparison.Ordinal)
            && string.IsNullOrWhiteSpace(existing.LeftPath)
            && string.IsNullOrWhiteSpace(incoming.LeftPath);
    }

    private static ComparisonLogEntry Copy(ComparisonLogEntry source)
    {
        return new ComparisonLogEntry
        {
            Id = source.Id,
            FinishedAt = source.FinishedAt,
            LeftName = source.LeftName,
            RightName = source.RightName,
            LeftPath = source.LeftPath,
            RightPath = source.RightPath,
            LeftRunId = source.LeftRunId,
            RightRunId = source.RightRunId,
            BundlePath = source.BundlePath,
            BundleHash = source.BundleHash,
            Alignment = source.Alignment,
            DeltasFired = source.DeltasFired?.ToList() ?? [],
            Kind = source.Kind
        };
    }

    private static void Write(string directory, List<ComparisonLogEntry> entries)
    {
        Directory.CreateDirectory(directory);
        var json = JsonSerializer.Serialize(entries, Options);
        File.WriteAllText(FilePathFor(directory), json);
    }
}

/// <summary>
/// One finished review. Home binds the computed title, subtitle, and time.
/// </summary>
public sealed class ComparisonLogEntry
{
    public string Id { get; set; } = "";
    public DateTimeOffset FinishedAt { get; set; }
    public string LeftName { get; set; } = "";
    public string RightName { get; set; } = "";
    public string? LeftPath { get; set; }
    public string? RightPath { get; set; }
    public string? LeftRunId { get; set; }
    public string? RightRunId { get; set; }
    public string? BundlePath { get; set; }
    public string? BundleHash { get; set; }
    public string Alignment { get; set; } = "";
    public List<string> DeltasFired { get; set; } = [];

    /// <summary>compare, example, or bundle.</summary>
    public string Kind { get; set; } = "compare";

    [JsonIgnore]
    public string Title => string.Equals(Kind, "bundle", StringComparison.Ordinal) || string.IsNullOrWhiteSpace(RightName)
        ? LeftName
        : $"{LeftName} vs {RightName}";

    [JsonIgnore]
    public string Subtitle
    {
        get
        {
            var deltas = DeltasFired is { Count: > 0 }
                ? string.Join(" · ", DeltasFired)
                : "No deltas fired";

            if (!string.IsNullOrWhiteSpace(BundleHash) && BundleHash.Length >= 8)
                return $"{deltas} · {BundleHash[..8]}";

            return deltas;
        }
    }

    [JsonIgnore]
    public string TimeAgo
    {
        get
        {
            var elapsed = DateTimeOffset.UtcNow - FinishedAt;
            if (elapsed.TotalMinutes < 1) return "Just now";
            if (elapsed.TotalMinutes < 60) return $"{(int)elapsed.TotalMinutes}m ago";
            if (elapsed.TotalHours < 24) return $"{(int)elapsed.TotalHours}h ago";
            if (elapsed.TotalDays < 7) return $"{(int)elapsed.TotalDays}d ago";
            return FinishedAt.LocalDateTime.ToString("MMM d");
        }
    }
}
