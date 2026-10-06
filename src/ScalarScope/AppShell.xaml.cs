using ScalarScope.Views;

namespace ScalarScope;

public partial class AppShell : Shell
{
    public AppShell()
    {
        InitializeComponent();

        // Register routes
        Routing.RegisterRoute("recovery", typeof(RecoveryPage));

        if (App.NeedsRecovery)
            Loaded += NavigateToRecovery;
    }

    private async void NavigateToRecovery(object? sender, EventArgs e)
    {
        Loaded -= NavigateToRecovery;
        try
        {
            await GoToAsync("recovery");
        }
        catch (Exception ex)
        {
            App.RecoveryNotice = "Recovery could not open. " + ex.Message;
            try { await GoToAsync("//welcome"); }
            catch { /* Stay on the page that is already showing. */ }
        }
    }
}
