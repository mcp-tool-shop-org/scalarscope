using System.Xml.Linq;
using FluentAssertions;
using Xunit;

namespace ScalarScope.FixtureTests;

public class VersionConsistencyTests
{
    private static string FindRepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("Could not find repo root");
    }

    [Fact]
    public void VortexKit_VersionIsSemver()
    {
        var root = FindRepoRoot();
        var csproj = XDocument.Load(Path.Combine(root, "src", "VortexKit", "VortexKit.csproj"));
        var version = csproj.Descendants("Version").First().Value;
        version.Should().MatchRegex(@"^\d+\.\d+\.\d+");
    }

    [Fact]
    public void ScalarScope_VersionIsSemver()
    {
        var root = FindRepoRoot();
        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        var version = csproj.Descendants("ApplicationDisplayVersion").First().Value;
        version.Should().MatchRegex(@"^\d+\.\d+\.\d+");
    }

    [Fact]
    public void ScalarScope_VersionAtLeast1()
    {
        var root = FindRepoRoot();
        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        var version = csproj.Descendants("ApplicationDisplayVersion").First().Value;
        var major = int.Parse(version.Split('.')[0]);
        major.Should().BeGreaterThanOrEqualTo(1);
    }
}
