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
        version.Should().MatchRegex(@"^\d+\.\d+\.\d+$");
    }

    [Fact]
    public void ScalarScope_VersionAtLeast1()
    {
        var root = FindRepoRoot();
        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        var version = csproj.Descendants("ApplicationDisplayVersion").First().Value;
        var parts = version.Split('.').Select(int.Parse).ToArray();
        parts.Should().HaveCountGreaterOrEqualTo(3);
        parts[0].Should().BeGreaterThanOrEqualTo(3, "display version must not fall behind 3.0.0");
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
        var manifestVersion = identity.Attribute("Version")!.Value;
        manifestVersion.Should().MatchRegex(@"^\d+\.\d+\.\d+\.\d+$");
        manifestVersion.Should().Be("3.0.0.0");
        properties.Element(ns + "PublisherDisplayName")!.Value.Should().Be("mcp-tool-shop");

        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        var display = csproj.Descendants("ApplicationDisplayVersion").First().Value;
        var packaged = csproj.Descendants("Version").First().Value;
        display.Should().Be("3.0.0");
        packaged.Should().Be("3.0.0.0");
        packaged.Should().Be(manifestVersion);
        csproj.Descendants("ApplicationId").First().Value.Should().Be("org.mcptoolshop.scalarscope");

        var displayParts = display.Split('.').Select(int.Parse).ToArray();
        var manifestParts = manifestVersion.Split('.').Select(int.Parse).ToArray();
        manifestParts.Should().HaveCount(4);
        var behindDisplay = manifestParts[0] < displayParts[0]
            || (manifestParts[0] == displayParts[0] && manifestParts[1] < displayParts[1])
            || (manifestParts[0] == displayParts[0] && manifestParts[1] == displayParts[1] && manifestParts[2] < displayParts[2]);
        behindDisplay.Should().BeFalse("the packaged four-part version must not sit behind ApplicationDisplayVersion");

        // ApplicationVersion is the MSIX revision counter, not the fourth Identity component.
        int.Parse(csproj.Descendants("ApplicationVersion").First().Value)
            .Should().BeGreaterThanOrEqualTo(30, "ApplicationVersion is the package revision and must stay above the 2.x revision");
    }
}
