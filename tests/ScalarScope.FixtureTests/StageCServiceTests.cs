using System.Net;
using FluentAssertions;
using ScalarScope.Models;
using ScalarScope.Services;
using ScalarScope.Services.Integrations;
using ScalarScope.ViewModels;
using VkExport = VortexKit.Core.ExportService;
using VkOptions = VortexKit.Core.ExportOptions;
using VkPlayback = VortexKit.Core.PlaybackController;
using Xunit;

namespace ScalarScope.FixtureTests;

public class StageCServiceTests
{
    [Fact]
    public void Lyapunov_refuses_a_clock_that_is_not_a_positive_step()
    {
        var service = new AnalysisService();
        var bad = Run(25, i => i == 0 ? 0 : 0);
        var refused = service.EstimateLyapunovExponent(bad);
        refused.IsValid.Should().BeFalse();
        refused.ErrorMessage.Should().Contain("positive time step");
        refused.Classification.Should().NotBe("Chaotic");
        double.IsFinite(refused.AverageExponent).Should().BeTrue();

        var negative = Run(25, i => i == 0 ? 1 : 0);
        service.EstimateLyapunovExponent(negative).IsValid.Should().BeFalse();

        var spaced = service.EstimateLyapunovExponent(Run(25, i => i * 0.1));
        spaced.IsValid.Should().BeTrue();
        double.IsFinite(spaced.AverageExponent).Should().BeTrue();
        var again = service.EstimateLyapunovExponent(Run(25, i => i * 0.1));
        again.AverageExponent.Should().Be(spaced.AverageExponent);

        var over = service.EstimateLyapunovExponent(Run(AnalysisService.OverlayScanCap + 1, i => i * 0.1));
        over.IsValid.Should().BeFalse();
        over.AverageExponent.Should().Be(0);
        over.ErrorMessage.Should().Contain("overlay scan");
        over.Classification.Should().Be("Unknown");

        var eigenvalues = service.GetEigenvalueTimeline(Run(AnalysisService.OverlayScanCap + 1, i => i * 0.1));
        eigenvalues.Points.Should().BeEmpty();
        eigenvalues.UnavailableReason.Should().Contain("overlay scan");
    }

    [Fact]
    public void Log_rotation_opens_a_new_file_and_caps_memory()
    {
        var dir = Path.Combine(Path.GetTempPath(), "ss-log-" + Guid.NewGuid().ToString("N"));
        var log = ErrorLoggingService.CreateForDirectory(dir);
        log.MaxLogFileSize = 50;
        log.MaxSessionEntries = 3;
        for (var i = 0; i < 8; i++)
            log.Log(ErrorSeverity.Warning, new string('x', 40), "rotation");

        log.GetLogFiles().Count.Should().BeGreaterThan(1);
        log.GetRecentErrors(20).Count.Should().BeLessOrEqualTo(3);
        new FileInfo(log.GetCurrentLogPath()).Length.Should().BeLessOrEqualTo(200);
    }

    [Fact]
    public async Task Support_bundle_reads_a_log_tail()
    {
        var path = Path.Combine(Path.GetTempPath(), "ss-tail-" + Guid.NewGuid().ToString("N") + ".log");
        await File.WriteAllTextAsync(path, new string('A', 5000) + new string('B', 5000));
        var tail = await CrashReportingService.ReadLogTailAsync(path, 1000);
        tail.Should().NotContain("A");
        tail.Should().Contain("B");
        tail.Length.Should().BeLessOrEqualTo(1000);
        File.Delete(path);
    }

    [Fact]
    public void Session_version_is_the_assembly_version()
    {
        AppSession.Version.Should().Be(VersionInfo.Version);
        File.ReadAllText(Path.Combine(FindRepoRoot(), "src/ScalarScope/Services/ErrorLoggingService.cs"))
            .Should().NotContain("=> \"1.5.0\"");
    }

    [Fact]
    public async Task TensorBoard_does_not_invent_steps_and_a_token_stays_on_its_request()
    {
        var dir = Path.Combine(Path.GetTempPath(), "ss-tb-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        await File.WriteAllBytesAsync(Path.Combine(dir, "events.out.tfevents.1"), [1, 2, 3]);
        var imported = await new IntegrationService().ImportFromTensorBoardAsync(new TensorBoardConfig { LogDir = dir });
        imported.IsSuccess.Should().BeFalse();
        imported.Message.Should().Contain("not parsed");
        (imported.Data?.PointCount ?? 0).Should().Be(0);

        var seen = new List<string?>();
        var handler = new RecordingHandler(seen, metricsFail: true);
        var service = new IntegrationService(new HttpClient(handler));
        var wandb = await service.ImportFromWandBAsync(new WandBConfig
        {
            ApiKey = "wandb-secret",
            Entity = "e",
            Project = "p",
            RunId = "r"
        });
        wandb.IsSuccess.Should().BeFalse();
        seen.Should().Contain("wandb-secret");

        var mlflow = await service.ImportFromMLflowAsync(new MLflowConfig
        {
            TrackingUri = "http://mlflow.example",
            RunId = "run-1",
            MetricKeys = ["loss", "accuracy"]
        });
        mlflow.IsSuccess.Should().BeFalse();
        mlflow.Message.Should().Contain("loss");
        seen.Where(header => header == "wandb-secret").Should().HaveCount(1);
        seen.Last().Should().BeNull();
    }

    [Fact]
    public void Sequence_export_refuses_an_unbounded_frame_count_and_marks_a_failed_write()
    {
        VkExport.FrameCount(30, 1).Should().Be(30);
        var act = () => VkExport.FrameCount(30, 20);
        act.Should().Throw<ArgumentOutOfRangeException>();
    }

    [Fact]
    public async Task A_failed_sequence_leaves_a_partial_marker()
    {
        var dir = Path.Combine(Path.GetTempPath(), "ss-seq-" + Guid.NewGuid().ToString("N"));
        var service = new VkExport();
        Func<Task> act = () => service.ExportSequenceAsync(
            (_, _, _) => throw new InvalidOperationException("render"),
            dir,
            new VkOptions { Fps = 1, Duration = 1, Width = 8, Height = 8 });
        await act.Should().ThrowAsync<InvalidOperationException>();
        (await File.ReadAllTextAsync(Path.Combine(dir, "sequence_partial.txt")))
            .Should().Contain("Frames written: 0 of 1");
    }

    [Fact]
    public void Stopped_playback_does_not_move_and_a_dropped_tick_stops_the_player()
    {
        var player = new TrajectoryPlayerViewModel();
        player.Time = 0.4;
        player.IsPlaying = false;
        player.AdvanceOneTick();
        player.Time.Should().Be(0.4);

        player.IsPlaying = true;
        player.Duration = 0;
        player.AdvanceOneTick();
        player.Time.Should().Be(0.4);

        player.Duration = 10;
        player.Speed = 1;
        player.TimeChanged += () => throw new InvalidOperationException("listener");
        var moving = () => player.AdvanceOneTick();
        moving.Should().NotThrow();
        player.Dispose();

        var vortex = new VkPlayback { IsPlaying = true };
        string? fault = null;
        vortex.PlaybackFault += message => fault = message;
        vortex.TickPoster = _ => false;
        vortex.QueueTick(DateTime.UtcNow);
        vortex.IsPlaying.Should().BeFalse();
        fault.Should().Contain("could not be posted");
        vortex.Dispose();
        vortex.IsPlaying = true;
        vortex.PlayPause();
        vortex.IsPlaying.Should().BeTrue();
    }

    [Fact]
    public void Screenshot_success_waits_for_a_path()
    {
        KeyboardService.ExportAnnouncement(false, null, null).Should().Be("Export failed: no run is open.");
        KeyboardService.ExportAnnouncement(true, null, null).Should().Be("Export failed: no file was written.");
        KeyboardService.ExportAnnouncement(true, "frame.png", null).Should().Be("Screenshot saved");
        KeyboardService.ExportAnnouncement(true, null, new IOException("disk")).Should().Contain("disk");
    }

    [Fact]
    public void Resume_keeps_the_session_until_the_load_succeeds()
    {
        RecoveryResume.ShouldClearSession(true).Should().BeTrue();
        RecoveryResume.ShouldClearSession(false).Should().BeFalse();
        RecoveryResume.Alert(false, true, "locked", "//compare").Should().Contain("kept").And.Contain("locked");
        RecoveryResume.Alert(false, false, null, "//welcome").Should().Contain("missing");
        RecoveryResume.Alert(true, true, null, "//compare").Should().BeEmpty();
    }

    [Fact]
    public async Task Json_errors_quote_the_line_the_editor_shows()
    {
        var path = Path.Combine(Path.GetTempPath(), "ss-json-" + Guid.NewGuid().ToString("N") + ".json");
        await File.WriteAllTextAsync(path, "{ \"a\": }\n");
        var loaded = await FileValidationService.ValidateAndLoadAsync(path);
        loaded.IsSuccess.Should().BeFalse();
        loaded.GetFormattedError().Should().Contain("Near:");

        var imported = ImportSchemaService.Validate(path);
        imported.IsValid.Should().BeFalse();
        imported.Errors[0].Explanation!.TechnicalNote.Should().Contain("line 1");
        imported.Errors[0].Explanation!.TechnicalNote.Should().NotContain("line 0");
        File.Delete(path);
    }

    [Fact]
    public void Diagnostics_do_not_call_bitness_a_gpu_and_name_both_folders()
    {
        var report = DiagnosticsService.RunDiagnostics();
        var summary = report.ToSummary();
        summary.Should().Contain("does not probe a graphics device");
        summary.Should().Contain("comparison log");
        summary.Should().NotContain("GPU capable");
    }

    private static GeometryRun Run(int count, Func<int, double> time)
    {
        var steps = new List<TrajectoryTimestep>(count);
        for (var i = 0; i < count; i++)
            steps.Add(new TrajectoryTimestep { T = time(i), State2D = [i * 0.01, 0.0] });
        return new GeometryRun
        {
            Metadata = new RunMetadata { RunId = "lyapunov" },
            Trajectory = new Trajectory { Timesteps = steps }
        };
    }

    private static string FindRepoRoot()
    {
        var dir = AppContext.BaseDirectory;
        while (dir != null && !File.Exists(Path.Combine(dir, "ScalarScope.sln")))
            dir = Directory.GetParent(dir)?.FullName;
        return dir ?? throw new InvalidOperationException("Could not find repo root");
    }

    private sealed class RecordingHandler : HttpMessageHandler
    {
        private readonly List<string?> _seen;
        private readonly bool _metricsFail;

        public RecordingHandler(List<string?> seen, bool metricsFail)
        {
            _seen = seen;
            _metricsFail = metricsFail;
        }

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            _seen.Add(request.Headers.Authorization?.Parameter);
            var path = request.RequestUri?.AbsolutePath ?? "";
            if (_metricsFail && path.Contains("metrics", StringComparison.Ordinal))
                return Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotFound));
            if (path.Contains("wandb", StringComparison.Ordinal) || request.RequestUri?.Host.Contains("wandb") == true)
                return Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotFound));
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}")
            });
        }
    }
}
