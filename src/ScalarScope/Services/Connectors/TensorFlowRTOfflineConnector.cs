// TensorFlowRT Offline Connector
// Parses TFRT profiler exports, CSVs, and logs into RuntimeRunTrace.
// Enables before/after optimization comparison for inference workloads.

using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace ScalarScope.Services.Connectors;

#region Error Codes

/// <summary>
/// TFRT-specific error codes.
/// All routed through Phase 6 ErrorExplanationService.
/// </summary>
public static partial class TfrtErrorCodes
{
    /// <summary>No supported TFRT export found at source.</summary>
    public const string TFRT_NO_SUPPORTED_EXPORT = "TFRT_NO_SUPPORTED_EXPORT";
    
    /// <summary>File unreadable or malformed.</summary>
    public const string TFRT_PARSE_FAILED = "TFRT_PARSE_FAILED";
    
    /// <summary>Cannot extract latency signal from source.</summary>
    public const string TFRT_NO_LATENCY_SIGNAL = "TFRT_NO_LATENCY_SIGNAL";
    
    /// <summary>Parsed series have inconsistent lengths.</summary>
    public const string TFRT_INCONSISTENT_LENGTHS = "TFRT_INCONSISTENT_LENGTHS";
    
    /// <summary>Cannot normalize units (ambiguous).</summary>
    public const string TFRT_UNIT_AMBIGUOUS = "TFRT_UNIT_AMBIGUOUS";
    
    // Legacy codes (kept for compatibility)
    
    /// <summary>Timeline data is inconsistent (non-monotonic steps).</summary>
    public const string TFRT_TIMELINE_INCONSISTENT = "TFRT_TIMELINE_INCONSISTENT";
    
    /// <summary>Metric units could not be determined.</summary>
    public const string TFRT_UNITS_UNKNOWN = "TFRT_UNITS_UNKNOWN";
    
    /// <summary>Profile data parsing failed.</summary>
    public const string TFRT_PROFILE_PARSE_FAILED = "TFRT_PROFILE_PARSE_FAILED";
    
    /// <summary>SavedModel could not be fingerprinted.</summary>
    public const string TFRT_MODEL_FINGERPRINT_FAILED = "TFRT_MODEL_FINGERPRINT_FAILED";
    
    /// <summary>CSV has unexpected format.</summary>
    public const string TFRT_CSV_FORMAT_ERROR = "TFRT_CSV_FORMAT_ERROR";
}

#endregion

#region TFRT Source Detection

/// <summary>
/// Detected TFRT source type.
/// Priority order: ProfilerTrace > ProfilerOverview > BenchmarkCsv > BenchmarkJson > RuntimeLog
/// </summary>
public enum TfrtSourceType
{
    /// <summary>Profiler trace.json (highest fidelity).</summary>
    ProfilerTrace,
    
    /// <summary>Profiler overview.json.</summary>
    ProfilerOverview,
    
    /// <summary>Benchmark CSV export.</summary>
    BenchmarkCsv,
    
    /// <summary>Benchmark JSON export.</summary>
    BenchmarkJson,
    
    /// <summary>Runtime log files (last resort).</summary>
    RuntimeLog,
    
    /// <summary>Unknown source.</summary>
    Unknown
}

/// <summary>
/// Detected TFRT source.
/// </summary>
public sealed record TfrtSource
{
    public required TfrtSourceType Type { get; init; }
    public required string Path { get; init; }
    
    /// <summary>Priority for selection (higher = preferred).</summary>
    public int Priority { get; init; }
    
    /// <summary>Additional context files found nearby.</summary>
    public TfrtFolderContext? Context { get; init; }
}

/// <summary>
/// Context from the TFRT folder structure.
/// </summary>
public sealed record TfrtFolderContext
{
    /// <summary>Path to saved_model directory (if found).</summary>
    public string? SavedModelPath { get; init; }
    
    /// <summary>Path to config.json (if found).</summary>
    public string? ConfigPath { get; init; }
    
    /// <summary>Explicit warmup_steps from config.</summary>
    public int? WarmupSteps { get; init; }

    /// <summary>
    /// Environment facts copied from the export (environment object, or gpu/host/os/device).
    /// Null when the trace did not record them. The importer process is not a substitute.
    /// </summary>
    public string? EnvironmentFacts { get; init; }
}

#endregion

#region TensorFlowRTOfflineConnector

/// <summary>
/// Offline connector for TensorFlow Runtime (TFRT) profiler data.
/// Converts TFRT exports to RuntimeRunTrace for before/after comparison.
/// </summary>
public sealed partial class TensorFlowRTOfflineConnector : IRunConnector
{
    /// <summary>Connector identifier.</summary>
    public const string Id = "tensorflowrt-offline";
    
    /// <summary>Connector version.</summary>
    public const string Version = "1.0.0";
    
    #region IRunConnector Implementation
    
    /// <inheritdoc />
    public string ConnectorId => Id;
    
    /// <inheritdoc />
    public string DisplayName => "TensorFlow-TRT Offline";
    
    /// <inheritdoc />
    public string Description => "Import TensorFlow-TensorRT profiler traces, CSVs, and logs for before/after comparison.";
    
    /// <inheritdoc />
    public ConnectorSourceType SourceType => ConnectorSourceType.LogDirectory;
    
    /// <inheritdoc />
    public IReadOnlyList<string> FileExtensions => [".json", ".json.gz", ".csv", ".log"];
    
    /// <inheritdoc />
    public ConnectorCapabilities Capabilities => 
        ConnectorCapabilities.Scalars | 
        ConnectorCapabilities.Milestones | 
        ConnectorCapabilities.WallClock |
        ConnectorCapabilities.Fingerprints;
    
    /// <inheritdoc />
    public async Task<ConnectorProbeResult> ProbeAsync(string source, CancellationToken ct = default)
    {
        var detectedSources = await DetectSourcesAsync(source, ct);
        
        if (detectedSources.Count == 0)
        {
            return ConnectorProbeResult.CannotHandle("No TFRT exports found");
        }
        
        var best = detectedSources.OrderByDescending(s => s.Priority).First();
        
        return ConnectorProbeResult.Success(
            confidence: best.Type == TfrtSourceType.ProfilerTrace ? 0.9 : 0.7,
            type: ConnectorSourceType.LogDirectory,
            capabilities: Capabilities,
            description: $"Found {best.Type} at {best.Path}"
        );
    }
    
    /// <summary>
    /// Import the runtime trace Compare reviews. The bytes stay the latency, throughput, and memory series.
    /// </summary>
    public async Task<RuntimeRunTrace> ImportRuntimeAsync(string source, CancellationToken ct = default)
    {
        if (File.Exists(source) && LooksLikeStoredRunTrace(source))
            return await ReadStoredRunTraceAsync(source, ct);

        var detectedSources = await DetectSourcesAsync(source, ct);
        if (detectedSources.Count == 0)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_NO_SUPPORTED_EXPORT}] No supported TFRT export found at: {source}");
        }

        var best = detectedSources.OrderByDescending(s => s.Priority).First();
        var rawData = await ParseSourceAsync(best, ct);
        return await BuildRuntimeTraceAsync(rawData, source, ct);
    }

    /// <inheritdoc />
    public async Task<RunTrace> ImportAsync(string source, ConnectorOptions options, CancellationToken ct = default)
    {
        var runtime = await ImportRuntimeAsync(source, ct);
        return ConvertToRunTrace(runtime, options);
    }
    
    /// <inheritdoc />
    public IAsyncEnumerable<RunTraceUpdate> StreamAsync(string source, ConnectorOptions options, CancellationToken ct = default)
    {
        // TFRT offline connector does not support streaming
        throw new NotSupportedException("TFRT offline connector does not support streaming. Use ImportAsync.");
    }
    
    #endregion
    
    #region Source Detection
    
    /// <summary>
    /// Detect available TFRT sources at the given path.
    /// Scans recursively for the folder structure contract.
    /// Priority: profiler/trace.json > profiler/overview.json > benchmark.csv > benchmark.json > runtime.log
    /// </summary>
    private async Task<List<TfrtSource>> DetectSourcesAsync(string source, CancellationToken ct)
    {
        var sources = new List<TfrtSource>();
        
        // Handle file vs directory
        var isDirectory = Directory.Exists(source);
        var isFile = File.Exists(source);
        
        if (!isDirectory && !isFile)
            return sources;
        
        if (isFile)
        {
            var fileSource = ClassifyFile(source, null);
            if (fileSource != null)
                sources.Add(fileSource);
            return sources;
        }
        
        // Detect folder context (saved_model, config.json)
        var context = await DetectFolderContextAsync(source, ct);
        
        // Search directory for TFRT exports
        await Task.Run(() =>
        {
            // Priority 1 (100): profiler/trace.json (highest fidelity)
            var profilerDir = Path.Combine(source, "profiler");
            if (Directory.Exists(profilerDir))
            {
                var traceFile = Path.Combine(profilerDir, "trace.json");
                var traceGz = Path.Combine(profilerDir, "trace.json.gz");
                
                if (File.Exists(traceFile))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.ProfilerTrace, Path = traceFile, Priority = 100, Context = context });
                else if (File.Exists(traceGz))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.ProfilerTrace, Path = traceGz, Priority = 100, Context = context });
                
                // Priority 2 (90): profiler/overview.json
                var overviewFile = Path.Combine(profilerDir, "overview.json");
                if (File.Exists(overviewFile))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.ProfilerOverview, Path = overviewFile, Priority = 90, Context = context });
            }
            
            // Also search for trace.json recursively (fallback)
            foreach (var file in Directory.EnumerateFiles(source, "trace.json*", SearchOption.AllDirectories).Take(5))
            {
                if (!sources.Any(s => s.Path == file))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.ProfilerTrace, Path = file, Priority = 95, Context = context });
            }
            
            // Priority 3 (50): benchmark.csv
            var benchmarkCsv = Path.Combine(source, "benchmark.csv");
            if (File.Exists(benchmarkCsv) && IsTfrtCsv(benchmarkCsv))
                sources.Add(new TfrtSource { Type = TfrtSourceType.BenchmarkCsv, Path = benchmarkCsv, Priority = 50, Context = context });
            
            // Also search for any CSV with TFRT metrics
            foreach (var file in Directory.EnumerateFiles(source, "*.csv", SearchOption.AllDirectories).Take(10))
            {
                if (!sources.Any(s => s.Path == file) && IsTfrtCsv(file))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.BenchmarkCsv, Path = file, Priority = 45, Context = context });
            }
            
            // Priority 4 (40): benchmark.json
            var benchmarkJson = Path.Combine(source, "benchmark.json");
            if (File.Exists(benchmarkJson))
                sources.Add(new TfrtSource { Type = TfrtSourceType.BenchmarkJson, Path = benchmarkJson, Priority = 40, Context = context });
            
            // Priority 5 (10): runtime.log (last resort)
            var runtimeLog = Path.Combine(source, "runtime.log");
            if (File.Exists(runtimeLog) && IsTfrtLog(runtimeLog))
                sources.Add(new TfrtSource { Type = TfrtSourceType.RuntimeLog, Path = runtimeLog, Priority = 10, Context = context });
            
            // Also search for any log with TFRT patterns
            foreach (var file in Directory.EnumerateFiles(source, "*.log", SearchOption.AllDirectories).Take(10))
            {
                if (!sources.Any(s => s.Path == file) && IsTfrtLog(file))
                    sources.Add(new TfrtSource { Type = TfrtSourceType.RuntimeLog, Path = file, Priority = 5, Context = context });
            }
        }, ct);
        
        return sources;
    }
    
    /// <summary>
    /// Detect folder context (saved_model, config.json, warmup_steps).
    /// </summary>
    private static async Task<TfrtFolderContext?> DetectFolderContextAsync(string source, CancellationToken ct)
    {
        string? savedModelPath = null;
        string? configPath = null;
        int? warmupSteps = null;
        string? environmentFacts = null;
        
        await Task.Run(() =>
        {
            // Look for saved_model directory
            var savedModelDir = Path.Combine(source, "saved_model");
            if (Directory.Exists(savedModelDir))
                savedModelPath = savedModelDir;
            
            // Look for config.json
            var configFile = Path.Combine(source, "config.json");
            if (File.Exists(configFile))
            {
                configPath = configFile;
                
                // Try to extract warmup_steps
                try
                {
                    var configText = File.ReadAllText(configFile);
                    using var doc = JsonDocument.Parse(configText);
                    
                    if (doc.RootElement.TryGetProperty("warmup_steps", out var warmupProp) ||
                        doc.RootElement.TryGetProperty("warmup_iterations", out warmupProp))
                    {
                        warmupSteps = warmupProp.GetInt32();
                    }

                    environmentFacts = ReadEnvironmentFacts(doc.RootElement);
                }
                catch { }
            }
        }, ct);
        
        if (savedModelPath == null && configPath == null && warmupSteps == null && environmentFacts == null)
            return null;
        
        return new TfrtFolderContext
        {
            SavedModelPath = savedModelPath,
            ConfigPath = configPath,
            WarmupSteps = warmupSteps,
            EnvironmentFacts = environmentFacts
        };
    }

    /// <summary>
    /// Environment facts belong to the exported trace. A missing block stays absent.
    /// </summary>
    private static string? ReadEnvironmentFacts(JsonElement root)
    {
        if (root.TryGetProperty("environment", out var env))
        {
            if (env.ValueKind == JsonValueKind.String)
            {
                var text = env.GetString();
                if (!string.IsNullOrWhiteSpace(text))
                    return text;
            }
            else if (env.ValueKind == JsonValueKind.Object)
            {
                return env.GetRawText();
            }
        }

        var parts = new List<string>();
        foreach (var key in new[] { "gpu", "host", "os", "device" })
        {
            if (root.TryGetProperty(key, out var value) && value.ValueKind == JsonValueKind.String)
            {
                var text = value.GetString();
                if (!string.IsNullOrWhiteSpace(text))
                    parts.Add(key + "=" + text);
            }
        }

        return parts.Count == 0 ? null : string.Join(";", parts);
    }
    
    /// <summary>
    /// A stored run trace carries schemaVersion and scalars.series.
    /// A profiler trace and a geometry file stay on their own paths.
    /// </summary>
    private static bool LooksLikeStoredRunTrace(string path)
    {
        if (!path.EndsWith(".json", StringComparison.OrdinalIgnoreCase))
            return false;

        try
        {
            using var stream = File.OpenRead(path);
            using var document = JsonDocument.Parse(stream);
            var root = document.RootElement;
            if (root.ValueKind != JsonValueKind.Object)
                return false;
            if (root.TryGetProperty("traceEvents", out _))
                return false;
            if (root.TryGetProperty("trajectory", out _))
                return false;
            if (!root.TryGetProperty("schemaVersion", out var version) || version.ValueKind != JsonValueKind.String)
                return false;
            return root.TryGetProperty("scalars", out var scalars)
                && scalars.ValueKind == JsonValueKind.Object
                && scalars.TryGetProperty("series", out var series)
                && series.ValueKind == JsonValueKind.Array;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Read the latency series and the milestones the file already stores.
    /// The steady-state detector is not asked to invent a second set.
    /// </summary>
    private async Task<RuntimeRunTrace> ReadStoredRunTraceAsync(string path, CancellationToken ct)
    {
        await using var stream = File.OpenRead(path);
        using var document = await JsonDocument.ParseAsync(stream, cancellationToken: ct);
        var root = document.RootElement;

        var runTypeText = JsonString(root, "runType") ?? "inference";
        if (runTypeText.Equals("training", StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_NO_SUPPORTED_EXPORT}] A training history is not an inference trace.");
        }

        var series = ReadStoredSeries(root);
        var latency = series.FirstOrDefault(item =>
            item.Name.Equals("latency_ms", StringComparison.OrdinalIgnoreCase)
            || item.Name.Equals("latency", StringComparison.OrdinalIgnoreCase));
        if (latency == null || !latency.Values.Any(value => value.HasValue))
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_NO_LATENCY_SIGNAL}] The run trace has no latency series.");
        }

        var count = latency.Values.Count;
        var milestones = ReadStoredMilestones(root);
        var metadataElement = root.TryGetProperty("metadata", out var metadataNode) ? metadataNode : default;
        var wall = ReadNumberList(root, "timeline", "wallTimeSeconds");

        return new RuntimeRunTrace
        {
            SchemaVersion = JsonString(root, "schemaVersion") ?? RuntimeRunTrace.CurrentSchemaVersion,
            RunId = JsonString(root, "runId") ?? Path.GetFileNameWithoutExtension(path),
            RunType = runTypeText.Equals("evaluation", StringComparison.OrdinalIgnoreCase)
                ? RunType.Evaluation
                : RunType.Inference,
            Framework = ParseStoredFramework(JsonString(root, "framework")),
            CreatedUtc = root.TryGetProperty("createdUtc", out var created) && created.TryGetDateTimeOffset(out var stamp)
                ? stamp
                : DateTimeOffset.UnixEpoch,
            Label = JsonString(root, "label"),
            Metadata = new RuntimeMetadata
            {
                ModelFingerprint = JsonString(metadataElement, "modelFingerprint") ?? RuntimeMetadata.AbsentFingerprint,
                DatasetFingerprint = JsonString(metadataElement, "datasetFingerprint") ?? RuntimeMetadata.AbsentFingerprint,
                CodeFingerprint = JsonString(metadataElement, "codeFingerprint") ?? RuntimeMetadata.AbsentFingerprint,
                EnvironmentFingerprint = JsonString(metadataElement, "environmentFingerprint") ?? RuntimeMetadata.AbsentFingerprint,
                Seed = metadataElement.ValueKind == JsonValueKind.Object
                    && metadataElement.TryGetProperty("seed", out var seed) && seed.TryGetInt32(out var seedValue)
                    ? seedValue
                    : null,
                Notes = JsonString(metadataElement, "notes")
            },
            Timeline = new RuntimeTimeline
            {
                Steps = ReadStepList(root, count),
                WallTimeSeconds = wall.Count == count ? wall : null
            },
            Scalars = new RuntimeScalars { Series = series },
            Milestones = milestones,
            Capabilities = ReadStoredCapabilities(root, series),
            Provenance = new RuntimeProvenance
            {
                Source = path,
                ConnectorId = ConnectorId,
                ConnectorVersion = Version,
                IngestedUtc = DateTimeOffset.UtcNow
            }
        };
    }

    private static List<RuntimeScalarSeries> ReadStoredSeries(JsonElement root)
    {
        var series = new List<RuntimeScalarSeries>();
        if (!root.TryGetProperty("scalars", out var scalars) || !scalars.TryGetProperty("series", out var list))
            return series;

        foreach (var item in list.EnumerateArray())
        {
            var name = JsonString(item, "name");
            if (string.IsNullOrWhiteSpace(name))
                continue;
            series.Add(new RuntimeScalarSeries
            {
                Name = name,
                Unit = ParseStoredUnit(JsonString(item, "unit")),
                Values = ReadNullableDoubles(item, "values"),
                Description = JsonString(item, "description"),
                SourceKey = JsonString(item, "sourceKey"),
                Aggregation = ParseStoredAggregation(JsonString(item, "aggregation"))
            });
        }

        return series;
    }

    private static RuntimeMilestones ReadStoredMilestones(JsonElement root)
    {
        var list = new List<RuntimeMilestone>();
        if (root.TryGetProperty("milestones", out var milestones)
            && milestones.TryGetProperty("list", out var items)
            && items.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in items.EnumerateArray())
            {
                if (!item.TryGetProperty("step", out var step) || !step.TryGetInt32(out var stepValue))
                    continue;
                list.Add(new RuntimeMilestone
                {
                    Type = ParseStoredMilestone(JsonString(item, "type")),
                    Step = stepValue,
                    Label = JsonString(item, "label")
                });
            }
        }

        return new RuntimeMilestones { List = list };
    }

    private static RuntimeCapabilities ReadStoredCapabilities(JsonElement root, List<RuntimeScalarSeries> series)
    {
        if (!root.TryGetProperty("capabilities", out var caps) || caps.ValueKind != JsonValueKind.Object)
            return RuntimeCapabilities.Detect(new RuntimeScalars { Series = series });

        return new RuntimeCapabilities
        {
            HasLoss = JsonBool(caps, "hasLoss"),
            HasAccuracy = JsonBool(caps, "hasAccuracy"),
            HasLatency = JsonBool(caps, "hasLatency"),
            HasThroughput = JsonBool(caps, "hasThroughput"),
            HasMemory = JsonBool(caps, "hasMemory"),
            HasCheckpoints = JsonBool(caps, "hasCheckpoints"),
            HasProfiler = JsonBool(caps, "hasProfiler"),
            HasEvaluatorVectors = JsonBool(caps, "hasEvaluatorVectors"),
            HasEigenSpectrum = JsonBool(caps, "hasEigenSpectrum")
        };
    }

    private static List<int> ReadStepList(JsonElement root, int count)
    {
        if (root.TryGetProperty("timeline", out var timeline)
            && timeline.TryGetProperty("steps", out var steps)
            && steps.ValueKind == JsonValueKind.Array)
        {
            var list = new List<int>();
            foreach (var step in steps.EnumerateArray())
            {
                if (step.TryGetInt32(out var value))
                    list.Add(value);
            }
            if (list.Count > 0)
                return list;
        }

        return Enumerable.Range(0, count).ToList();
    }

    private static List<double> ReadNumberList(JsonElement root, string objectName, string arrayName)
    {
        var list = new List<double>();
        if (root.TryGetProperty(objectName, out var owner)
            && owner.TryGetProperty(arrayName, out var array)
            && array.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in array.EnumerateArray())
            {
                if (item.ValueKind == JsonValueKind.Number)
                    list.Add(item.GetDouble());
            }
        }

        return list;
    }

    private static List<double?> ReadNullableDoubles(JsonElement owner, string name)
    {
        var list = new List<double?>();
        if (!owner.TryGetProperty(name, out var array) || array.ValueKind != JsonValueKind.Array)
            return list;
        foreach (var item in array.EnumerateArray())
            list.Add(item.ValueKind == JsonValueKind.Number ? item.GetDouble() : null);
        return list;
    }

    private static string? JsonString(JsonElement owner, string name)
    {
        if (owner.ValueKind != JsonValueKind.Object)
            return null;
        if (!owner.TryGetProperty(name, out var value) || value.ValueKind != JsonValueKind.String)
            return null;
        var text = value.GetString();
        return string.IsNullOrWhiteSpace(text) ? null : text;
    }

    private static bool JsonBool(JsonElement owner, string name)
    {
        return owner.TryGetProperty(name, out var value)
            && value.ValueKind == JsonValueKind.True;
    }

    private static FrameworkType ParseStoredFramework(string? value) => value?.ToLowerInvariant() switch
    {
        "tensorflowrt" => FrameworkType.TensorFlowRT,
        "tensorflow" => FrameworkType.TensorFlow,
        "pytorch" => FrameworkType.PyTorch,
        "jax" => FrameworkType.Jax,
        "mlflow" => FrameworkType.MLflow,
        "wandb" => FrameworkType.WandB,
        "tensorboard" => FrameworkType.TensorBoard,
        _ => FrameworkType.Unknown
    };

    private static ScalarUnit ParseStoredUnit(string? value) => value?.ToLowerInvariant() switch
    {
        "milliseconds" => ScalarUnit.Milliseconds,
        "seconds" => ScalarUnit.Seconds,
        "microseconds" => ScalarUnit.Microseconds,
        "items_per_second" => ScalarUnit.ItemsPerSecond,
        "bytes" => ScalarUnit.Bytes,
        "megabytes" => ScalarUnit.Megabytes,
        "gigabytes" => ScalarUnit.Gigabytes,
        "percent" => ScalarUnit.Percent,
        "loss" => ScalarUnit.Loss,
        "accuracy" => ScalarUnit.Accuracy,
        "count" => ScalarUnit.Count,
        _ => ScalarUnit.None
    };

    private static ScalarAggregation? ParseStoredAggregation(string? value) => value?.ToLowerInvariant() switch
    {
        "none" => ScalarAggregation.None,
        "mean" => ScalarAggregation.Mean,
        "median" => ScalarAggregation.Median,
        "p50" => ScalarAggregation.P50,
        "p90" => ScalarAggregation.P90,
        "p95" => ScalarAggregation.P95,
        "p99" => ScalarAggregation.P99,
        _ => null
    };

    private static RuntimeMilestoneType ParseStoredMilestone(string? value) => value?.ToLowerInvariant() switch
    {
        "warmup_end" => RuntimeMilestoneType.WarmupEnd,
        "steady_state_start" => RuntimeMilestoneType.SteadyStateStart,
        "steady_state_end" => RuntimeMilestoneType.SteadyStateEnd,
        "epoch_start" => RuntimeMilestoneType.EpochStart,
        "epoch_end" => RuntimeMilestoneType.EpochEnd,
        "eval" => RuntimeMilestoneType.Eval,
        "checkpoint" => RuntimeMilestoneType.Checkpoint,
        _ => RuntimeMilestoneType.Custom
    };

    private TfrtSource? ClassifyFile(string path, TfrtFolderContext? context)
    {
        var name = Path.GetFileName(path).ToLowerInvariant();
        
        if (name.StartsWith("trace.json") || LooksLikeProfilerTrace(path))
            return new TfrtSource { Type = TfrtSourceType.ProfilerTrace, Path = path, Priority = 100, Context = context };
        
        if (name == "overview.json")
            return new TfrtSource { Type = TfrtSourceType.ProfilerOverview, Path = path, Priority = 90, Context = context };
        
        if (name == "benchmark.csv" || (name.EndsWith(".csv") && IsTfrtCsv(path)))
            return new TfrtSource { Type = TfrtSourceType.BenchmarkCsv, Path = path, Priority = 50, Context = context };
        
        if (name.EndsWith(".json") && LooksLikeBenchmarkJson(path))
            return new TfrtSource { Type = TfrtSourceType.BenchmarkJson, Path = path, Priority = 40, Context = context };
        
        if (name.EndsWith(".log") && IsTfrtLog(path))
            return new TfrtSource { Type = TfrtSourceType.RuntimeLog, Path = path, Priority = 10, Context = context };
        
        return null;
    }

    /// <summary>
    /// A profiler export is a Chrome trace. The file name does not have to be trace.json.
    /// </summary>
    private static bool LooksLikeProfilerTrace(string path)
    {
        if (!path.EndsWith(".json", StringComparison.OrdinalIgnoreCase))
            return false;

        try
        {
            using var stream = File.OpenRead(path);
            using var document = JsonDocument.Parse(stream);
            return document.RootElement.TryGetProperty("traceEvents", out var events)
                && events.ValueKind == JsonValueKind.Array;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// A benchmark export carries a results, iterations, or benchmarks array.
    /// </summary>
    private static bool LooksLikeBenchmarkJson(string path)
    {
        try
        {
            using var stream = File.OpenRead(path);
            using var document = JsonDocument.Parse(stream);
            var root = document.RootElement;
            return ArrayProperty(root, "results") || ArrayProperty(root, "iterations") || ArrayProperty(root, "benchmarks");
        }
        catch
        {
            return false;
        }
    }

    private static bool ArrayProperty(JsonElement root, string name)
    {
        return root.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.Array;
    }
    
    private static bool IsTfrtCsv(string path)
    {
        try
        {
            using var reader = new StreamReader(path);
            var header = reader.ReadLine();
            if (header == null) return false;
            
            // Required: latency_ms OR equivalent
            // Optional: throughput, memory_mb, iteration/step
            var headerLower = header.ToLowerInvariant();
            return headerLower.Contains("latency") ||
                   headerLower.Contains("throughput") ||
                   headerLower.Contains("memory");
        }
        catch
        {
            return false;
        }
    }
    
    private static bool IsTfrtLog(string path)
    {
        try
        {
            using var reader = new StreamReader(path);
            for (int i = 0; i < 20; i++)
            {
                var line = reader.ReadLine();
                if (line == null) break;
                
                // Look for TFRT-specific patterns
                if (line.Contains("TensorRT", StringComparison.OrdinalIgnoreCase) ||
                    line.Contains("latency_ms", StringComparison.OrdinalIgnoreCase) ||
                    line.Contains("TF-TRT", StringComparison.OrdinalIgnoreCase) ||
                    line.Contains("batch size", StringComparison.OrdinalIgnoreCase))
                    return true;
            }
        }
        catch { }
        
        return false;
    }
    
    #endregion
    
    #region Source Parsing
    
    private async Task<TfrtRawData> ParseSourceAsync(TfrtSource source, CancellationToken ct)
    {
        var raw = source.Type switch
        {
            TfrtSourceType.ProfilerTrace => await ParseProfilerTraceAsync(source.Path, ct),
            TfrtSourceType.ProfilerOverview => await ParseProfilerOverviewAsync(source.Path, ct),
            TfrtSourceType.BenchmarkCsv => await ParseCsvExportAsync(source.Path, ct),
            TfrtSourceType.BenchmarkJson => await ParseBenchmarkJsonAsync(source.Path, ct),
            TfrtSourceType.RuntimeLog => await ParseLogFileAsync(source.Path, ct),
            _ => throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_NO_SUPPORTED_EXPORT}] Unknown source type: {source.Type}")
        };
        
        // Set context from folder structure
        raw.WarmupStepsFromConfig = source.Context?.WarmupSteps;
        raw.SavedModelPath = source.Context?.SavedModelPath;
        raw.EnvironmentFacts = source.Context?.EnvironmentFacts;
        
        // Validate: must have latency signal. Missing cells stay null on their step.
        if (!raw.Samples.Any(sample => sample.LatencyMs.HasValue))
        {
            // For logs, this is a warning → expect lower quality
            if (source.Type == TfrtSourceType.RuntimeLog)
            {
                // Log-only: expect warnings in validation
            }
            else
            {
                throw new InvalidOperationException(
                    $"[{TfrtErrorCodes.TFRT_NO_LATENCY_SIGNAL}] Cannot extract latency signal from {source.Type}");
            }
        }
        
        return raw;
    }
    
    /// <summary>
    /// Parse TensorBoard profiler trace.
    /// </summary>
    private async Task<TfrtRawData> ParseProfilerTraceAsync(string path, CancellationToken ct)
    {
        var raw = new TfrtRawData { SourcePath = path, SourceType = TfrtSourceType.ProfilerTrace };
        
        try
        {
            // Read potentially gzipped trace
            Stream stream;
            if (path.EndsWith(".gz", StringComparison.OrdinalIgnoreCase))
            {
                var compressed = await File.ReadAllBytesAsync(path, ct);
                stream = new System.IO.Compression.GZipStream(
                    new MemoryStream(compressed), 
                    System.IO.Compression.CompressionMode.Decompress);
            }
            else
            {
                stream = File.OpenRead(path);
            }
            
            await using (stream)
            {
                var json = await JsonDocument.ParseAsync(stream, cancellationToken: ct);
                ParseTraceEvents(json.RootElement, raw);
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_PROFILE_PARSE_FAILED}] Failed to parse profiler trace: {ex.Message}", ex);
        }
        
        return raw;
    }
    
    private static void ParseTraceEvents(JsonElement root, TfrtRawData raw)
    {
        // Chrome trace format: { "traceEvents": [...] }
        if (!root.TryGetProperty("traceEvents", out var events))
            return;
        
        int step = 0;
        foreach (var evt in events.EnumerateArray())
        {
            if (!evt.TryGetProperty("name", out var nameProp))
                continue;
            
            var name = nameProp.GetString() ?? "";
            
            // Extract timing events. One event is one sample; a missing field stays null.
            if (name.Contains("TensorRT", StringComparison.OrdinalIgnoreCase) ||
                name.Contains("inference", StringComparison.OrdinalIgnoreCase))
            {
                double? latencyMs = null;
                double? wallSeconds = null;
                if (evt.TryGetProperty("dur", out var dur))
                    latencyMs = dur.GetDouble() / 1000.0;
                
                if (evt.TryGetProperty("ts", out var ts))
                    wallSeconds = ts.GetDouble() / 1_000_000.0;

                if (latencyMs.HasValue || wallSeconds.HasValue)
                    raw.AddSample(step++, latencyMs: latencyMs, wallTimeSeconds: wallSeconds);
            }
            
            // Memory-only events fill the open sample, or start a new one.
            if (name.Contains("memory", StringComparison.OrdinalIgnoreCase) &&
                evt.TryGetProperty("args", out var args))
            {
                if (args.TryGetProperty("bytes", out var bytes))
                    raw.AttachMemory(bytes.GetInt64());
            }
        }
    }
    
    /// <summary>
    /// Parse profiler overview.json file.
    /// </summary>
    private async Task<TfrtRawData> ParseProfilerOverviewAsync(string path, CancellationToken ct)
    {
        var raw = new TfrtRawData { SourcePath = path, SourceType = TfrtSourceType.ProfilerOverview };
        
        try
        {
            var json = await File.ReadAllTextAsync(path, ct);
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;
            
            // Overview typically has summary stats, not per-iteration.
            // Summary fields share one sample so a lone throughput cannot shift later rows.
            if (root.TryGetProperty("inference_stats", out var stats) ||
                root.TryGetProperty("run_stats", out stats))
            {
                double? latencyMs = stats.TryGetProperty("avg_latency_ms", out var avgLat) ? avgLat.GetDouble() : null;
                double? throughput = stats.TryGetProperty("throughput", out var thr) ? thr.GetDouble() : null;
                long? memoryBytes = stats.TryGetProperty("peak_memory_bytes", out var mem) ? mem.GetInt64() : null;
                if (latencyMs.HasValue || throughput.HasValue || memoryBytes.HasValue)
                    raw.AddSample(raw.Samples.Count, latencyMs: latencyMs, throughput: throughput, memoryBytes: memoryBytes);
            }
            
            // Try to get per-iteration data if available
            if (root.TryGetProperty("iterations", out var iterations))
            {
                foreach (var iter in iterations.EnumerateArray())
                {
                    if (iter.TryGetProperty("latency_ms", out var lat))
                        raw.AddSample(raw.Samples.Count, latencyMs: lat.GetDouble());
                }
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_PARSE_FAILED}] Failed to parse overview: {ex.Message}", ex);
        }
        
        return raw;
    }
    
    /// <summary>
    /// Parse benchmark.json file.
    /// </summary>
    private async Task<TfrtRawData> ParseBenchmarkJsonAsync(string path, CancellationToken ct)
    {
        var raw = new TfrtRawData { SourcePath = path, SourceType = TfrtSourceType.BenchmarkJson };
        
        try
        {
            var json = await File.ReadAllTextAsync(path, ct);
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;
            
            // Look for iterations/results array
            JsonElement results;
            if (root.TryGetProperty("results", out results) ||
                root.TryGetProperty("iterations", out results) ||
                root.TryGetProperty("benchmarks", out results))
            {
                int step = 0;
                foreach (var item in results.EnumerateArray())
                {
                    double? latencyMs = null;
                    if (item.TryGetProperty("latency_ms", out var lat) ||
                        item.TryGetProperty("latency", out lat))
                    {
                        latencyMs = lat.GetDouble();
                    }
                    
                    double? throughput = null;
                    if (item.TryGetProperty("throughput", out var thr) ||
                        item.TryGetProperty("items_per_sec", out thr))
                    {
                        throughput = thr.GetDouble();
                    }
                    
                    long? memoryBytes = null;
                    if (item.TryGetProperty("memory_mb", out var memMb))
                        memoryBytes = (long)(memMb.GetDouble() * 1_000_000);
                    else if (item.TryGetProperty("memory_bytes", out var mem))
                        memoryBytes = mem.GetInt64();
                    
                    var sampleStep = step;
                    if (item.TryGetProperty("step", out var stepVal) ||
                        item.TryGetProperty("iteration", out stepVal))
                    {
                        sampleStep = stepVal.GetInt32();
                    }

                    raw.AddSample(sampleStep, latencyMs: latencyMs, throughput: throughput, memoryBytes: memoryBytes);
                    step++;
                }
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_PARSE_FAILED}] Failed to parse benchmark JSON: {ex.Message}", ex);
        }
        
        return raw;
    }
    
    /// <summary>
    /// Parse CSV export.
    /// </summary>
    private async Task<TfrtRawData> ParseCsvExportAsync(string path, CancellationToken ct)
    {
        var raw = new TfrtRawData { SourcePath = path, SourceType = TfrtSourceType.BenchmarkCsv };
        
        try
        {
            var lines = await File.ReadAllLinesAsync(path, ct);
            if (lines.Length < 2)
            {
                throw new InvalidOperationException(
                    $"[{TfrtErrorCodes.TFRT_CSV_FORMAT_ERROR}] CSV file has no data rows");
            }
            
            // Parse header
            var header = lines[0].Split(',').Select(h => h.Trim().ToLowerInvariant()).ToList();
            var stepIdx = FindColumnIndex(header, "step", "iteration", "batch");
            var latencyIdx = FindColumnIndex(header, "latency_ms", "latency", "time_ms");
            var throughputIdx = FindColumnIndex(header, "throughput", "items_per_sec", "samples_per_sec");
            var memoryIdx = FindColumnIndex(header, "memory_bytes", "memory_mb", "memory");
            var wallTimeIdx = FindColumnIndex(header, "wall_time", "timestamp", "time_sec");
            
            // Parse data rows. Each row is one sample. A blank metric stays null at that step.
            int autoStep = 0;
            for (int i = 1; i < lines.Length; i++)
            {
                if (string.IsNullOrWhiteSpace(lines[i]))
                    continue;
                
                var values = lines[i].Split(',');
                
                int sampleStep;
                if (stepIdx >= 0 && stepIdx < values.Length &&
                    int.TryParse(values[stepIdx], out var step))
                {
                    sampleStep = step;
                }
                else
                {
                    sampleStep = autoStep;
                }
                autoStep++;
                
                double? latencyMs = null;
                if (latencyIdx >= 0 && latencyIdx < values.Length &&
                    double.TryParse(values[latencyIdx], NumberStyles.Float, CultureInfo.InvariantCulture, out var lat))
                {
                    latencyMs = lat;
                }
                
                double? throughput = null;
                if (throughputIdx >= 0 && throughputIdx < values.Length &&
                    double.TryParse(values[throughputIdx], NumberStyles.Float, CultureInfo.InvariantCulture, out var thr))
                {
                    throughput = thr;
                }
                
                long? memoryBytes = null;
                if (memoryIdx >= 0 && memoryIdx < values.Length &&
                    long.TryParse(values[memoryIdx], out var mem))
                {
                    if (header[memoryIdx] == "memory_mb")
                        mem *= 1_000_000;
                    memoryBytes = mem;
                }
                
                double? wallSeconds = null;
                if (wallTimeIdx >= 0 && wallTimeIdx < values.Length &&
                    double.TryParse(values[wallTimeIdx], NumberStyles.Float, CultureInfo.InvariantCulture, out var wt))
                {
                    wallSeconds = wt;
                }

                raw.AddSample(sampleStep, latencyMs: latencyMs, throughput: throughput, memoryBytes: memoryBytes, wallTimeSeconds: wallSeconds);
            }
        }
        catch (InvalidOperationException)
        {
            throw;
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_CSV_FORMAT_ERROR}] Failed to parse CSV: {ex.Message}", ex);
        }
        
        return raw;
    }
    
    private static int FindColumnIndex(List<string> header, params string[] candidates)
    {
        foreach (var candidate in candidates)
        {
            var idx = header.FindIndex(h => string.Equals(h, candidate, StringComparison.OrdinalIgnoreCase));
            if (idx >= 0)
                return idx;
        }
        return -1;
    }
    
    /// <summary>
    /// Parse log file (last resort - expect warnings).
    /// </summary>
    private async Task<TfrtRawData> ParseLogFileAsync(string path, CancellationToken ct)
    {
        var raw = new TfrtRawData { SourcePath = path, SourceType = TfrtSourceType.RuntimeLog };
        
        // Regex patterns for common TFRT log formats
        var latencyRegex = LatencyPattern();
        var throughputRegex = ThroughputPattern();
        var memoryRegex = MemoryPattern();
        var stepRegex = StepPattern();
        
        var lines = await File.ReadAllLinesAsync(path, ct);
        int autoStep = 0;
        
        foreach (var line in lines)
        {
            var stepMatch = stepRegex.Match(line);
            int? step = stepMatch.Success && int.TryParse(stepMatch.Groups[1].Value, out var parsedStep)
                ? parsedStep
                : null;
            
            double? latencyMs = null;
            var latMatch = latencyRegex.Match(line);
            if (latMatch.Success && double.TryParse(latMatch.Groups[1].Value, NumberStyles.Float, 
                CultureInfo.InvariantCulture, out var lat))
            {
                latencyMs = lat;
            }
            
            double? throughput = null;
            var thrMatch = throughputRegex.Match(line);
            if (thrMatch.Success && double.TryParse(thrMatch.Groups[1].Value, NumberStyles.Float,
                CultureInfo.InvariantCulture, out var thr))
            {
                throughput = thr;
            }
            
            long? memoryBytes = null;
            var memMatch = memoryRegex.Match(line);
            if (memMatch.Success && long.TryParse(memMatch.Groups[1].Value, out var mem))
                memoryBytes = mem;

            if (step is null && latencyMs is null && throughput is null && memoryBytes is null)
                continue;

            var sampleStep = step ?? autoStep;
            autoStep = step is null ? autoStep + 1 : step.Value + 1;
            raw.AddSample(sampleStep, latencyMs: latencyMs, throughput: throughput, memoryBytes: memoryBytes);
        }
        
        return raw;
    }
    
    [GeneratedRegex(@"latency[_\s]*[:=]?\s*([\d.]+)\s*(?:ms)?", RegexOptions.IgnoreCase)]
    private static partial Regex LatencyPattern();
    
    [GeneratedRegex(@"throughput[_\s]*[:=]?\s*([\d.]+)", RegexOptions.IgnoreCase)]
    private static partial Regex ThroughputPattern();
    
    [GeneratedRegex(@"memory[_\s]*[:=]?\s*(\d+)", RegexOptions.IgnoreCase)]
    private static partial Regex MemoryPattern();
    
    [GeneratedRegex(@"step[_\s]*[:=]?\s*(\d+)", RegexOptions.IgnoreCase)]
    private static partial Regex StepPattern();
    
    #endregion
    
    #region Runtime Trace Building
    
    private async Task<RuntimeRunTrace> BuildRuntimeTraceAsync(TfrtRawData raw, string source, CancellationToken ct)
    {
        // Validate timeline
        if (raw.Samples.Count == 0)
        {
            throw new InvalidOperationException(
                $"[{TfrtErrorCodes.TFRT_TIMELINE_INCONSISTENT}] No timeline data extracted from source");
        }
        
        // Ensure steps are monotonic
        for (int i = 1; i < raw.Samples.Count; i++)
        {
            if (raw.Samples[i].Step < raw.Samples[i - 1].Step)
            {
                throw new InvalidOperationException(
                    $"[{TfrtErrorCodes.TFRT_TIMELINE_INCONSISTENT}] Non-monotonic step at index {i}: {raw.Samples[i].Step} < {raw.Samples[i - 1].Step}");
            }
        }
        
        // Build scalar series. Each list is one value per sample, null where that row omitted the metric.
        var series = new List<RuntimeScalarSeries>();
        
        if (raw.Samples.Any(sample => sample.LatencyMs.HasValue))
        {
            series.Add(new RuntimeScalarSeries
            {
                Name = "latency_ms",
                Unit = ScalarUnit.Milliseconds,
                Values = raw.Samples.Select(sample => sample.LatencyMs).ToList(),
                Description = "Inference latency per step",
                SourceKey = "latency_ms"
            });
        }
        
        if (raw.Samples.Any(sample => sample.ThroughputItemsPerSec.HasValue))
        {
            series.Add(new RuntimeScalarSeries
            {
                Name = "throughput_items_per_sec",
                Unit = ScalarUnit.ItemsPerSecond,
                Values = raw.Samples.Select(sample => sample.ThroughputItemsPerSec).ToList(),
                Description = "Inference throughput",
                SourceKey = "throughput"
            });
        }
        
        if (raw.Samples.Any(sample => sample.MemoryBytes.HasValue))
        {
            series.Add(new RuntimeScalarSeries
            {
                Name = "memory_bytes",
                Unit = ScalarUnit.Bytes,
                Values = raw.Samples.Select(sample => sample.MemoryBytes.HasValue ? (double?)sample.MemoryBytes.Value : null).ToList(),
                Description = "Memory usage",
                SourceKey = "memory"
            });
        }
        
        var scalars = new RuntimeScalars { Series = series };
        
        // Detect milestones
        var milestones = DetectMilestones(raw, scalars);
        
        // Generate fingerprints
        var metadata = await GenerateMetadataAsync(raw, source, ct);
        
        // Build capabilities
        var capabilities = RuntimeCapabilities.Detect(scalars, raw.SourceType == TfrtSourceType.ProfilerTrace);
        
        return new RuntimeRunTrace
        {
            SchemaVersion = RuntimeRunTrace.CurrentSchemaVersion,
            RunId = Guid.NewGuid().ToString("N"),
            RunType = RunType.Inference,
            Framework = FrameworkType.TensorFlowRT,
            CreatedUtc = DateTimeOffset.UtcNow,
            Label = Path.GetFileNameWithoutExtension(source),
            Metadata = metadata,
            Timeline = new RuntimeTimeline
            {
                Steps = raw.Samples.Select(sample => sample.Step).ToList(),
                WallTimeSeconds = raw.Samples.All(sample => sample.WallTimeSeconds.HasValue)
                    ? raw.Samples.Select(sample => sample.WallTimeSeconds!.Value).ToList()
                    : null,
                Epoch = null // Not applicable to inference
            },
            Scalars = scalars,
            Milestones = milestones,
            Capabilities = capabilities,
            Provenance = new RuntimeProvenance
            {
                Source = source,
                ConnectorId = ConnectorId,
                ConnectorVersion = Version,
                IngestedUtc = DateTimeOffset.UtcNow
            }
        };
    }
    
    private static RuntimeMilestones DetectMilestones(TfrtRawData raw, RuntimeScalars scalars)
    {
        var milestones = new List<RuntimeMilestone>();
        
        int? warmupEnd = null;
        
        // Priority 1: explicit warmup_steps from config.json
        if (raw.WarmupStepsFromConfig.HasValue)
        {
            warmupEnd = raw.WarmupStepsFromConfig.Value;
            milestones.Add(new RuntimeMilestone
            {
                Type = RuntimeMilestoneType.WarmupEnd,
                Step = warmupEnd.Value,
                Label = "Warmup Complete (from config)"
            });
        }
        else
        {
            // Priority 2: auto-detect from latency stabilization heuristic
            var latencySeries = scalars.GetByName("latency_ms");
            if (latencySeries != null)
            {
                warmupEnd = SteadyStateDetector.DetectWarmupEnd(latencySeries.Values);
                if (warmupEnd.HasValue)
                {
                    milestones.Add(new RuntimeMilestone
                    {
                        Type = RuntimeMilestoneType.WarmupEnd,
                        Step = warmupEnd.Value,
                        Label = "Warmup Complete"
                    });
                }
            }
        }
        
        // Detect steady state if we have warmup end
        if (warmupEnd.HasValue)
        {
            var latencySeries = scalars.GetByName("latency_ms");
            if (latencySeries != null)
            {
                var steadyState = SteadyStateDetector.DetectSteadyState(
                    latencySeries.Values, warmupEnd.Value);
                
                if (steadyState.HasValue)
                {
                    milestones.Add(new RuntimeMilestone
                    {
                        Type = RuntimeMilestoneType.SteadyStateStart,
                        Step = steadyState.Value.Start,
                        Label = "Steady State Begin"
                    });
                    
                    milestones.Add(new RuntimeMilestone
                    {
                        Type = RuntimeMilestoneType.SteadyStateEnd,
                        Step = steadyState.Value.End,
                        Label = "Steady State End"
                    });
                }
            }
        }
        
        return new RuntimeMilestones { List = milestones };
    }
    
    private async Task<RuntimeMetadata> GenerateMetadataAsync(TfrtRawData raw, string source, CancellationToken ct)
    {
        // Try to find SavedModel for model fingerprint
        var modelFingerprint = await ComputeModelFingerprintAsync(source, ct);
        
        // Environment comes from the export. The importer process is not hashed.
        var envFingerprint = string.IsNullOrWhiteSpace(raw.EnvironmentFacts)
            ? RuntimeMetadata.AbsentFingerprint
            : RuntimeMetadata.CreateFingerprint(raw.EnvironmentFacts);
        
        // Code and dataset are not in a TFRT export. Absent is not a shared identity.
        var codeFingerprint = RuntimeMetadata.AbsentFingerprint;
        var datasetFingerprint = RuntimeMetadata.AbsentFingerprint;
        
        return new RuntimeMetadata
        {
            ModelFingerprint = modelFingerprint,
            DatasetFingerprint = datasetFingerprint,
            CodeFingerprint = codeFingerprint,
            EnvironmentFingerprint = envFingerprint,
            Tags = ["tensorflowrt", "inference"],
            FrameworkDetails = new Dictionary<string, object>
            {
                ["sourceType"] = raw.SourceType.ToString(),
                ["sourcePath"] = raw.SourcePath
            }
        };
    }
    
    private async Task<string> ComputeModelFingerprintAsync(string source, CancellationToken ct)
    {
        try
        {
            // Look for SavedModel in source directory
            var searchDir = File.Exists(source) ? Path.GetDirectoryName(source) : source;
            if (searchDir == null)
                return RuntimeMetadata.UnknownFingerprint("model");
            
            // Check for saved_model.pb
            var savedModelPath = Path.Combine(searchDir, "saved_model.pb");
            if (!File.Exists(savedModelPath))
            {
                // Search up to 2 levels up
                var parent = Path.GetDirectoryName(searchDir);
                if (parent != null)
                {
                    savedModelPath = Path.Combine(parent, "saved_model.pb");
                }
            }
            
            if (File.Exists(savedModelPath))
            {
                var bytes = await File.ReadAllBytesAsync(savedModelPath, ct);
                var hash = SHA256.HashData(bytes);
                return Convert.ToHexString(hash).ToLowerInvariant();
            }
            
            return RuntimeMetadata.UnknownFingerprint("model");
        }
        catch
        {
            return RuntimeMetadata.AbsentFingerprint;
        }
    }
    
    #endregion
    
    #region RunTrace Conversion
    
    /// <summary>
    /// Convert RuntimeRunTrace to standard RunTrace.
    /// </summary>
    private static RunTrace ConvertToRunTrace(RuntimeRunTrace runtime, ConnectorOptions? options)
    {
        // Map runtime scalars to RunTrace scalars
        var scalars = new Dictionary<string, ScalarSeries>();
        var steps = runtime.Timeline.Steps.ToList();
        
        foreach (var series in runtime.Scalars.Series)
        {
            var mapped = MapScalarName(series.Name);
            var values = series.Values.Select(v => v ?? double.NaN).ToList();
            scalars[mapped] = new ScalarSeries
            {
                Steps = steps,
                Values = values,
                WallClockSeconds = runtime.Timeline.WallTimeSeconds?.ToList()
            };
        }
        
        // Map milestones
        var milestones = runtime.Milestones.List.Select(m => new Milestone
        {
            Step = m.Step,
            Type = MapMilestoneType(m.Type),
            Label = m.Label
        }).ToList();
        
        return new RunTrace
        {
            TraceVersion = RunTrace.CurrentVersion,
            Capabilities = ConnectorCapabilities.Scalars | ConnectorCapabilities.Milestones | ConnectorCapabilities.WallClock,
            Metadata = new RunTraceMetadata
            {
                RunId = runtime.RunId,
                Label = runtime.Label ?? "TensorFlowRT Run",
                Source = ConnectorSourceType.LogDirectory,
                SourcePath = runtime.Provenance?.Source,
                CreatedUtc = runtime.CreatedUtc,
                Tags = runtime.Metadata.Tags?.ToDictionary(t => t, _ => "true"),
                Fingerprints = new RunFingerprints
                {
                    Model = runtime.Metadata.ModelFingerprint,
                    Dataset = runtime.Metadata.DatasetFingerprint,
                    Code = runtime.Metadata.CodeFingerprint,
                    Environment = runtime.Metadata.EnvironmentFingerprint
                }
            },
            Timeline = new RunTraceTimeline
            {
                StepCount = runtime.Timeline.StepCount,
                StepUnit = StepUnit.Iteration,
                WallClockPerStep = runtime.Timeline.WallTimeSeconds?.ToList()
            },
            Scalars = scalars,
            Milestones = milestones
        };
    }
    
    private static string MapScalarName(string name)
    {
        return name switch
        {
            "latency_ms" => "latency",
            "throughput_items_per_sec" => "throughput",
            "memory_bytes" => "memory",
            _ => name
        };
    }
    
    private static MilestoneType MapMilestoneType(RuntimeMilestoneType type)
    {
        return type switch
        {
            RuntimeMilestoneType.EpochStart or RuntimeMilestoneType.EpochEnd => MilestoneType.Epoch,
            RuntimeMilestoneType.Eval => MilestoneType.Eval,
            RuntimeMilestoneType.Checkpoint => MilestoneType.Checkpoint,
            RuntimeMilestoneType.WarmupEnd or RuntimeMilestoneType.SteadyStateStart or 
            RuntimeMilestoneType.SteadyStateEnd => MilestoneType.Custom,
            _ => MilestoneType.Custom
        };
    }
    
    #endregion
}

#endregion

#region Internal Types

/// <summary>
/// Raw data extracted from TFRT sources before transformation.
/// </summary>
internal sealed class TfrtRawData
{
    public required string SourcePath { get; init; }
    public required TfrtSourceType SourceType { get; init; }
    
    /// <summary>One row per step. Missing metrics stay null on that row.</summary>
    public List<TfrtSample> Samples { get; } = [];
    
    // Folder context
    public int? WarmupStepsFromConfig { get; set; }
    public string? SavedModelPath { get; set; }
    public string? EnvironmentFacts { get; set; }

    public void AddSample(
        int step,
        double? latencyMs = null,
        double? throughput = null,
        long? memoryBytes = null,
        double? wallTimeSeconds = null)
    {
        Samples.Add(new TfrtSample
        {
            Step = step,
            LatencyMs = latencyMs,
            ThroughputItemsPerSec = throughput,
            MemoryBytes = memoryBytes,
            WallTimeSeconds = wallTimeSeconds
        });
    }

    /// <summary>
    /// A memory-only event fills the latest sample when that sample has no memory yet.
    /// Otherwise it is its own sample and does not shift earlier metrics.
    /// </summary>
    public void AttachMemory(long bytes)
    {
        if (Samples.Count > 0 && Samples[^1].MemoryBytes is null)
        {
            Samples[^1].MemoryBytes = bytes;
            return;
        }

        AddSample(Samples.Count, memoryBytes: bytes);
    }
}

/// <summary>One TFRT observation keyed by step.</summary>
internal sealed class TfrtSample
{
    public int Step { get; init; }
    public double? LatencyMs { get; init; }
    public double? ThroughputItemsPerSec { get; init; }
    public long? MemoryBytes { get; set; }
    public double? WallTimeSeconds { get; init; }
}

#endregion

#region Registration Extension

/// <summary>
/// Extension to register TFRT connector.
/// </summary>
public static class TensorFlowRTConnectorExtensions
{
    /// <summary>
    /// Register the TensorFlowRT offline connector.
    /// </summary>
    public static ConnectorRegistry RegisterTensorFlowRT(this ConnectorRegistry registry)
    {
        registry.Register(new TensorFlowRTOfflineConnector());
        return registry;
    }
}

#endregion
