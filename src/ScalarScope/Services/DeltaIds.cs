namespace ScalarScope.Services;

/// <summary>
/// One id vocabulary for the detector, the bundle schema, and the importer.
/// Wire values are the camelCase tokens in deltas-1.0.0.schema.json.
/// Older PascalCase and delta_* aliases still canonicalize.
/// </summary>
public static class DeltaIds
{
    public const string FailurePresence = "failurePresence";
    public const string ConvergenceTiming = "convergenceTiming";
    public const string StructuralEmergence = "structuralEmergence";
    public const string EvaluatorAlignment = "evaluatorAlignment";
    public const string StabilityOscillation = "stabilityOscillation";

    public static string Canonical(string? id)
    {
        return id switch
        {
            FailurePresence or "FailurePresence" or "delta_f" => FailurePresence,
            ConvergenceTiming or "ConvergenceTiming" or "delta_tc" => ConvergenceTiming,
            StructuralEmergence or "StructuralEmergence" or "delta_td" => StructuralEmergence,
            EvaluatorAlignment or "EvaluatorAlignment" or "delta_a" => EvaluatorAlignment,
            StabilityOscillation or "StabilityOscillation" or "delta_o" => StabilityOscillation,
            _ => id ?? ""
        };
    }

    /// <summary>
    /// Guide "see in context" sends a short word. The row id is the wire token.
    /// </summary>
    public static string HighlightToken(string? token) => token switch
    {
        "failure" => FailurePresence,
        "convergence" => ConvergenceTiming,
        "dominance" => StructuralEmergence,
        "alignment" => EvaluatorAlignment,
        "oscillation" => StabilityOscillation,
        _ => Canonical(token)
    };

    /// <summary>
    /// DeltaType the detector assigns for each id. Importers must use this,
    /// not a fallback of Behavior for every unrecognized string.
    /// </summary>
    public static DeltaType ToDeltaType(string? id)
    {
        return Canonical(id) switch
        {
            FailurePresence => DeltaType.Event,
            ConvergenceTiming => DeltaType.Timing,
            StructuralEmergence => DeltaType.Timing,
            EvaluatorAlignment => DeltaType.Structure,
            StabilityOscillation => DeltaType.Behavior,
            _ => DeltaType.Behavior
        };
    }
}
