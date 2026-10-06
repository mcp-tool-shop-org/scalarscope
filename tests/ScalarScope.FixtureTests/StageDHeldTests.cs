using FluentAssertions;
using VortexKit.Core;
using Xunit;

namespace ScalarScope.FixtureTests;

public class StageDHeldTests
{
    [Fact]
    public void Playhead_text_follows_time_and_drawn_labels()
    {
        CanvasAccess.Describe(0.42, null).Should().Be("Playback canvas. Playhead 42%.");
        CanvasAccess.Describe(double.NaN, ["  steady  ", "steady"]).Should().Be("Playback canvas. Playhead 0%. Labels: steady.");

        CanvasAccess.TryApplyKey("Left", 0, out var stayed, out var tap).Should().BeTrue();
        stayed.Should().Be(0);
        tap.Should().BeFalse();

        CanvasAccess.TryApplyKey("Right", 0.5, out var later, out tap).Should().BeTrue();
        later.Should().BeApproximately(0.51, 1e-12);
        tap.Should().BeFalse();

        CanvasAccess.TryApplyKey("Enter", 0.2, out var same, out tap).Should().BeTrue();
        same.Should().Be(0.2);
        tap.Should().BeTrue();

        CanvasAccess.TryApplyKey("S", 0.2, out _, out tap).Should().BeFalse();
        tap.Should().BeFalse();
    }

    [Fact]
    public void Canvas_key_path_is_wired_for_a_focused_view()
    {
        var canvas = File.ReadAllText(Path.Combine(Root(), "src/VortexKit/Core/AnimatedCanvas.cs"));
        canvas.Should().Contain("AutomationProperties.SetName");
        canvas.Should().Contain("SemanticProperties.SetDescription");
        canvas.Should().Contain("TryHandleFocusedKey");
        canvas.Should().Contain("IsTabStop = true");

        var hook = File.ReadAllText(Path.Combine(Root(), "src/ScalarScope/Platforms/Windows/App.xaml.cs"));
        hook.Should().Contain("TryHandleFocusedKey");
        hook.Should().Contain("VirtualKey.Enter => \"Enter\"");

        var paint = File.ReadAllText(Path.Combine(Root(), "src/ScalarScope/Views/Controls/AccessibleCanvasHook.cs"));
        paint.Should().Contain("PublishAccess()");
        paint.Should().NotContain("AnimatedCanvas playback)\n        {\n            playback.PublishAccess();\n            SemanticProperties.SetDescription(\n                view,\n                null");
    }

    [Fact]
    public void Default_svg_sample_is_a_thin_segment_and_glow_is_once()
    {
        var svg = new SvgExportService().BuildSvg(new SvgExportData
        {
            Trajectories =
            [
                new SvgTrajectoryData
                {
                    Label = "run",
                    Color = "#89B4FA",
                    Points = [new SvgPoint(0, 0), new SvgPoint(1, 1)]
                }
            ],
            Grid = new SvgGridData
            {
                MajorLines = [new SvgLine(-1, 0, 1, 0)],
                MinorLines = [new SvgLine(-1, 0.5, 1, 0.5)],
                Labels = [new SvgLabel(0, 0, "0")]
            },
            Annotations = [new SvgAnnotation(0, 0, "note")],
            Markers = [new SvgMarker(0, 0, 0.05, SvgMarkerType.Failure, "miss")]
        });

        svg.Should().Contain("viewBox=\"-1 -1 2 2\"");
        svg.Should().Contain("vector-effect=\"non-scaling-stroke\"");
        svg.Should().Contain("stroke-width=\"2\"");
        svg.Should().Contain("M 0 0 L 1 1");
        svg.Should().NotContain("stdDeviation=\"3\"");
        svg.Should().NotContain("font-size=\"12\"");
        svg.Should().NotContain("font-size=\"10\"");
        svg.Should().NotContain("width=\"20\"");
        svg.Should().NotContain("stroke-dasharray=\"2,2\"");
        Count(svg, "filter=\"url(#glow)\"").Should().Be(1);
        Count(svg, "feGaussianBlur").Should().Be(1);

        var path = svg.IndexOf("M 0 0 L 1 1", StringComparison.Ordinal);
        AttributeAfter(svg, "r=\"", path).Should().BeLessThan(0.05);
        var marker = svg.IndexOf("href=\"#failure-marker\"", StringComparison.Ordinal);
        AttributeAfter(svg, "width=\"", marker).Should().BeLessThan(1);
        AttributeAfter(svg, "stdDeviation=\"", 0).Should().BeLessThan(0.05);
    }

    [Fact]
    public void Colored_segments_share_one_glow_filter()
    {
        var points = new List<SvgPoint>();
        for (var i = 0; i < 8; i++)
            points.Add(new SvgPoint(i, i));

        var svg = new SvgExportService().BuildSvg(
            new SvgExportData
            {
                Trajectories = [new SvgTrajectoryData { Points = points }]
            },
            new SvgExportOptions { ColorMode = SvgColorMode.Time, EnableGlow = true });

        // Seven colored segments, plus the arrow marker path in defs.
        Count(svg, "stroke-linecap=\"round\"").Should().Be(points.Count - 1);
        Count(svg, "<path ").Should().Be(points.Count);
        svg.Should().Contain("<g id=\"layer-trajectory-0\" inkscape:groupmode=\"layer\" inkscape:label=\"Trajectory 1\" filter=\"url(#glow)\">");
        Count(svg, "filter=\"url(#glow)\"").Should().Be(1);
        Count(svg, "feGaussianBlur").Should().Be(1);
        foreach (var line in svg.Split('\n'))
        {
            if (line.Contains("<path ", StringComparison.Ordinal))
                line.Should().NotContain("filter=");
        }

        var quiet = new SvgExportService().BuildSvg(
            new SvgExportData
            {
                Trajectories = [new SvgTrajectoryData { Points = points }]
            },
            new SvgExportOptions { ColorMode = SvgColorMode.Time, EnableGlow = false });
        Count(quiet, "filter=\"url(#glow)\"").Should().Be(0);
        Count(quiet, "feGaussianBlur").Should().Be(0);
    }

    private static int Count(string text, string needle)
    {
        var count = 0;
        var index = 0;
        while ((index = text.IndexOf(needle, index, StringComparison.Ordinal)) >= 0)
        {
            count++;
            index += needle.Length;
        }
        return count;
    }

    private static double AttributeAfter(string svg, string name, int from)
    {
        var at = svg.IndexOf(name, from, StringComparison.Ordinal);
        at.Should().BeGreaterThan(0);
        var start = at + name.Length;
        var end = svg.IndexOf('"', start);
        return double.Parse(svg[start..end], System.Globalization.CultureInfo.InvariantCulture);
    }

    private static string Root()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("repo root");
    }
}
