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

    [Fact]
    public void PackageIdentity_MatchesPartnerCenter()
    {
        var root = FindRepoRoot();
        XNamespace ns = "http://schemas.microsoft.com/appx/manifest/foundation/windows10";
        var manifest = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "Platforms", "Windows", "Package.appxmanifest"));
        var identity = manifest.Root!.Element(ns + "Identity")!;
        var properties = manifest.Root.Element(ns + "Properties")!;

        identity.Attribute("Name")!.Value.Should().Be("mcp-tool-shop.ScalarScope");
        identity.Attribute("Publisher")!.Value.Should().Be("CN=5305D976-6952-4F00-9C21-3A5DB090359F");
        identity.Attribute("Version")!.Value.Should().Be("3.0.0.0");
        properties.Element(ns + "PublisherDisplayName")!.Value.Should().Be("mcp-tool-shop");

        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        csproj.Descendants("ApplicationDisplayVersion").First().Value.Should().Be("3.0.0");
        csproj.Descendants("Version").First().Value.Should().Be("3.0.0.0");
        int.Parse(csproj.Descendants("ApplicationVersion").First().Value).Should().BeGreaterThanOrEqualTo(30);
    }
}
