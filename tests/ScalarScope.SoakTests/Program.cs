using ScalarScope.SoakTests;
using Microsoft.Extensions.Logging;

var parsed = SoakCommand.Parse(args);
if (parsed.ShowHelp)
{
    Console.WriteLine(SoakCommand.Usage());
    return 0;
}

if (!parsed.Ok)
{
    Console.Error.WriteLine(parsed.Error);
    return 1;
}

// Run Phase 3.2 validation if requested
if (parsed.Validate)
{
    var validationRunner = new Phase32ValidationRunner();
    var validationReport = await validationRunner.RunValidationAsync();

    var validationOutput = parsed.ValidationOutputPath ?? "phase32_validation_report.md";

    await File.WriteAllTextAsync(validationOutput, validationReport.ToSummaryTable());

    Console.WriteLine("╔═══════════════════════════════════════════════════════════════╗");
    Console.WriteLine("║         Phase 3.2 Validation Report Generated                 ║");
    Console.WriteLine("╚═══════════════════════════════════════════════════════════════╝");
    Console.WriteLine();
    Console.WriteLine(validationReport.ToSummaryTable());
    Console.WriteLine();
    Console.WriteLine($"  Report saved to: {validationOutput}");

    return validationReport.Locked && validationReport.AllGatesPassed ? 0 : 1;
}

// Set up logging
using var loggerFactory = LoggerFactory.Create(builder =>
{
    builder
        .SetMinimumLevel(LogLevel.Information)
        .AddConsole(options =>
        {
            options.TimestampFormat = "[HH:mm:ss] ";
        });
});

var logger = loggerFactory.CreateLogger<SoakTestRunner>();

Console.WriteLine("╔═══════════════════════════════════════════════════════════════╗");
Console.WriteLine("║           ScalarScope Soak Test Runner                     ║");
Console.WriteLine("╚═══════════════════════════════════════════════════════════════╝");
Console.WriteLine();
Console.WriteLine($"  Duration:    {parsed.DurationMinutes} minutes");
Console.WriteLine($"  Output:      {parsed.OutputPath}");
Console.WriteLine();
Console.WriteLine("  Press Ctrl+C to stop early...");
Console.WriteLine();

var config = new SoakTestConfig
{
    DurationMinutes = parsed.DurationMinutes,
    PauseBetweenIterationsSeconds = 5,
    MaxMemoryGrowthMB = 100
};

using var stop = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) =>
{
    e.Cancel = true;
    stop.Cancel();
};

var runner = new SoakTestRunner(logger, config);
var report = await runner.RunAsync(stop.Token);

// Output report
report.WriteToConsole();
await report.WriteToFileAsync(parsed.OutputPath);

Console.WriteLine();
Console.WriteLine($"  Report saved to: {parsed.OutputPath}");
Console.WriteLine();

return report.OverallPassed ? 0 : 1;
