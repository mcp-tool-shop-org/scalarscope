using FluentAssertions;
using ScalarScope.Models;
using ScalarScope.Services;
using ScalarScope.SoakTests;
using ScalarScope.Views.Controls;
using Xunit;

namespace ScalarScope.FixtureTests;

public class HealthFixTests
{
    private static string FindRepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("Could not find repo root");
    }

    private static GeometryRun Line(string id, int count, double y, int nanAt = -1)
    {
        var steps = new List<TrajectoryTimestep>(count);
        for (var i = 0; i < count; i++)
        {
            var x = count == 1 ? 0 : (double)i / (count - 1);
            var px = i == nanAt ? double.NaN : x;
            steps.Add(new TrajectoryTimestep { T = x, State2D = [px, y] });
        }

        return new GeometryRun
        {
            Metadata = new RunMetadata { RunId = id },
            Trajectory = new Trajectory { Timesteps = steps }
        };
    }

    [Fact]
    public void Frechet_and_dtw_keep_the_distance_for_a_pair_that_fits()
    {
        var service = new ComparativeAnalysisService();
        var left = Line("left", 2, 0);
        var right = Line("right", 2, 1);

        var frechet = service.ComputeFrechet(left, right);
        frechet.IsValid.Should().BeTrue();
        frechet.Reason.Should().BeNull();
        frechet.Distance.Should().BeApproximately(1, 1e-9);

        var dtw = service.ComputeDtw(left, right);
        dtw.IsValid.Should().BeTrue();
        dtw.Reason.Should().BeNull();
        dtw.Distance.Should().BeApproximately(2, 1e-9);
        dtw.NormalizedDistance.Should().BeApproximately(0.5, 1e-9);
        dtw.WarpingPath.Should().NotBeEmpty();
    }

    [Fact]
    public void Identical_line_under_the_budget_has_distance_zero()
    {
        var service = new ComparativeAnalysisService();
        var line = Line("same", 1000, 0);

        var frechet = service.ComputeFrechet(line, line);
        frechet.IsValid.Should().BeTrue();
        frechet.Distance.Should().Be(0);

        var dtw = service.ComputeDtw(line, line);
        dtw.IsValid.Should().BeTrue();
        dtw.Distance.Should().Be(0);
    }

    [Fact]
    public void Non_finite_link_stops_without_a_distance()
    {
        var service = new ComparativeAnalysisService();
        var left = Line("nan", 3, 0, nanAt: 1);
        var right = Line("ok", 3, 0);

        var frechet = service.ComputeFrechet(left, right);
        frechet.IsValid.Should().BeFalse();
        frechet.Reason.Should().Be(ComparativeAnalysisService.NonFiniteLinkReason);
        double.IsFinite(frechet.Distance).Should().BeFalse();

        var dtw = service.ComputeDtw(left, right);
        dtw.IsValid.Should().BeFalse();
        dtw.Reason.Should().Be(ComparativeAnalysisService.NonFiniteLinkReason);
        dtw.WarpingPath.Should().BeEmpty();
    }

    [Fact]
    public void Over_budget_pair_is_refused_before_the_table_exists()
    {
        var side = (int)Math.Floor(Math.Sqrt(ComparativeAnalysisService.DistanceCellBudget)) + 1;
        ((long)side * side).Should().BeGreaterThan(ComparativeAnalysisService.DistanceCellBudget);

        var service = new ComparativeAnalysisService();
        var left = Line("wide-a", side, 0);
        var right = Line("wide-b", side, 1);

        var frechet = service.ComputeFrechet(left, right);
        frechet.IsValid.Should().BeFalse();
        frechet.Reason.Should().Be(ComparativeAnalysisService.DistanceBudgetReason);

        var dtw = service.ComputeDtw(left, right);
        dtw.IsValid.Should().BeFalse();
        dtw.Reason.Should().Be(ComparativeAnalysisService.DistanceBudgetReason);
        dtw.WarpingPath.Should().BeEmpty();
    }

    [Fact]
    public void Guide_search_matches_each_token_not_a_contiguous_phrase()
    {
        const string failure = "delta glossary failure ΔF rate";
        const string convergence = "delta convergence ΔTc time speed";

        failure.Contains("failure rate", StringComparison.Ordinal).Should().BeFalse();
        GuideSearch.Matches(failure, "failure rate").Should().BeTrue();
        GuideSearch.Matches(convergence, "failure rate").Should().BeFalse();
        GuideSearch.Matches(convergence, "convergence time").Should().BeTrue();
        GuideSearch.Matches(failure, "failure").Should().BeTrue();
        GuideSearch.Matches(failure, "failure missing").Should().BeFalse();
        GuideSearch.Matches(failure, "δf").Should().BeFalse();
    }

    [Fact]
    public void Help_page_uses_token_search()
    {
        var page = File.ReadAllText(Path.Combine(FindRepoRoot(), "src", "ScalarScope", "Views", "HelpPage.xaml.cs"));
        page.Should().Contain("GuideSearch.Matches(kvp.Key, query)");
        page.Should().NotContain("kvp.Key.Contains(query)");
    }

    [Fact]
    public void Alignment_captions_are_the_short_sentences()
    {
        AlignmentCaption.For(TemporalAlignment.ByStep).Should().Be("Aligned by training step");
        AlignmentCaption.For(TemporalAlignment.ByConvergence).Should().Be("Aligned at convergence");
        AlignmentCaption.For(TemporalAlignment.ByFirstInstability).Should().Be("Aligned at first change");
    }

    [Fact]
    public void Comparison_page_binds_the_alignment_caption()
    {
        var root = FindRepoRoot();
        var xaml = File.ReadAllText(Path.Combine(root, "src", "ScalarScope", "Views", "ComparisonPage.xaml"));
        var start = xaml.IndexOf("<controls:AlignmentControl", StringComparison.Ordinal);
        var end = xaml.IndexOf("/>", start, StringComparison.Ordinal);
        var tag = xaml[start..end];
        tag.Should().Contain("AlignmentDescription=\"{Binding AlignmentDescription}\"");
        tag.Should().Contain("SelectedAlignment=\"{Binding SelectedAlignment}\"");

        var control = File.ReadAllText(Path.Combine(root, "src", "ScalarScope", "Views", "Controls", "AlignmentControl.xaml.cs"));
        control.Should().Contain("OnAlignmentDescriptionChanged");
        var conv = control.IndexOf("OnByConvergenceTapped", StringComparison.Ordinal);
        var assign = control.IndexOf("AlignmentCaption.For(TemporalAlignment.ByConvergence)", conv, StringComparison.Ordinal);
        var set = control.IndexOf("SelectedAlignment = TemporalAlignment.ByConvergence", conv, StringComparison.Ordinal);
        assign.Should().BeGreaterThan(conv);
        set.Should().BeGreaterThan(assign);
    }

    [Fact]
    public void Delta_row_columns_stay_distinct_and_the_row_seeks()
    {
        DeltaZone.InfoButtonColumn.Should().Be(4);
        DeltaZone.CopyButtonColumn.Should().Be(5);
        DeltaZone.AnchorColumn.Should().Be(6);

        var root = FindRepoRoot();
        var zone = File.ReadAllText(Path.Combine(root, "src", "ScalarScope", "Views", "Controls", "DeltaZone.xaml.cs"));
        zone.Should().Contain("Grid.SetColumn(infoButton, InfoButtonColumn)");
        zone.Should().Contain("Grid.SetColumn(copyButton, CopyButtonColumn)");
        zone.Should().Contain("Grid.SetColumn(anchorLabel, AnchorColumn)");
        zone.Should().NotContain("Grid.SetColumn(copyButton, 4)");
        zone.Should().Contain("anchorLabel.GestureRecognizers.Add(anchorTap)");
        zone.Should().Contain("row.Stroke");
        zone.Should().Contain("#4ecdc4");
        zone.Should().NotContain("Implementation would update visual states");

        var page = File.ReadAllText(Path.Combine(root, "src", "ScalarScope", "Views", "ComparisonPage.xaml.cs"));
        page.Should().Contain("deltaZone.DeltaClicked +=");
        page.Should().Contain("ViewModel.JumpToDeltaAnchor(delta)");
    }

    [Fact]
    public void Grid_dash_is_disposed_with_the_paint()
    {
        var canvas = File.ReadAllText(Path.Combine(FindRepoRoot(), "src", "VortexKit", "Core", "AnimatedCanvas.cs"));
        canvas.Should().Contain("using var dash = SKPathEffect.CreateDash");
        canvas.Should().NotContain("paint.PathEffect = SKPathEffect.CreateDash");
        var dash = canvas.IndexOf("using var dash", StringComparison.Ordinal);
        var paint = canvas.IndexOf("using var paint", StringComparison.Ordinal);
        dash.Should().BeGreaterThan(0);
        paint.Should().BeGreaterThan(dash);
    }

    [Fact]
    public void Release_workflow_pins_the_toolchain_and_limits_write()
    {
        var yml = File.ReadAllText(Path.Combine(FindRepoRoot(), ".github", "workflows", "release.yml"));
        yml.Should().Contain("dtolnay/rust-toolchain@7e38f4b43b4db5c8dd498af069a4f6196df1d067");
        yml.Should().Contain("toolchain: 1.98.1");
        yml.Should().NotContain("@stable");
        yml.Should().Contain("contents: read");
        yml.Should().Contain("persist-credentials: false");
        yml.Should().Contain("actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
        yml.Should().Contain("actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c");
        yml.Should().Contain("3.1.1.0");
        yml.Should().NotContain("llvm-tools");

        var buildAt = yml.IndexOf("build-release:", StringComparison.Ordinal);
        var publishAt = yml.IndexOf("\n  publish:", StringComparison.Ordinal);
        var writeAt = yml.IndexOf("contents: write", StringComparison.Ordinal);
        publishAt.Should().BeGreaterThan(buildAt);
        writeAt.Should().BeGreaterThan(publishAt);
        yml.LastIndexOf("contents: write", StringComparison.Ordinal).Should().Be(writeAt);
        yml.Substring(buildAt, publishAt - buildAt).Should().NotContain("contents: write");
    }

    [Fact]
    public void Phase32_floors_stay_in_the_runner()
    {
        var runner = File.ReadAllText(Path.Combine(
            FindRepoRoot(), "tests", "ScalarScope.SoakTests", "Phase32ValidationRunner.cs"));
        runner.Should().Contain("ResolutionStepsFloor = 3");
        runner.Should().Contain("MinDurationFloor = 4");
        runner.Should().Contain("PersistenceStepsFloor = 3");
        runner.Should().Contain("StepCount = 40");
    }

    [Fact]
    public async Task Phase32_floors_stay_locked_and_do_not_rewrite_the_report()
    {
        var reportPath = Path.Combine(FindRepoRoot(), "phase32_validation_report.md");
        var before = await File.ReadAllBytesAsync(reportPath);

        var report = await new Phase32ValidationRunner().RunValidationAsync();

        report.Locked.Should().BeTrue(report.LockDecision);
        report.AllGatesPassed.Should().BeTrue();
        var after = await File.ReadAllBytesAsync(reportPath);
        after.Should().Equal(before);
    }
}
