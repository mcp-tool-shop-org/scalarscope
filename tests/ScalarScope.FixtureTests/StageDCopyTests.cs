using FluentAssertions;
using ScalarScope.Services;
using ScalarScope.Services.Connectors;
using Xunit;

namespace ScalarScope.FixtureTests;

public class StageDCopyTests
{
    [Fact]
    public void Unfired_summary_does_not_invent_a_tie_or_a_matched_machine()
    {
        var withheld = Summary(
            [Delta("ΔTc", false, "steady state not detected")],
            Prints(Diff("dataset")));
        withheld.Should().Contain("Steady state was not detected");
        withheld.Should().NotContain("no significant change");
        withheld.Should().NotContain("No runtime failures were detected");
        withheld.Should().NotContain("identical inputs and hardware");
        withheld.Should().Contain("does not establish that the inputs and hardware matched");

        var fewer = Summary(
            [Delta("ΔF", false, "Right has 2 fewer 3-sigma outliers; an elimination does not fire ΔF")],
            Prints());
        fewer.Should().Contain("elimination does not fire ΔF");
        fewer.Should().NotContain("No runtime failures were detected");
        fewer.Should().NotContain("identical inputs and hardware");

        var tie = Summary(
            [Delta("ΔF", false, "No change in runtime anomalies"), Delta("ΔTc", false, "No change in stabilization time", 4, 4)],
            Prints(Diff("environment")));
        tie.Should().Contain("ΔF did not fire");
        tie.Should().Contain("same step");
        tie.Should().NotContain("no significant change");
        tie.Should().NotContain("Steady state was not detected");
        tie.Should().NotContain("identical inputs and hardware");

        var fired = Summary(
            [Delta("ΔTc", true, "Stabilizes 4 steps earlier", 10, 6)],
            Prints(Diff("model", expected: true)));
        fired.Should().Contain("4 steps earlier");
        fired.Should().Contain("model fingerprint differs");
        fired.Should().NotContain("identical inputs and hardware");
    }

    [Fact]
    public void File_banner_names_a_geometry_run_or_a_latency_trace()
    {
        var state = ErrorStateMapping.GetByCode("FILE_FORMAT_INVALID");
        state.Should().NotBeNull();
        state!.UserExplanation.Should().NotContain("training run");
        state.UserExplanation.Should().Contain(".json").And.Contain(".csv");
        state.UserExplanation.Should().Contain("latency trace");
        string.Join(" ", state.SuggestedActions).Should().NotContain("training export");
        string.Join(" ", state.SuggestedActions).Should().NotContain("TRAJECTORY_FORMAT");
    }

    [Fact]
    public void Open_pages_say_what_the_shell_does()
    {
        var settings = Read("src/ScalarScope/Views/SettingsPage.xaml");
        settings.Should().NotContain("cryptographic hash");
        settings.Should().NotContain("Show the welcome demo again");
        settings.Should().NotContain("Clear the recent files list");
        settings.Should().Contain("content check, not a signature");
        settings.Should().Contain("The demo does not start.");
        settings.Should().Contain("do not show that list");
        settings.Should().Contain("does not clear the comparison log");

        var home = Read("src/ScalarScope/Views/WelcomePage.xaml");
        home.Should().NotContain("SHA-256 integrity");
        home.Should().NotContain("verified against the original hash");
        home.Should().Contain("content check, not a signature");
        home.Should().Contain("It is not a signature.");

        var help = Read("src/ScalarScope/Views/HelpPage.xaml");
        var pathA = help.IndexOf("Path A: Orthogonal", StringComparison.Ordinal);
        var pathB = help.IndexOf("Path B: Correlated", StringComparison.Ordinal);
        pathA.Should().BeGreaterThan(0);
        pathB.Should().BeGreaterThan(pathA);
        help[pathA..help.IndexOf("/>", pathA, StringComparison.Ordinal)].Should().Contain("#4ecdc4");
        help[pathB..help.IndexOf("/>", pathB, StringComparison.Ordinal)].Should().Contain("#ff6b6b");

        var script = Read("docs/DEMO_SCRIPT.md");
        script.Should().NotContain("[Overview");
        script.Should().NotContain("[Trajectory");
        script.Should().NotContain("[Geometry");
        script.Should().NotContain("Load Geometry Run");
        script.Should().Contain("[Home]");
        script.Should().Contain("[Compare]");
        script.Should().Contain("[Guide]");
        script.Should().Contain("[Settings]");

        var quick = Read("docs/QUICK_REFERENCE.md");
        quick.Should().Contain("The Windows key hook does not send E.");
        quick.Should().Contain("No notification is shown.");
        quick.Should().NotContain("| `Ctrl+E` | Quick screenshot");

        var runbook = Read("docs/INSTALL_RUNBOOK.md");
        runbook.Should().Contain("No notification is shown.");
        runbook.Should().NotContain("notification appears");

        var beginners = Read("site/src/content/docs/handbook/beginners.md");
        // The Rust review's shortcuts, not 2.0's.
        beginners.Should().Contain("| `F1` | Guide |");
        beginners.Should().NotContain("`Ctrl+S` or `Ctrl+E` | Quick export");

        var readme = Read("README.md");
        readme.Should().NotContain("ScalarScope-Desktop/readme.png");
        readme.Should().Contain("| `F1` | Guide |");
        readme.Should().NotContain("Ctrl+E");
        foreach (var language in new[] { "ja", "zh", "es", "fr", "hi", "it", "pt-BR" })
            Read($"README.{language}.md").Should().NotContain("ScalarScope-Desktop/readme.png");

        var site = Read("site/src/site-config.ts");
        site.Should().Contain("href: 'https://apps.microsoft.com/detail/9P3HT1PHBKQK', label: 'Get from Microsoft Store'");
        site.Should().NotContain("href: '#quickstart', label: 'Get from Microsoft Store'");
    }

    private static string Summary(IReadOnlyList<ComparisonDelta> deltas, FingerprintComparison fingerprints)
    {
        var result = new ComparisonResult
        {
            ComparisonId = "stage-d-copy",
            ComputedUtc = DateTimeOffset.UnixEpoch,
            Intent = ComparisonIntent.TfrtOptimization(),
            Fingerprints = fingerprints,
            Alignment = new AlignmentResult
            {
                Mode = ScalarScope.Services.Connectors.AlignmentMode.RuntimeMilestone,
                AlignedStepCount = 0,
                SkippedStepsA = 0,
                SkippedStepsB = 0,
                Issues = []
            },
            Deltas = deltas,
            Warnings = [],
            DeltaSpecVersion = "1.0.0",
            PresetId = "stage-d-copy"
        };
        return ExecutiveSummaryGenerator.GenerateInferenceOptimizationSummary(result);
    }

    private static ComparisonDelta Delta(string type, bool fired, string interpretation, double a = 0, double b = 0) => new()
    {
        DeltaType = type,
        Signal = "latency_ms",
        ValueA = a,
        ValueB = b,
        Confidence = 0,
        Fired = fired,
        Interpretation = interpretation
    };

    private static FingerprintComparison Prints(params FingerprintDifference[] diffs) => new()
    {
        Differences = diffs,
        Issues = []
    };

    private static FingerprintDifference Diff(string category, bool expected = false) => new()
    {
        Category = category,
        FingerprintA = "a",
        FingerprintB = "b",
        IsExpectedForOptimization = expected
    };

    private static string Read(string relative)
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return File.ReadAllText(Path.Combine(dir ?? throw new InvalidOperationException("repo root"), relative));
    }
}
