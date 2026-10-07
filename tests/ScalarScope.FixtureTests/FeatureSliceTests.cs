using FluentAssertions;
using Xunit;

namespace ScalarScope.FixtureTests;

public class FeatureSliceTests
{
    [Fact]
    public void Handbook_and_landing_teach_the_rust_review()
    {
        var review = Read("site/src/content/docs/handbook/rust-review.md");
        review.Should().Contain("cargo run --manifest-path rust/Cargo.toml");
        review.Should().Contain("Open path A");
        review.Should().Contain("Open path B");
        review.Should().Contain("Open bundle");
        review.Should().Contain("Save bundle");
        // 3.1.1 is the Store package; the pages must not say the Store still has the .NET app.
        review.Should().Contain("an update of listing `9P3HT1PHBKQK`");
        review.Should().NotContain("still the previous .NET");
        review.Should().Contain("run_history.json");
        review.Should().Contain("Loss stays loss");

        foreach (var relative in new[]
        {
            "site/src/content/docs/handbook/getting-started.md",
            "site/src/content/docs/handbook/beginners.md",
            "site/src/site-config.ts"
        })
        {
            var page = Read(relative);
            page.Should().Contain("cargo run --manifest-path rust/Cargo.toml");
            page.Should().NotContain("still the previous .NET");
        }

        var index = Read("site/src/content/docs/handbook/index.md");
        index.Should().Contain("Rust review");
        index.Should().Contain("an update of the same listing");
        index.Should().NotContain("still the previous .NET");
    }

    [Fact]
    public void Getting_started_has_one_example_of_each_file()
    {
        var page = Read("site/src/content/docs/handbook/getting-started.md");
        page.Should().Contain("step,latency_ms");
        page.Should().Contain("\"latency_ms\": 12.5");
        page.Should().Contain("ProfilerStep#1");
        page.Should().Contain("\"ph\": \"X\"");
        page.Should().Contain("\"dur\": 12500");
        page.Should().Contain("\"loss_history\": [1.2, 0.8]");
        page.Should().Contain("\"final_loss\": 0.8");

        var faq = Read("site/src/content/docs/handbook/beginners.md");
        faq.Should().Contain("latency CSV");
        faq.Should().Contain("benchmark JSON");
        faq.Should().Contain("Chrome profiler trace");
        faq.Should().Contain("run_history.json");
        faq.Should().Contain("/scalarscope/handbook/getting-started/");
        faq.Should().NotContain("TensorFlow-TRT log directories");
    }

    [Fact]
    public void Pack_review_keeps_the_store_files()
    {
        var build = Read(".github/workflows/build.yml");
        var packAt = build.IndexOf("pack-review:", StringComparison.Ordinal);
        packAt.Should().BeGreaterThan(0);
        var pack = build[packAt..];
        pack.Should().Contain("release/ScalarScope_3.1.1.0_x64.msix");
        pack.Should().Contain("release/ScalarScope_3.1.1.0_Store.msixupload");
        pack.Should().Contain("release/checksums.txt");
        pack.Should().Contain("if-no-files-found: error");
        pack.Should().Contain("name: scalarscope-store-3.1.1.0");
        pack.Should().NotContain("dotnet publish");

        var verify = Read("docs/VERIFY_RELEASE.md");
        verify.Should().Contain("./packaging/pack.ps1");
        verify.Should().Contain("release/ScalarScope_3.1.1.0_Store.msixupload");
        verify.Should().NotContain("dotnet publish src/ScalarScope/ScalarScope.csproj");
    }

    [Fact]
    public void Maui_project_is_not_the_store_upload()
    {
        var project = Read("src/ScalarScope/ScalarScope.csproj");
        project.Should().Contain("<WindowsPackageType>MSIX</WindowsPackageType>");
        project.Should().Contain("<ApplicationDisplayVersion>3.1.1</ApplicationDisplayVersion>");
        project.Should().Contain("<Version>3.1.1.0</Version>");
        project.Should().Contain("not the Partner Center upload");
        project.Should().Contain("RejectMauiAsStoreUpload");
        project.Should().Contain("Do not publish this MAUI app as that upload.");
        project.Should().NotContain("Windows packaging for Microsoft Store");
    }

    private static string Read(string relative)
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return File.ReadAllText(Path.Combine(dir ?? throw new InvalidOperationException("repo root"), relative));
    }
}
