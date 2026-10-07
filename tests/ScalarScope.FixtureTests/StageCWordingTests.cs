using FluentAssertions;
using ScalarScope.Services;
using Xunit;

namespace ScalarScope.FixtureTests;

/// <summary>
/// Stage C wording and workflow locks. A published sentence is the defect.
/// </summary>
public class StageCWordingTests
{
    private static string RepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("Could not find repo root");
    }

    private static string Read(string relative) =>
        File.ReadAllText(Path.Combine(RepoRoot(), relative));

    private static CanonicalDelta Delta(
        string id,
        string explanation = "",
        double delta = 0.5,
        int? steps = null,
        bool? failedA = null,
        bool? failedB = null) => new()
    {
        Id = id,
        Name = id,
        Explanation = explanation,
        Delta = delta,
        DeltaTcSteps = steps,
        FailedA = failedA,
        FailedB = failedB
    };

    [Fact]
    public void Describe_uses_the_detector_sentence_and_never_percent_as_steps()
    {
        var told = Delta(DeltaIds.ConvergenceTiming, explanation: "steady state not detected", delta: 0.5, steps: 4);
        DeltaCopyService.Describe(told, "A", "B").Should().Be("steady state not detected");

        var sooner = Delta(DeltaIds.ConvergenceTiming, steps: -3, delta: 0.5);
        var text = DeltaCopyService.Describe(sooner, "A", "B");
        text.Should().Be("A reached steady state 3 steps sooner than B.");
        text.Should().NotContain("50");

        DeltaCopyService.Describe(Delta(DeltaIds.ConvergenceTiming, steps: 2), "A", "B")
            .Should().Be("B reached steady state 2 steps sooner than A.");
        DeltaCopyService.Describe(Delta(DeltaIds.ConvergenceTiming, steps: 0), "A", "B")
            .Should().Be("A and B reached steady state together.");
    }

    [Fact]
    public void Both_failures_stay_both_failures()
    {
        var both = Delta(DeltaIds.FailurePresence, failedA: true, failedB: true);
        DeltaCopyService.Describe(both, "A", "B").Should().Be("Both A and B failed.");
        DeltaCopyService.BottomLine([both], "A", "B")
            .Should().Be("Both A and B failed. Neither approach completed successfully.");

        var onlyA = Delta(DeltaIds.FailurePresence, failedA: true, failedB: false);
        DeltaCopyService.Describe(onlyA, "A", "B").Should().Be("A failed. B did not.");
        DeltaCopyService.BottomLine([onlyA], "A", "B")
            .Should().Contain("B completed successfully while A failed");
    }

    [Fact]
    public void Why_heading_follows_status_and_highlight_uses_the_wire_token()
    {
        DeltaCopyService.WhyHeading(DeltaStatus.Present).Should().Be("Why this fired:");
        DeltaCopyService.WhyHeading(DeltaStatus.Suppressed).Should().Be("Why this is withheld:");
        DeltaCopyService.WhyHeading(DeltaStatus.Indeterminate).Should().Be("Why this is not settled:");

        DeltaIds.HighlightToken("failure").Should().Be(DeltaIds.FailurePresence);
        DeltaIds.HighlightToken("convergence").Should().Be(DeltaIds.ConvergenceTiming);
        DeltaIds.HighlightToken("dominance").Should().Be(DeltaIds.StructuralEmergence);
        DeltaIds.HighlightToken("alignment").Should().Be(DeltaIds.EvaluatorAlignment);
        DeltaIds.HighlightToken("oscillation").Should().Be(DeltaIds.StabilityOscillation);
        DeltaIds.HighlightToken("delta_f").Should().Be(DeltaIds.FailurePresence);
        DeltaIds.Canonical("failure").Should().Be("failure");
    }

    [Fact]
    public void Guide_and_home_do_not_call_the_band_a_confidence_interval()
    {
        var help = Read("src/ScalarScope/Views/HelpPage.xaml");
        help.Should().Contain("It is not a confidence interval.");
        help.Should().Contain("A missing milestone is not a stabilization time.");
        help.Should().Contain("An inference preset suppresses");
        help.Should().Contain("Loss stays loss");
        help.Should().Contain("preferences.json");
        help.Should().NotContain("Ctrl+O");
        help.Should().NotContain("settings.json");

        var home = Read("src/ScalarScope/Views/WelcomePage.xaml");
        home.Should().Contain("Open Compare");
        home.Should().Contain("not a confidence interval");
        home.Should().NotContain("Load Runs");

        var why = Read("src/ScalarScope/Views/Controls/DeltaWhyPanel.xaml.cs");
        why.Should().NotContain("delta_f");
        why.Should().NotContain("delta_tc");
        Read("src/ScalarScope/Views/Controls/DeltaWhyPanel.xaml")
            .Should().Contain("WhyHeading");
    }

    [Fact]
    public void Published_pages_call_the_hash_a_content_check()
    {
        const string claim = "A matching SHA-256 is a content check, not a signature.";
        Read("docs/index.md").Should().Contain(claim);
        Read("docs/index.md").Should().NotContain("cryptographic integrity");
        Read("README.md").Should().Contain(claim);
        Read("site/src/content/docs/handbook/bundles.md").Should().NotContain("results are verified");

        // The English sentence stays on the English page. Each translation
        // has to say the same thing in its own language. The forbidden words
        // are the ones that would turn the hash into a signature.
        foreach (var (name, claimInLanguage) in new (string Name, string Claim)[]
        {
            ("ja", "一致するSHA-256は、署名ではなく、コンテンツチェックです。"),
            ("zh", "匹配的 SHA-256 是内容检查，而不是签名。"),
            ("es", "Una coincidencia de SHA-256 es una verificación de contenido, no una firma."),
            ("fr", "Un hachage SHA-256 correspondant est une vérification du contenu, et non une signature."),
            ("hi", "मिलान SHA-256 एक सामग्री जांच है, हस्ताक्षर नहीं।"),
            ("it", "Un SHA-256 corrispondente è un controllo del contenuto, non una firma."),
            ("pt-BR", "Um SHA-256 correspondente é uma verificação de conteúdo, não uma assinatura."),
        })
        {
            var page = Read($"README.{name}.md");
            page.Should().Contain(claimInLanguage);
            page.Should().NotContain("cryptographically");
            page.Should().NotContain("criptográficamente");
            page.Should().NotContain("cryptographiquement");
            page.Should().NotContain("crittograficamente");
            page.Should().NotContain("criptograficamente");
            page.Should().NotContain("暗号学的");
            page.Should().NotContain("密码学");
            page.Should().NotContain("क्रिप्टोग्राफिक");
        }
    }

    [Fact]
    public void Persistence_and_install_pages_name_the_files_the_app_writes()
    {
        var data = Read("docs/DATA_PERSISTENCE.md");
        data.Should().Contain("preferences.json");
        data.Should().Contain("comparison-log.json");
        data.Should().Contain("does not write `settings.json`, `recent.json`, or `window.json`");
        data.Should().Contain("mcp-tool-shop.ScalarScope_yn6b8xqrexa5j");
        data.Should().NotContain("### settings.json");

        var install = Read("docs/INSTALL_RUNBOOK.md");
        install.Should().Contain("has no `--software-rendering` switch");
        install.Should().Contain("Developer Mode does not install this file");
        install.Should().NotContain("Help >");
        install.Should().Contain("will not install");
        install.Should().Contain("Settings > About");
        install.Should().Contain("Guide page");

        Read("STORE_LISTING.md").Should().NotContain("ΔTc should fire");
        Read("STORE_LISTING.md").Should().Contain("steady-state milestone");

        var beta = Read("docs/BETA_GUIDE.md");
        beta.Should().NotContain("Help >");
        beta.Should().NotContain("Self-signed");
        beta.Should().NotContain("Trajectory, Scalars, Geometry");
        beta.Should().Contain("Unsigned upload");
        beta.Should().Contain("Guide page");

        var process = Read("docs/RELEASE_PROCESS.md");
        process.Should().Contain("ScalarScope_3.1.0.0_x64.msix");
        process.Should().Contain("ScalarScope_3.1.0.0_Store.msixupload");
        process.Should().Contain("Partner Center signs");
        process.Should().NotContain("MSIX is signed");
        process.Should().Contain("Do not look for `ScalarScope-{version}-{arch}.msix`");

        var qa = Read("docs/RELEASE_QA_CHECKLIST.md");
        qa.Should().Contain("17763");
        qa.Should().NotContain("19041+");
        qa.Should().Contain("not part of this sign-off");
    }

    [Fact]
    public void Release_workflows_pin_the_pack_and_refuse_another_tag()
    {
        var release = Read(".github/workflows/release.yml");
        release.Should().Contain("refs/tags/v3.1.0.0");
        release.Should().Contain("--locked");
        release.Should().Contain("--remap-path-prefix=$env:GITHUB_WORKSPACE=.");
        release.Should().Contain("toolchain: 1.98.1");

        var pages = Read(".github/workflows/pages.yml");
        pages.Should().Contain("contents: read");
        pages.Should().Contain("pages: write");
        pages.Should().Contain("id-token: write");
        pages.Should().NotContain("contents: write");

        var build = Read(".github/workflows/build.yml");
        build.Should().Contain("pack-review:");
        build.Should().Contain("packaging/**");
        build.Should().Contain("rust/**");
        build.Should().Contain("--locked");
        build.Should().Contain("ScalarScope_3.1.0.0_x64.msix");
    }
}
