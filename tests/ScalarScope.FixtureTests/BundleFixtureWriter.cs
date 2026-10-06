using System.Text.Json;
using FluentAssertions;
using ScalarScope.Models;
using ScalarScope.Services;
using ScalarScope.Services.Connectors;
using Xunit;

namespace ScalarScope.FixtureTests;

/// <summary>
/// Bundles written by the 2.0 code, kept in tests/Fixtures/Bundles so the Rust review is tested against
/// what 2.0 actually wrote, and a bundle written by the Rust review, checked against the 2.0 importer.
/// Set SCALARSCOPE_WRITE_BUNDLE_FIXTURES=1 to rewrite the 2.0 files; otherwise nothing is written.
/// </summary>
public class BundleFixtureWriter
{
    private static string Root()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory != null && !File.Exists(Path.Combine(directory.FullName, "ScalarScope.sln")))
            directory = directory.Parent;
        return directory?.FullName ?? throw new InvalidOperationException("The repo root was not found.");
    }

    private static string Fixtures() => Path.Combine(Root(), "tests", "Fixtures", "Bundles");

    private static GeometryRun Sample(string name)
    {
        var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
        var json = File.ReadAllText(Path.Combine(Root(), "src/ScalarScope/Resources/Raw/Samples", name));
        return JsonSerializer.Deserialize<GeometryRun>(json, options)!;
    }

    [Fact]
    public async Task Writes_the_2_0_bundle_fixtures_when_asked()
    {
        if (Environment.GetEnvironmentVariable("SCALARSCOPE_WRITE_BUNDLE_FIXTURES") != "1")
            return;
        Directory.CreateDirectory(Fixtures());

        // A geometry comparison bundle, as the 2.0 Compare page exported it.
        var result = CanonicalDeltaService.ComputeDeltasWithAlignment(
            Sample("orthogonal_professors.json"), Sample("correlated_professors.json"), TemporalAlignment.ByStep, 1.0);
        var bundle = ComparisonBundleService.Instance.CreateBundle(result, null, null, BundleProfile.Review);
        var compare = Path.Combine(Fixtures(), "dotnet-geometry-compare.scbundle");
        var exported = await ComparisonBundleService.Instance.ExportAsync(bundle, compare);
        exported.Success.Should().BeTrue(exported.ErrorMessage);

        // An inference review, as 2.0 auto-saved it: review/review.json only.
        var comparer = new RunTraceComparer();
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var comparison = comparer.Compare(baseline, optimized, ComparisonIntent.TfrtOptimization("Baseline", "Optimized"));
        comparer.WriteReviewBundle(comparison, baseline, optimized, Path.Combine(Fixtures(), "dotnet-inference-review.scbundle"));
    }

    [Fact]
    public async Task A_bundle_the_rust_review_wrote_imports_in_2_0()
    {
        var path = Path.Combine(Fixtures(), "rust-inference-review.scbundle");
        File.Exists(path).Should().BeTrue("rust/tests/compat_tests.rs writes it");
        var imported = await BundleImportService.Instance.ImportAsync(path);
        imported.Success.Should().BeTrue(imported.ErrorMessage);
        imported.LoadedBundle.Should().NotBeNull();
        imported.LoadedBundle!.Deltas.Should().NotBeEmpty();
    }
}
