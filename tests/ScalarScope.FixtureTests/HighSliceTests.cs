using System.Globalization;
using System.IO.Compression;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using FluentAssertions;
using ScalarScope.Models;
using ScalarScope.Services.Connectors;
using Xunit;

namespace ScalarScope.FixtureTests;

public class HighSliceTests
{
    [Fact]
    public void Professor_samples_have_a_trajectory()
    {
        var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
        foreach (var name in new[] { "orthogonal_professors.json", "correlated_professors.json" })
        {
            var json = File.ReadAllText(Path.Combine(RepoRoot(), "src/ScalarScope/Resources/Raw/Samples", name));
            var run = JsonSerializer.Deserialize<GeometryRun>(json, options);
            run.Should().NotBeNull();
            run!.Trajectory.Timesteps.Should().NotBeEmpty();
        }
    }

    [Fact]
    public async Task Golden_runtrace_imports_the_latency_series_and_milestones()
    {
        var connector = new TensorFlowRTOfflineConnector();
        foreach (var name in new[] { "baseline_tfrt_runtrace.json", "optimized_tfrt_runtrace.json" })
        {
            var viaDto = FixtureLoader.LoadRunTrace(name);
            var viaOpen = await connector.ImportRuntimeAsync(FixtureLoader.GetFixturePath(name));

            viaOpen.Scalars.GetByName("latency_ms")!.Values.Should().Equal(
                viaDto.Scalars.GetByName("latency_ms")!.Values);
            viaOpen.Milestones.List.Select(item => (item.Type, item.Step)).Should().Equal(
                viaDto.Milestones.List.Select(item => (item.Type, item.Step)));
        }
    }

    [Fact]
    public void Inference_review_file_hashes_the_findings()
    {
        var comparer = new RunTraceComparer();
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var comparison = comparer.Compare(baseline, optimized, ComparisonIntent.TfrtOptimization("Before", "After"));
        var directory = Path.Combine(Path.GetTempPath(), "scalarscope-review-" + Guid.NewGuid().ToString("N"));
        var path = Path.Combine(directory, "inference-review.scbundle");
        try
        {
            var bundle = comparer.WriteReviewBundle(comparison, baseline, optimized, path);
            var again = comparer.ExportReviewBundle(comparison, baseline, optimized);

            bundle.RecomputeDisabled.Should().BeTrue();
            again.BundleHash.Should().Be(bundle.BundleHash);
            bundle.BundleHash.Should().Be(Sha256(ReviewPreimage(bundle)));
            path.Should().EndWith(".scbundle");

            using var archive = ZipFile.OpenRead(path);
            var entry = archive.GetEntry("review/review.json");
            entry.Should().NotBeNull();
            using var reader = new StreamReader(entry!.Open());
            var json = reader.ReadToEnd();
            json.Should().Contain("\"recomputeDisabled\":true");
            json.Should().Contain(bundle.Comparison.Alignment.Summary);
        }
        finally
        {
            if (Directory.Exists(directory))
                Directory.Delete(directory, true);
        }
    }

    [Fact]
    public void Open_highs_say_what_the_shell_does()
    {
        var playback = Read("src/ScalarScope/Views/Controls/PlaybackControl.xaml");
        playback.Should().Contain("Loop");
        playback.Should().NotContain("Particles");
        playback.Should().NotContain("Motion Blur");
        playback.Should().NotContain("Cinematic");

        var home = Read("src/ScalarScope/Views/WelcomePage.xaml");
        home.Should().Contain("Orthogonal and correlated professors");
        home.Should().NotContain("TFRT Optimization Example");

        var compare = Read("src/ScalarScope/Views/ComparisonPage.xaml");
        compare.Should().NotContain("FallbackValue='TFRT'");
        compare.Should().Contain("HasFrameworkName");

        var viewModel = Read("src/ScalarScope/ViewModels/ComparisonViewModel.cs");
        viewModel.Should().Contain("private string _frameworkName = \"\";");
        viewModel.Should().Contain("FrameworkName = \"Geometry\";");
        viewModel.Should().Contain("FrameworkType.TensorFlowRT => \"TFRT\"");

        var panel = Read("src/ScalarScope/Views/Controls/BundleExportPanel.xaml.cs");
        panel.Should().Contain("assets/export-info.txt");
        panel.Should().NotContain("screenshots, cards");

        var readme = Read("src/ScalarScope/Services/ComparisonBundleService.cs");
        readme.Should().Contain("export-info.txt");
        readme.Should().NotContain("screenshots, cards");

        var audit = Read("docs/THEME_AUDIT.md");
        audit.Should().Contain("Home");
        audit.Should().Contain("Compare");
        audit.Should().Contain("Guide");
        audit.Should().Contain("Settings");
        audit.Should().Contain("White text on `#4ecdc4`");
        audit.Should().Contain("**Overall Status**: Fail");
        audit.Should().NotContain("Ready for RC1");
        audit.Should().NotContain("7 pages");

        var canvas = Read("src/ScalarScope/Views/Controls/ComparisonTrajectoryCanvas.cs");
        canvas.Should().Contain("using var dash = SKPathEffect.CreateDash");
        canvas.Should().Contain("LabelTypeface");
        canvas.Should().Contain("if (!IsShown(this))");
        Read("src/ScalarScope/Views/Controls/OverlayComparisonCanvas.cs")
            .Should().Contain("using var dash = SKPathEffect.CreateDash");

        var grid = Read("src/VortexKit/Theme/VortexColors.cs");
        grid.Should().Contain("#70708a");
        grid.Should().NotContain("#2a2a4e");
        Contrast("#70708a", "#0f0f1a").Should().BeGreaterThanOrEqualTo(3.0);

        var svg = Read("src/VortexKit/Core/SvgExportService.cs");
        svg.Should().Contain("GridMajor { get; init; } = \"#8e8eae\"");
        svg.Should().Contain("GridMinor { get; init; } = \"#6b6b8a\"");
        svg.Should().Contain("GridMajor = \"#5c5c72\"");
        svg.Should().Contain("GridMinor = \"#6e6e88\"");
        svg.Should().Contain("GridMajor = \"#666666\"");
        Contrast("#8e8eae", "#1E1E2E").Should().BeGreaterThanOrEqualTo(3.0);
        Contrast("#6b6b8a", "#1E1E2E").Should().BeGreaterThanOrEqualTo(3.0);
        Contrast("#5c5c72", "#FFFFFF").Should().BeGreaterThanOrEqualTo(3.0);
        Contrast("#6e6e88", "#FFFFFF").Should().BeGreaterThanOrEqualTo(3.0);
        Contrast("#666666", "#000000").Should().BeGreaterThanOrEqualTo(3.0);
    }

    private static string ReviewPreimage(ReviewOnlyBundle bundle)
    {
        var findings = string.Join("\n", bundle.Comparison.Deltas.Select(delta =>
            "finding:" + string.Join("|",
                delta.DeltaType,
                delta.Signal,
                delta.ValueA.ToString("R", CultureInfo.InvariantCulture),
                delta.ValueB.ToString("R", CultureInfo.InvariantCulture),
                delta.Fired ? "fired" : "quiet",
                delta.IsSuppressed ? "suppressed" : "live",
                delta.Interpretation ?? "")));
        var summary = bundle.FingerprintSummary;
        var fingerprints = "fingerprint:" + string.Join("|",
            summary.LabelA,
            summary.LabelB,
            summary.ModelFingerprintA,
            summary.ModelFingerprintB,
            summary.DatasetFingerprintA,
            summary.DatasetFingerprintB,
            summary.CodeFingerprintA,
            summary.CodeFingerprintB,
            summary.EnvironmentFingerprintA,
            summary.EnvironmentFingerprintB,
            summary.FrameworkA,
            summary.FrameworkB);
        return "alignment:" + bundle.Comparison.Alignment.Summary + "\n" + findings + "\n" + fingerprints;
    }

    private static string Sha256(string content)
    {
        var hash = SHA256.HashData(Encoding.UTF8.GetBytes(content));
        return Convert.ToHexString(hash).ToLowerInvariant();
    }

    private static double Contrast(string foreground, string background)
    {
        var lighter = Math.Max(Luminance(foreground), Luminance(background));
        var darker = Math.Min(Luminance(foreground), Luminance(background));
        return (lighter + 0.05) / (darker + 0.05);
    }

    private static double Luminance(string hex)
    {
        hex = hex.TrimStart('#');
        double Channel(string pair)
        {
            var value = int.Parse(pair, NumberStyles.HexNumber, CultureInfo.InvariantCulture) / 255.0;
            return value <= 0.03928 ? value / 12.92 : Math.Pow((value + 0.055) / 1.055, 2.4);
        }

        var red = Channel(hex[..2]);
        var green = Channel(hex[2..4]);
        var blue = Channel(hex[4..6]);
        return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    }

    private static string Read(string relative) => File.ReadAllText(Path.Combine(RepoRoot(), relative));

    private static string RepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("repo root");
    }
}
