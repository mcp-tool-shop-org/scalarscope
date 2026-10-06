using System.Runtime.CompilerServices;
using ScalarScope.Services;

namespace ScalarScope.Views;

/// <summary>
/// Applies text scale, screen-reader descriptions, and the larger pointer to the page that is showing.
/// </summary>
public static class AccessibleShell
{
    private static readonly ConditionalWeakTable<Label, StrongBox<double>> FontBases = new();

    public static void Apply(Element root)
    {
        var settings = AccessibilityService.Instance.Settings;
        var scale = ShellStartup.ClampTextScale(settings.TextScale);
        var reader = settings.ScreenReaderEnabled;
        var pointer = settings.LargePointer;

        foreach (var element in Walk(root))
        {
            if (element is Label label)
            {
                var box = FontBases.GetOrCreateValue(label);
                if (box.Value <= 0)
                    box.Value = label.FontSize > 0 ? label.FontSize : 14;
                label.FontSize = box.Value * scale;
                if (reader && !string.IsNullOrWhiteSpace(label.Text)
                    && string.IsNullOrWhiteSpace(SemanticProperties.GetDescription(label)))
                {
                    SemanticProperties.SetDescription(label, label.Text);
                }
            }
            else if (pointer && element is Button)
            {
                var button = (Button)element;
                button.MinimumHeightRequest = Math.Max(button.MinimumHeightRequest, 44);
                button.MinimumWidthRequest = Math.Max(button.MinimumWidthRequest, 44);
            }
        }
    }

    private static IEnumerable<Element> Walk(Element node)
    {
        yield return node;
        if (node is not IVisualTreeElement visual)
            yield break;

        foreach (var child in visual.GetVisualChildren())
        {
            if (child is not Element element)
                continue;
            foreach (var descendant in Walk(element))
                yield return descendant;
        }
    }
}
