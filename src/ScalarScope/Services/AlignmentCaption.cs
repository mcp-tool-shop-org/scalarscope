namespace ScalarScope.Services;

/// <summary>
/// Short captions the alignment control shows the moment a mode is chosen.
/// The comparison view-model then replaces them with the measured anchor sentence.
/// </summary>
public static class AlignmentCaption
{
    public static string For(TemporalAlignment alignment) => alignment switch
    {
        TemporalAlignment.ByStep => "Aligned by training step",
        TemporalAlignment.ByConvergence => "Aligned at convergence",
        TemporalAlignment.ByFirstInstability => "Aligned at first change",
        _ => "Aligned by training step"
    };
}
