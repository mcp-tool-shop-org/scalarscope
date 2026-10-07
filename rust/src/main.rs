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
            let mut app = ScalarScopeApp::default();
            // `scalarscope <path A> <path B>` opens that pair on Compare.
            let paths: Vec<std::path::PathBuf> = std::env::args_os().skip(1).map(std::path::PathBuf::from).collect();
            if let [left, right] = paths.as_slice() {
                app.open_pair(left, right);
            }
            Ok(Box::new(app))
        }),
    )
}
