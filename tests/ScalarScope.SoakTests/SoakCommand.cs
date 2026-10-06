namespace ScalarScope.SoakTests;

/// <summary>
/// Parses soak arguments. A duration below one minute is a usage error.
/// </summary>
public static class SoakCommand
{
    public sealed record ParseResult(
        bool Ok,
        int DurationMinutes,
        string? Error,
        bool ShowHelp,
        bool Validate,
        string OutputPath,
        string? ValidationOutputPath);

    public static ParseResult Parse(string[] args)
    {
        var duration = 120;
        var showHelp = false;
        var validate = false;
        var output = "soak_test_report.json";
        string? validationOutput = null;

        for (var i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--duration":
                case "-d":
                    if (i + 1 >= args.Length || !int.TryParse(args[i + 1], out var parsed))
                        return Reject("Duration must be a whole number of minutes.");
                    if (parsed < 1)
                        return Reject("Duration must be at least 1 minute.");
                    duration = parsed;
                    i++;
                    break;
                case "--output":
                case "-o":
                    if (i + 1 >= args.Length)
                        return Reject($"Missing value for {args[i]}.");
                    output = args[i + 1];
                    i++;
                    break;
                case "--validate-output":
                    if (i + 1 >= args.Length)
                        return Reject($"Missing value for {args[i]}.");
                    validationOutput = args[i + 1];
                    i++;
                    break;
                case "--quick":
                case "-q":
                    duration = 5;
                    break;
                case "--validate":
                case "-v":
                    validate = true;
                    break;
                case "--help":
                case "-h":
                    showHelp = true;
                    break;
                default:
                    return Reject($"Unrecognized argument '{args[i]}'.");
            }
        }

        return new ParseResult(true, duration, null, showHelp, validate, output, validationOutput);
    }

    public static string Usage() =>
        """
        Usage: ScalarScope.SoakTests [options]

        Options:
          -d, --duration <minutes>  Test duration in minutes (default: 120, minimum: 1)
          -o, --output <path>       Output report path (default: soak_test_report.json)
          -q, --quick               Quick 5-minute test
          -v, --validate            Run Phase 3.2 validation suite
              --validate-output     Validation report output path
          -h, --help                Show this help
        """;

    private static ParseResult Reject(string reason)
        => new(false, 0, reason + Environment.NewLine + Usage(), false, false, "", null);
}
