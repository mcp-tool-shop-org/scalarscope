use scalarscope::ui::{install_style, ScalarScopeApp};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_title("ScalarScope"),
        ..Default::default()
    };
    eframe::run_native(
        "ScalarScope",
        options,
        Box::new(|creation| {
            install_style(creation);
            Ok(Box::new(ScalarScopeApp::default()))
        }),
    )
}
