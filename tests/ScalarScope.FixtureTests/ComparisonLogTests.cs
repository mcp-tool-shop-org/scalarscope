using FluentAssertions;
using ScalarScope.Services;
using Xunit;

namespace ScalarScope.FixtureTests;

public class ComparisonLogTests
{
    [Fact]
    public void SymbolsFor_KeepsPresentDeltasInCanonicalOrder()
    {
        var symbols = ComparisonLog.SymbolsFor(
        [
            Delta("StabilityOscillation", DeltaStatus.Present),
            Delta("FailurePresence", DeltaStatus.Present),
            Delta("ConvergenceTiming", DeltaStatus.Suppressed),
            Delta("EvaluatorAlignment", DeltaStatus.Indeterminate),
            Delta("delta_td", DeltaStatus.Present)
        ]);

        symbols.Should().Equal("ΔF", "ΔTd", "ΔO");
    }

    [Fact]
    public void Record_UpdatesTheSameTwoRunsInsteadOfStacking()
    {
        var directory = NewDirectory();

        var first = ComparisonLog.Record(Pair("left.json", "right.json", ["ΔF"], "ByStep"), directory);
        var second = ComparisonLog.Record(Pair("left.json", "right.json", ["ΔF", "ΔTc"], "ByConvergence"), directory);
        var other = ComparisonLog.Record(Pair("other.json", "right.json", ["ΔO"], "ByStep"), directory);

        var entries = ComparisonLog.Read(directory);
        entries.Should().HaveCount(2);
        entries[0].Id.Should().Be(other.Id);
        entries[1].Id.Should().Be(first.Id);
        second.Id.Should().Be(first.Id);
        entries[1].DeltasFired.Should().Equal("ΔF", "ΔTc");
        entries[1].Alignment.Should().Be("ByConvergence");
        entries[1].BundlePath.Should().BeNull();
    }

    [Fact]
    public void Record_KeepsAnExistingBundleHashWhenTheRunsAreRecordedAgain()
    {
        var directory = NewDirectory();
        var first = ComparisonLog.Record(Pair("left.json", "right.json", ["ΔF"], "ByStep"), directory);
        ComparisonLog.AttachBundle(@"D:\reviews\one.scbundle", "abcdef0123456789", directory);

        var again = ComparisonLog.Record(Pair("left.json", "right.json", ["ΔTc"], "ByStep"), directory);

        again.Id.Should().Be(first.Id);
        again.BundlePath.Should().Be(@"D:\reviews\one.scbundle");
        again.BundleHash.Should().Be("abcdef0123456789");
        again.DeltasFired.Should().Equal("ΔTc");
        ComparisonLog.Read(directory).Should().ContainSingle();
    }

    [Fact]
    public void AttachBundle_StampsTheNewestLiveReview()
    {
        var directory = NewDirectory();
        ComparisonLog.Record(Pair("left.json", "right.json", ["ΔO"], "ByStep"), directory);

        var stamped = ComparisonLog.AttachBundle(@"D:\reviews\one.scbundle", "0123456789abcdef", directory);

        stamped.Kind.Should().Be("compare");
        stamped.BundleHash.Should().Be("0123456789abcdef");
        stamped.Subtitle.Should().Be("ΔO · 01234567");
    }

    [Fact]
    public void Record_CapsTheFileAtFortyEntries()
    {
        var directory = NewDirectory();
        for (var i = 0; i < ComparisonLog.MaxEntries + 1; i++)
            ComparisonLog.Record(Pair($"l{i}.json", $"r{i}.json", ["ΔF"], "ByStep"), directory);

        var entries = ComparisonLog.Read(directory);
        entries.Should().HaveCount(ComparisonLog.MaxEntries);
        entries[0].LeftPath.Should().Be($"l{ComparisonLog.MaxEntries}.json");
        entries.Should().NotContain(entry => entry.LeftPath == "l0.json");
    }

    [Fact]
    public void Read_ReturnsEmptyWhenTheFileIsCorrupt()
    {
        var directory = NewDirectory();
        File.WriteAllText(ComparisonLog.FilePathFor(directory), "{not json");

        ComparisonLog.Read(directory).Should().BeEmpty();
    }

    [Fact]
    public void Clear_RemovesTheLog()
    {
        var directory = NewDirectory();
        ComparisonLog.Record(Pair("left.json", "right.json", [], "ByStep"), directory);

        ComparisonLog.Clear(directory);

        ComparisonLog.Read(directory).Should().BeEmpty();
        File.Exists(ComparisonLog.FilePathFor(directory)).Should().BeFalse();
    }

    private static ComparisonLogEntry Pair(string left, string right, IEnumerable<string> deltas, string alignment)
    {
        return new ComparisonLogEntry
        {
            Kind = "compare",
            LeftName = Path.GetFileNameWithoutExtension(left),
            RightName = Path.GetFileNameWithoutExtension(right),
            LeftPath = left,
            RightPath = right,
            Alignment = alignment,
            DeltasFired = deltas.ToList()
        };
    }

    private static CanonicalDelta Delta(string id, DeltaStatus status)
    {
        return new CanonicalDelta
        {
            Id = id,
            Name = id,
            Explanation = id,
            Status = status
        };
    }

    private static string NewDirectory()
    {
        var directory = Path.Combine(Path.GetTempPath(), "scalarscope-log-" + Guid.NewGuid().ToString("n"));
        Directory.CreateDirectory(directory);
        return directory;
    }
}
