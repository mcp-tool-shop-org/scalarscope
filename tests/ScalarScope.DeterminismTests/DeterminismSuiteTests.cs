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
        "RunIdNormalization"
    ];

    [Fact]
    public void RunAllTests_AllPassed()
    {
        var results = DeterminismTestSuite.RunAllTests();

        Assert.NotNull(results);
        Assert.True(results.TotalTests >= RequiredTests.Length, results.ToString());
        Assert.True(results.AllPassed, results.ToString());
        Assert.Equal(0, results.FailedTests);

        foreach (var name in RequiredTests)
        {
            Assert.Contains(results.Results, result => result.TestName == name && result.Passed);
        }
    }
}
