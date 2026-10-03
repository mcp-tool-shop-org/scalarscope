using System.IO.Compression;
using System.Text;
using System.Text.Json;
using FluentAssertions;
using ScalarScope.Services;
using Xunit;
using V1 = ScalarScope.Services.Bundles;

namespace ScalarScope.FixtureTests;

public class BundleHashTests
{
    private static readonly Encoding Utf8NoBom = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false);

    private static readonly JsonSerializerOptions CamelCase = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true
    };

    [Fact]
    public async Task ExportedBundle_RecomputesTheHashItStates()
    {
        var directory = NewDirectory();
        var path = Path.Combine(directory, "review.scbundle");
        var exported = await ComparisonBundleService.Instance.ExportAsync(SampleBundle(), path);

        exported.Success.Should().BeTrue(exported.ErrorMessage);
        var manifestBytes = ReadZipEntry(path, "manifest.json");
        manifestBytes[0].Should().Be((byte)'{');

        var verification = await BundleIntegrityService.VerifyBundleAsync(path);
        verification.IsValid.Should().BeTrue(string.Join("; ", verification.Issues.Select(issue => issue.Message)));
        verification.ActualBundleHash.Should().Be(exported.BundleHash);
        verification.BundleHashMatch.Should().BeTrue();
    }

    [Fact]
    public async Task StatedHash_IsRejectedWhenItIsNotTheHashOfTheBytes()
    {
        var path = await ExportSample();
        var honest = await BundleIntegrityService.VerifyBundleAsync(path);

        ReplaceZipText(path, "integrity.json", json =>
            json.Replace(honest.ActualBundleHash!, new string('0', 64), StringComparison.Ordinal));

        var verification = await BundleIntegrityService.VerifyBundleAsync(path);
        verification.IsValid.Should().BeFalse();
        verification.BundleHashMatch.Should().BeFalse();
        verification.ActualBundleHash.Should().Be(honest.ActualBundleHash);
        verification.Issues.Should().Contain(issue => issue.Code == "BundleHashMismatch");
    }

    [Fact]
    public async Task FileLeftOutOfTheHashList_IsRejected()
    {
        var path = await ExportSample();
        ReplaceZipText(path, "integrity.json", json =>
        {
            var info = JsonSerializer.Deserialize<BundleIntegrityInfo>(json, CamelCase)!;
            var reduced = new Dictionary<string, string>(info.FileHashes);
            reduced.Remove("findings/deltas.json").Should().BeTrue();
            var lied = new BundleIntegrityInfo
            {
                FileHashes = reduced,
                BundleHash = BundleIntegrityService.ComputeBundleHash(reduced),
                ComputedAt = info.ComputedAt
            };
            return JsonSerializer.Serialize(lied, CamelCase);
        });

        var verification = await BundleIntegrityService.VerifyBundleAsync(path);
        verification.IsValid.Should().BeFalse();
        verification.Issues.Should().Contain(issue => issue.Code == "UndeclaredFile");
        verification.Issues.Should().Contain(issue => issue.Code == "BundleHashMismatch");
    }

    [Fact]
    public async Task ChangedDeltaBytes_AreRejected()
    {
        var path = await ExportSample();
        ReplaceZipText(path, "findings/deltas.json", _ => "[{\"id\":\"tampered\"}]");

        var verification = await BundleIntegrityService.VerifyBundleAsync(path);
        verification.IsValid.Should().BeFalse();
        verification.ModifiedFiles.Should().Contain("findings/deltas.json");
        verification.BundleHashMatch.Should().BeFalse();
    }

    [Fact]
    public async Task V1Bundle_RecomputesBundleHashFromTheManifestCore()
    {
        var directory = NewDirectory();
        var path = Path.Combine(directory, "v1.scbundle");
        var exported = await V1.BundleExporter.Instance.ExportAsync(SampleV1Payload(), path);

        exported.Success.Should().BeTrue(exported.ErrorMessage);
        var valid = await V1.BundleImporter.Instance.ValidateAsync(path);
        valid.IsValid.Should().BeTrue(string.Join("; ", valid.Errors));

        var manifestJson = Utf8NoBom.GetString(ReadZipEntry(path, "manifest.json"));
        var lied = manifestJson.Replace(exported.BundleHash!, new string('a', 64), StringComparison.Ordinal);
        ReplaceZipText(path, "manifest.json", _ => lied);

        var rejected = await V1.BundleImporter.Instance.ValidateAsync(path);
        rejected.IsValid.Should().BeFalse();
        rejected.Errors.Should().Contain(error => error.Contains("recomputed hash", StringComparison.Ordinal));
    }

    private static async Task<string> ExportSample()
    {
        var path = Path.Combine(NewDirectory(), "review.scbundle");
        var exported = await ComparisonBundleService.Instance.ExportAsync(SampleBundle(), path);
        exported.Success.Should().BeTrue(exported.ErrorMessage);
        return path;
    }

    private static ComparisonBundle SampleBundle()
    {
        return new ComparisonBundle
        {
            Manifest = new BundleManifest
            {
                BundleId = "bundlehash0001",
                BundleVersion = "1.0.0",
                Profile = BundleProfile.Share,
                CreatedAt = new DateTime(2026, 10, 3, 12, 0, 0, DateTimeKind.Utc),
                AppVersion = "3.0.0",
                DeltaSpecVersion = "1.0",
                Privacy = new BundlePrivacyPolicy
                {
                    IncludesRawData = false,
                    IncludesFilePaths = false,
                    IncludesMachineName = false,
                    DataClassification = "test"
                },
                Contents = new BundleContentsManifest
                {
                    IncludesFindings = true,
                    IncludesRepro = true,
                    IncludesEnvironment = true,
                    IncludesInsights = false,
                    IncludesAssets = false,
                    IncludesAudit = false,
                    IncludesEvidence = false
                }
            },
            Findings = new BundleFindings
            {
                Deltas =
                [
                    new BundleDelta
                    {
                        Id = "FailurePresence",
                        Name = "Failure Events",
                        Explanation = "One run failed",
                        DeltaType = "Event",
                        Status = "Present",
                        LeftValue = 1,
                        RightValue = 0,
                        Delta = -1,
                        Magnitude = 1,
                        Confidence = 1,
                        VisualAnchorTime = 0.5,
                        IsMeaningful = true
                    }
                ],
                WhyExplanations =
                [
                    new WhyExplanation
                    {
                        DeltaId = "FailurePresence",
                        DeltaName = "Failure Events",
                        Explanation = "Path A failed and path B did not."
                    }
                ],
                ComparativeSummary = "One failure delta."
            },
            Repro = new BundleRepro
            {
                InputFingerprint = "abc",
                DeltaHash = "def",
                DeterminismEnabled = true,
                DeltaSpecVersion = "1.0",
                ReproducibilityBadge = "Reproducible",
                AlignmentMode = "ByStep",
                TimestepCount = 4
            },
            Environment = new BundleEnvironment
            {
                AppVersion = "3.0.0",
                Platform = "Windows",
                DotNetVersion = "9.0.0",
                Is64BitProcess = true,
                ProcessorCount = 4
            }
        };
    }

    private static V1.ComparisonBundlePayload SampleV1Payload()
    {
        var comparisonId = Guid.Parse("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        var created = new DateTimeOffset(2026, 10, 3, 12, 0, 0, TimeSpan.Zero);
        var manifest = new V1.ComparisonBundleManifest
        {
            BundleVersion = "1.0.0",
            BundleId = Guid.Parse("11111111-2222-3333-4444-555555555555"),
            CreatedUtc = created,
            Profile = V1.BundleProfile.Share,
            App = new V1.AppInfo
            {
                Name = "ScalarScope",
                AppVersion = "3.0.0",
                DeltaSpecVersion = "1.0",
                Build = new V1.BuildInfo
                {
                    Channel = V1.BuildChannel.Stable,
                    WarningsCount = 0
                }
            },
            Comparison = new V1.ComparisonInfo
            {
                ComparisonId = comparisonId,
                LabelA = "baseline",
                LabelB = "candidate",
                AlignmentMode = V1.AlignmentMode.Step,
                CompareLength = 4
            },
            Reproducibility = new V1.ReproducibilityInfo
            {
                Status = V1.ReproStatus.Reproducible,
                Reasons = Array.Empty<V1.ReproReason>()
            },
            Contents = new V1.ContentsInfo
            {
                Required = V1.BundleBuilder.RequiredFiles,
                Optional = Array.Empty<string>()
            },
            Integrity = new V1.IntegrityInfo
            {
                HashAlgorithm = V1.HashAlgorithm.Sha256,
                Files = Array.Empty<V1.FileIntegrityEntry>(),
                BundleHash = "",
                BundleHashDefinition = V1.BundleHashAlgorithm.BundleHashDefinition
            },
            Privacy = new V1.PrivacyInfo
            {
                ContainsRawRunData = false,
                ContainsPII = false,
                Redactions = new[] { V1.PrivacyRedaction.None }
            }
        };

        return new V1.ComparisonBundlePayload
        {
            Manifest = manifest,
            Repro = new V1.ReproPayload
            {
                ComparisonId = comparisonId,
                DeltaSpecVersion = "1.0",
                Inputs = new V1.InputsInfo
                {
                    RunA = Run("a"),
                    RunB = Run("b")
                },
                Preset = new V1.PresetInfo
                {
                    PresetId = "tfrt",
                    PresetVersion = "1",
                    PresetHash = "abc",
                    Normalization = new V1.NormalizationInfo
                    {
                        InputNormalizerVersion = "1",
                        Rules = Array.Empty<string>()
                    },
                    AlignmentDefaults = new V1.AlignmentDefaultsInfo
                    {
                        Mode = V1.AlignmentMode.Step
                    }
                },
                Determinism = new V1.DeterminismInfo(),
                Results = new V1.ResultsInfo
                {
                    DeltaHash = "def",
                    DeltasCount = 1,
                    ComputedUtc = created
                },
                Environment = new V1.EnvironmentInfo
                {
                    Os = "Windows",
                    Dotnet = "9.0.0",
                    AppVersion = "3.0.0"
                }
            },
            Deltas = new V1.DeltasPayload
            {
                ComparisonId = comparisonId,
                DeltaSpecVersion = "1.0",
                GeneratedUtc = created,
                Deltas =
                [
                    new V1.DeltaEntry
                    {
                        Id = "FailurePresence",
                        Status = V1.DeltaStatus.Present,
                        Name = "Failure Events",
                        Explanation = "One run failed",
                        TriggerType = V1.TriggerType.Event,
                        Confidence = 1
                    }
                ]
            },
            Why = new V1.WhyPayload
            {
                ComparisonId = comparisonId,
                DeltaSpecVersion = "1.0",
                Why =
                [
                    new V1.WhyEntry
                    {
                        DeltaId = "FailurePresence",
                        WhyFired = "Path A failed",
                        ConditionSummary = "One failure",
                        Guardrails = Array.Empty<string>(),
                        ParameterChips = Array.Empty<V1.ParameterChip>(),
                        Confidence = new V1.WhyConfidenceInfo
                        {
                            Value = 1,
                            Label = "high"
                        }
                    }
                ]
            },
            SummaryMarkdown = "One failure delta."
        };
    }

    private static V1.RunInputInfo Run(string name)
    {
        return new V1.RunInputInfo
        {
            SourceType = V1.RunSourceType.File,
            SchemaVersion = "1.0",
            Fingerprint = name,
            NormalizedFingerprint = name,
            FileName = name + ".json"
        };
    }

    private static byte[] ReadZipEntry(string bundlePath, string entryName)
    {
        using var archive = ZipFile.OpenRead(bundlePath);
        var entry = archive.GetEntry(entryName);
        entry.Should().NotBeNull();
        using var stream = entry!.Open();
        using var memory = new MemoryStream();
        stream.CopyTo(memory);
        return memory.ToArray();
    }

    private static void ReplaceZipText(string bundlePath, string entryName, Func<string, string> rewrite)
    {
        string rewritten;
        using (var archive = ZipFile.Open(bundlePath, ZipArchiveMode.Update))
        {
            var entry = archive.GetEntry(entryName);
            entry.Should().NotBeNull();
            using (var stream = entry!.Open())
            using (var reader = new StreamReader(stream, Encoding.UTF8))
            {
                rewritten = rewrite(reader.ReadToEnd());
            }

            entry.Delete();
            var replacement = archive.CreateEntry(entryName);
            using var output = replacement.Open();
            var bytes = Utf8NoBom.GetBytes(rewritten);
            output.Write(bytes, 0, bytes.Length);
        }
    }

    private static string NewDirectory()
    {
        var directory = Path.Combine(Path.GetTempPath(), "scalarscope-hash-" + Guid.NewGuid().ToString("n"));
        Directory.CreateDirectory(directory);
        return directory;
    }
}
