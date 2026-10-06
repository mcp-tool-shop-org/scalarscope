using System.Text.Json;
using FluentAssertions;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Maui.ApplicationModel;
using ScalarScope.Services;
using ScalarScope.SoakTests;
using ScalarScope.ViewModels;
using ScalarScope.Views.Controls;
using Xunit;

namespace ScalarScope.FixtureTests;

public class StageCShellTests
{
    [Fact]
    public void Saved_playback_export_and_accessibility_are_what_the_shell_reads()
    {
        var directory = Scratch("prefs");
        UserPreferencesService.UseDirectory(directory);
        try
        {
            UserPreferencesService.SetDefaultPlaybackSpeed(2f);
            UserPreferencesService.SetAutoPlayOnLoad(true);
            UserPreferencesService.SetDefaultExportPath(directory);
            UserPreferencesService.SetDefaultExportResolution(800, 600);
            UserPreferencesService.SetColorVisionMode(2);
            UserPreferencesService.SetHighContrastMode(true);
            UserPreferencesService.SetTextScale(1.5f);
            UserPreferencesService.SetScreenReaderMode(true);
            UserPreferencesService.SetLargePointer(true);
            UserPreferencesService.SetAnnotationDensity(AnnotationDensity.Full);

            using var player = new TrajectoryPlayerViewModel();
            player.ApplySavedPlayback();
            player.Speed.Should().Be(2);
            player.IsPlaying.Should().BeTrue();

            ExportPreferences.FolderOr("fallback").Should().Be(directory);
            ExportPreferences.SizeOr(1920, 1080).Should().Be((800, 600));

            ShellStartup.ApplyAccessibility();
            var settings = AccessibilityService.Instance.Settings;
            settings.ColorPaletteMode.Should().Be(ColorPaletteMode.Protanopia);
            settings.HighContrastEnabled.Should().BeTrue();
            settings.TextScale.Should().Be(1.5f);
            settings.ScreenReaderEnabled.Should().BeTrue();
            settings.LargePointer.Should().BeTrue();

            UserPreferencesService.GetAnnotationDensity().Should().Be(AnnotationDensity.Full);

            ShellStartup.ShouldOpenSavedSession(true, directory).Should().BeTrue();
            ShellStartup.ShouldOpenSavedSession(false, directory).Should().BeFalse();
            ShellStartup.ShouldOpenSavedSession(true, "  ").Should().BeFalse();
        }
        finally
        {
            UserPreferencesService.UseDirectory(null);
            AccessibilityService.Instance.Settings = new AccessibilitySettings();
            try { Directory.Delete(directory, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public void Unreadable_preferences_stay_on_disk_and_settings_says_so()
    {
        var directory = Scratch("unread");
        var decoy = Path.Combine(directory, "preferences.json");
        Directory.CreateDirectory(decoy);
        UserPreferencesService.UseDirectory(directory);
        try
        {
            var vm = new SettingsViewModel();
            vm.StorageNotice.Should().Be("Saved settings could not be read. The preferences file was left unchanged.");
            vm.HasStorageNotice.Should().BeTrue();

            vm.ColorVisionIndex = 40;
            vm.ColorVisionIndex.Should().Be(0);
            vm.TextScale = 0.1f;
            vm.TextScale.Should().Be(0.75f);

            UserPreferencesService.SetTheme(AppTheme.Dark);
            Directory.Exists(decoy).Should().BeTrue();
            File.Exists(decoy).Should().BeFalse();
            UserPreferencesService.StorageNotice.Should().Be(
                "Saved settings could not be read. The preferences file was left unchanged.");

            var page = File.ReadAllText(Path.Combine(RepoRoot(), "src/ScalarScope/Views/SettingsPage.xaml"));
            page.Should().Contain("StorageNotice");
        }
        finally
        {
            UserPreferencesService.UseDirectory(null);
            try { Directory.Delete(directory, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public void Clear_log_keeps_the_list_when_the_file_cannot_be_deleted()
    {
        var directory = Scratch("log");
        ComparisonLog.UseDirectory(directory);
        var path = ComparisonLog.FilePathFor(directory);
        try
        {
            File.WriteAllText(path, "[]");
            File.SetAttributes(path, FileAttributes.ReadOnly);
            var vm = new WelcomeViewModel();
            vm.ComparisonLogEntries =
            [
                new ComparisonLogEntry { LeftName = "kept", RightName = "run" }
            ];
            vm.HasComparisonLog = true;

            vm.ClearLogCommand.Execute(null);

            vm.ComparisonLogEntries.Should().ContainSingle(entry => entry.LeftName == "kept");
            vm.HasComparisonLog.Should().BeTrue();
            vm.ClearNotice.Should().NotBeNullOrWhiteSpace();
            vm.HasClearNotice.Should().BeTrue();
            File.Exists(path).Should().BeTrue();

            var page = File.ReadAllText(Path.Combine(RepoRoot(), "src/ScalarScope/Views/WelcomePage.xaml"));
            page.Should().Contain("ClearNotice");
        }
        finally
        {
            ComparisonLog.UseDirectory(null);
            if (File.Exists(path))
                File.SetAttributes(path, FileAttributes.Normal);
            try { Directory.Delete(directory, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public void Playback_theme_and_annotation_soak_drive_the_shell()
    {
        var playback = SoakTestRunner.ExercisePlayback(1000);
        playback.Passed.Should().BeTrue(playback.Message);
        playback.Message.Should().Contain("TrajectoryPlayerViewModel");
        playback.Message.Should().NotContain("PlaybackController");

        var themeDir = Scratch("theme");
        var noteDir = Scratch("notes");
        try
        {
            var theme = SoakTestRunner.ExerciseThemeToggle(themeDir);
            theme.Passed.Should().BeTrue(theme.Message);

            var locked = SoakTestRunner.ExerciseThemeToggle(themeDir, lockFileBeforeRestore: true);
            locked.Passed.Should().BeFalse();
            locked.Message.Should().Contain("not stored");

            var notes = SoakTestRunner.ExerciseAnnotationToggle(noteDir);
            notes.Passed.Should().BeTrue(notes.Message);
            notes.Message.Should().Contain("demo_start");
        }
        finally
        {
            UserPreferencesService.UseDirectory(null);
            DemoService.EndDemo();
            DemoAnnotationService.ResetForNewDemo();
            ClearReadonly(themeDir);
            try { Directory.Delete(themeDir, true); } catch { /* throwaway */ }
            try { Directory.Delete(noteDir, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public async Task Soak_rejects_a_short_duration_and_a_cancelled_run_does_not_pass()
    {
        var zero = SoakCommand.Parse(["--duration", "0"]);
        zero.Ok.Should().BeFalse();
        zero.Error.Should().Contain("at least 1 minute");
        zero.Error.Should().Contain("Usage:");

        SoakCommand.Parse(["--duration", "-4"]).Ok.Should().BeFalse();
        SoakCommand.Parse(["--nope"]).Ok.Should().BeFalse();
        SoakCommand.Parse(["--quick"]).DurationMinutes.Should().Be(5);
        SoakCommand.Parse([]).DurationMinutes.Should().Be(120);
        SoakCommand.Parse(["--help"]).ShowHelp.Should().BeTrue();

        var runner = new SoakTestRunner(
            NullLogger.Instance,
            new SoakTestConfig { DurationMinutes = 1, PauseBetweenIterationsSeconds = 30 });
        using var cts = new CancellationTokenSource();
        cts.Cancel();
        var report = await runner.RunAsync(cts.Token);
        report.TotalIterations.Should().Be(0);
        report.TotalTests.Should().Be(0);
        report.OverallPassed.Should().BeFalse();

        var program = File.ReadAllText(Path.Combine(RepoRoot(), "tests/ScalarScope.SoakTests/Program.cs"));
        program.Should().Contain("SoakCommand.Parse");
        program.Should().Contain("CancelKeyPress");
        program.Should().Contain("RunAsync(stop.Token)");
    }

    [Fact]
    public void Gate_D_is_required_before_the_lock_banner()
    {
        var blocked = GateReport(gateD: false);
        blocked.Locked = true;
        Phase32ValidationRunner.CanLock(blocked).Should().BeFalse();
        blocked.ToSummaryTable().Should().Contain("NOT LOCKED");
        blocked.AllGatesPassed.Should().BeFalse();

        var open = GateReport(gateD: true);
        Phase32ValidationRunner.CanLock(open).Should().BeTrue();
    }

    [Fact]
    public void Demo_failure_names_the_sample_and_the_pages_show_it()
    {
        var sentence = DemoService.DescribeLoadFailure(
            DemoService.PathAFileName,
            new FileNotFoundException("missing sample"));
        sentence.Should().Contain(DemoService.PathAFileName);
        sentence.Should().Contain("missing sample");
        sentence.Should().Contain("could not be loaded");
        sentence.Should().NotContain("Failed to load demo data");
        sentence.Should().NotContain("Please try again");

        foreach (var relative in new[]
        {
            "src/ScalarScope/Views/HelpPage.xaml.cs",
            "src/ScalarScope/Views/OverviewPage.xaml.cs",
            "src/ScalarScope/Views/ComparisonPage.xaml.cs"
        })
        {
            File.ReadAllText(Path.Combine(RepoRoot(), relative)).Should().Contain("DemoService.LastFailure");
        }
    }

    [Fact]
    public void Mapped_errors_name_a_real_file_and_drop_the_phantom_controls()
    {
        ErrorStateMapping.MapException(new UnauthorizedAccessException("no")).Code.Should().Be("FILE_ACCESS_DENIED");
        ErrorStateMapping.MapException(new DirectoryNotFoundException("gone")).Code.Should().Be("FILE_NOT_FOUND");
        ErrorStateMapping.MapException(new JsonException("bad")).Code.Should().Be("FILE_FORMAT_INVALID");
        ErrorStateMapping.MapException(new TimeoutException("slow")).Code.Should().Be("COMPUTATION_TIMEOUT");
        ErrorStateMapping.MapException(new TimeoutException("slow"), computingDeltas: true).Code
            .Should().Be("DELTA_COMPUTATION_TIMEOUT");
        ErrorStateMapping.MapException(new IOException("disk full")).Code.Should().Be("EXPORT_WRITE_FAILED");

        var unexpected = ErrorStateMapping.MapException(new InvalidOperationException("nope"));
        unexpected.Code.Should().Be("UNEXPECTED_ERROR");
        unexpected.UserExplanation.Should().NotContain("recorded");

        Joined("FILE_ACCESS_DENIED").Should().NotContain("administrator");
        Joined("FILE_FORMAT_INVALID").Should().Contain(".json").And.Contain(".csv");
        Joined("FILE_FORMAT_INVALID").Should().NotContain("TRAJECTORY_FORMAT");
        Joined("ALIGNMENT_FAILED").Should().NotContain("Force Alignment");
        Joined("COMPUTATION_TIMEOUT").Should().NotContain("Quick Compare").And.NotContain("60");
        Joined("DELTA_COMPUTATION_TIMEOUT").Should().NotContain("Quick Compare");
        Joined("UNEXPECTED_ERROR").Should().NotContain("recorded").And.NotContain("error log");

        var folder = Scratch("import");
        var file = Path.Combine(folder, "notes.txt");
        try
        {
            File.WriteAllText(file, "nope");
            var result = ImportSchemaService.Validate(file);
            var explanation = result.Errors[0].Explanation;
            explanation.Should().NotBeNull();
            var text = explanation!.RootCause + " " + string.Join(" ", explanation.TroubleshootingSteps);
            text.Should().Contain("geometry run");
            text.Should().Contain(".json");
            text.Should().Contain("latency");
            text.Should().Contain("training history");
            text.Should().NotContain("TRAJECTORY_FORMAT");
        }
        finally
        {
            try { Directory.Delete(folder, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public void Hash_copy_failure_says_the_bundle_was_saved()
    {
        var notice = BundleExportPanel.HashCopyNotice(@"E:\reviews\one.scbundle", new InvalidOperationException("clipboard"));
        notice.Should().Contain(@"E:\reviews\one.scbundle");
        notice.Should().Contain("was saved");
        notice.Should().Contain("was not copied");
        notice.Should().Contain("clipboard");
    }

    [Fact]
    public async Task Boundary_log_is_in_the_support_bundle()
    {
        var directory = Scratch("boundary");
        var logPath = Path.Combine(directory, "nested", "error_log.txt");
        var bundlePath = Path.Combine(directory, "bundle.txt");
        try
        {
            Exception sample;
            try
            {
                throw new InvalidOperationException("boundary-marker");
            }
            catch (Exception ex)
            {
                sample = ex;
            }

            var sink = ErrorLoggingService.CreateForDirectory(directory);
            ErrorBoundary.RecordBoundaryFailure(sample, "wave3", sink, logPath);

            var recent = sink.GetRecentErrors(5);
            recent.Should().Contain(entry =>
                entry.ExceptionMessage != null && entry.ExceptionMessage.Contains("boundary-marker"));
            recent.Should().Contain(entry => !string.IsNullOrWhiteSpace(entry.StackTrace));
            var written = await File.ReadAllTextAsync(logPath);
            written.Should().Contain("boundary-marker");
            written.Should().Contain("InvalidOperationException");

            await CrashReportingService.GenerateSupportBundleAsync(bundlePath, logPath);
            var bundle = await File.ReadAllTextAsync(bundlePath);
            bundle.Should().Contain("error_log.txt");
            bundle.Should().Contain("boundary-marker");
        }
        finally
        {
            try { Directory.Delete(directory, true); } catch { /* throwaway */ }
        }
    }

    [Fact]
    public void Handbook_says_which_shell_applies_accessibility()
    {
        foreach (var relative in new[]
        {
            "site/src/content/docs/handbook/getting-started.md",
            "site/src/content/docs/handbook/beginners.md"
        })
        {
            var page = File.ReadAllText(Path.Combine(RepoRoot(), relative));
            page.Should().Contain("Rust review");
            page.Should().Contain("does not offer screen reader");
            page.Should().Contain("larger pointer");
            page.Should().Contain("leaves that file unchanged");
        }
    }

    private static string Joined(string code)
    {
        var state = ErrorStateMapping.GetByCode(code);
        state.Should().NotBeNull();
        var explained = ErrorExplanationService.Explain(state!);
        return explained.Summary + " " + explained.RootCause + " " + string.Join(" ", explained.TroubleshootingSteps);
    }

    private static Phase32ValidationReport GateReport(bool gateD)
    {
        return new Phase32ValidationReport
        {
            PairResults = [new PairValidationResult()],
            SuiteGates = new SuiteGatesResult
            {
                GateA = true,
                GateB = true,
                GateC = true,
                GateD = gateD
            },
            DeltaImplementationStatus = new DeltaImplementationStatus
            {
                DeltaA = new SoakTests.DeltaStatus { Implemented = true },
                DeltaTd = new SoakTests.DeltaStatus { Implemented = true },
                DeltaTc = new SoakTests.DeltaStatus { Implemented = true },
                DeltaO = new SoakTests.DeltaStatus { Implemented = true },
                DeltaF = new SoakTests.DeltaStatus { Implemented = true }
            },
            DeltaFVerification = new DeltaFVerificationResult
            {
                Checks = [new VerificationCheck { Passed = true }]
            }
        };
    }

    private static string Scratch(string name)
    {
        var directory = Path.Combine(Path.GetTempPath(), "ss-" + name + "-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        return directory;
    }

    private static void ClearReadonly(string directory)
    {
        if (!Directory.Exists(directory))
            return;
        foreach (var file in Directory.GetFiles(directory, "*", SearchOption.AllDirectories))
            File.SetAttributes(file, FileAttributes.Normal);
    }

    private static string RepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("Could not find repo root");
    }
}
