using FluentAssertions;
using ScalarScope.Services.Connectors;
using Xunit;

namespace ScalarScope.FixtureTests;

public class TraceReviewTests
{
    [Fact]
    public async Task LatencyCsv_RoundTripsIntoTheAlignedWindow()
    {
        var directory = NewDirectory();
        var leftPath = Path.Combine(directory, "baseline.csv");
        var rightPath = Path.Combine(directory, "optimized.csv");
        await File.WriteAllTextAsync(leftPath, "step,latency_ms\n0,10\n1,10\n2,12\n3,11\n4,10\n5,10\n6,13\n7,10\n");
        await File.WriteAllTextAsync(rightPath, "step,latency_ms\n0,5\n1,5\n2,6\n3,5\n4,5\n5,5\n6,7\n7,5\n");

        var connector = new TensorFlowRTOfflineConnector();
        var left = await connector.ImportRuntimeAsync(leftPath);
        var right = await connector.ImportRuntimeAsync(rightPath);
        var comparison = new RunTraceComparer().Compare(left, right, ComparisonIntent.TfrtOptimization("baseline", "optimized"));
        var review = TraceReviewBuilder.Build(left, right, comparison, leftPath, rightPath);

        review.Signal.Should().Be("latency_ms");
        review.Unit.Should().Be("ms");
        review.SkippedLeft.Should().Be(0);
        review.LeftValues.Should().Equal(10d, 10d, 12d, 11d, 10d, 10d, 13d, 10d);
        review.RightValues.Should().Equal(5d, 5d, 6d, 5d, 5d, 5d, 7d, 5d);
        review.LeftP50.Should().Be(10);
        review.LeftP95.Should().Be(13);
        review.RightP50.Should().Be(5);
        review.RightP95.Should().Be(7);
        review.LeftDistribution.Should().HaveCount(8);
        review.LeftDistribution[^1].Probability.Should().Be(1);
        review.LeftDistribution.Select(point => point.Value).Should().BeInAscendingOrder();
        review.LeftDistribution.Select(point => point.Value).Should().Equal(review.LeftValues.Select(value => value!.Value).OrderBy(value => value));
    }

    [Fact]
    public async Task StabilizationClaim_IsWithheldWithoutASteadyStateMilestone()
    {
        var directory = NewDirectory();
        var leftPath = Path.Combine(directory, "short.csv");
        var rightPath = Path.Combine(directory, "longer.csv");
        await File.WriteAllTextAsync(leftPath, "step,latency_ms\n0,10\n1,10\n2,10\n3,10\n");
        await File.WriteAllTextAsync(rightPath, "step,latency_ms\n0,10\n1,10\n2,10\n3,10\n4,10\n5,10\n6,10\n7,10\n");

        var connector = new TensorFlowRTOfflineConnector();
        var left = await connector.ImportRuntimeAsync(leftPath);
        var right = await connector.ImportRuntimeAsync(rightPath);
        var comparison = new RunTraceComparer().Compare(left, right, ComparisonIntent.TfrtOptimization());
        comparison.Deltas.Should().Contain(delta => delta.DeltaType == "ΔTc" && delta.Fired);

        var review = TraceReviewBuilder.Build(left, right, comparison, leftPath, rightPath);
        review.FiredSymbols.Should().NotContain("ΔTc");
        review.Withheld.Should().Contain(line => line.Contains("withheld", StringComparison.OrdinalIgnoreCase));
        review.Verdict.Should().Contain("not a stabilization time");
    }

    [Fact]
    public void GeometryExport_IsNotATrace()
    {
        const string geometry = """{"schema_version":"1.0","trajectory":{"timesteps":[{"t":0,"state_2d":[1,2]}]}}""";
        const string benchmark = """{"results":[{"latency_ms":1.5}]}""";

        TraceOpen.IsGeometryRun(geometry).Should().BeTrue();
        TraceOpen.IsGeometryRun(benchmark).Should().BeFalse();
        TraceOpen.IsGeometryRun("step,latency_ms\n0,1\n").Should().BeFalse();
    }

    [Fact]
    public async Task ProfilerTrace_ImportsLatencyWithoutTheTraceJsonName()
    {
        var path = Path.Combine(NewDirectory(), "capture.json");
        await File.WriteAllTextAsync(path, """
            {"traceEvents":[
              {"name":"TensorRT inference","ph":"X","dur":2500,"ts":0},
              {"name":"inference","ph":"X","dur":4000,"ts":3000}
            ]}
            """);

        var trace = await new TensorFlowRTOfflineConnector().ImportRuntimeAsync(path);
        trace.Scalars.GetByName("latency_ms")!.Values.Should().Equal(2.5d, 4d);
    }

    [Fact]
    public async Task BenchmarkJson_ImportsLatencyUnderAnyFileName()
    {
        var path = Path.Combine(NewDirectory(), "after.json");
        await File.WriteAllTextAsync(path, """{"results":[{"latency_ms":1.25},{"iteration":1,"latency_ms":1.75}]}""");

        var trace = await new TensorFlowRTOfflineConnector().ImportRuntimeAsync(path);
        trace.Scalars.GetByName("latency_ms")!.Values.Should().Equal(1.25d, 1.75d);
    }

    [Fact]
    public async Task CsvWithoutNumericLatency_IsRejected()
    {
        var path = Path.Combine(NewDirectory(), "empty.csv");
        await File.WriteAllTextAsync(path, "step,latency_ms\n0,fast\n");

        var act = async () => await new TensorFlowRTOfflineConnector().ImportRuntimeAsync(path);
        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("*TFRT_NO_LATENCY_SIGNAL*");
    }

    private static string NewDirectory()
    {
        var directory = Path.Combine(Path.GetTempPath(), "scalarscope-trace-" + Guid.NewGuid().ToString("n"));
        Directory.CreateDirectory(directory);
        return directory;
    }
}
