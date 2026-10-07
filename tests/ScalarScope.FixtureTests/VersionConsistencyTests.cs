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
        manifestVersion.Should().Be("3.1.0.0");
        properties.Element(ns + "PublisherDisplayName")!.Value.Should().Be("mcp-tool-shop");

        var csproj = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "ScalarScope.csproj"));
        var display = csproj.Descendants("ApplicationDisplayVersion").First().Value;
        var packaged = csproj.Descendants("Version").First().Value;
        display.Should().Be("3.1.0");
        packaged.Should().Be("3.1.0.0");
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
            .Should().BeGreaterThanOrEqualTo(31, "ApplicationVersion is the package revision and must stay above 3.0's 30");
    }

    [Fact]
    public void Pack_script_takes_its_version_from_the_manifest_it_packs()
    {
        var root = FindRepoRoot();
        XNamespace ns = "http://schemas.microsoft.com/appx/manifest/foundation/windows10";
        var packed = XDocument.Load(Path.Combine(root, "packaging", "AppxManifest.xml")).Root!.Element(ns + "Identity")!.Attribute("Version")!.Value;
        var maui = XDocument.Load(Path.Combine(root, "src", "ScalarScope", "Platforms", "Windows", "Package.appxmanifest")).Root!.Element(ns + "Identity")!.Attribute("Version")!.Value;
        packed.Should().Be(maui, "the Rust package and the MAUI manifest carry one Store identity version");

        var pack = File.ReadAllText(Path.Combine(root, "packaging", "pack.ps1"));
        pack.Should().Contain("$version = $source.Package.Identity.Version");
        pack.Should().Contain("\"ScalarScope_${version}_x64.msix\"");
        pack.Should().Contain("\"ScalarScope_${version}_Store.msixupload\"");
        pack.Should().NotMatchRegex(@"ScalarScope_\d+\.\d+\.\d+\.\d+_", "no file name is pinned to a version in the script");

        var cargo = File.ReadAllText(Path.Combine(root, "rust", "Cargo.toml"));
        cargo.Should().Contain($"version = \"{packed[..packed.LastIndexOf('.')]}\"", "the Rust crate's version is the package version without its revision");
    }

}
