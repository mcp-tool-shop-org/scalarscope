using ScalarScope.Services;
using Xunit;

namespace ScalarScope.DeterminismTests;

public class DeterminismSuiteTests
{
    private static readonly string[] RequiredTests =
    [
        "InputNormalization",
        "FingerprintDeterminism",
        "DeltaHashDeterminism",
        "DoubleNormalization",
        "RunIdNormalization",
        "InstabilityAlignmentExport",
        "SingleStepAlignment",
        "EnumSchemaRoundTrip",
        "FiveDeltaIdRoundTrip"
    ];

    [Fact]
    public void RunAllTests_AllPassed()
    {
        var results = DeterminismTestSuite.RunAllTests();

        Assert.NotNull(results);
        Assert.Equal(RequiredTests.Length, results.TotalTests);
        Assert.True(results.AllPassed, results.ToString());
        Assert.Equal(0, results.FailedTests);

        var names = results.Results.Select(result => result.TestName).OrderBy(name => name, StringComparer.Ordinal);
        Assert.Equal(RequiredTests.OrderBy(name => name, StringComparer.Ordinal), names);
        Assert.All(results.Results, result => Assert.True(result.Passed, result.TestName));
    }
}
