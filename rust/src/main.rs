use scalarscope::ui::{install_style, ScalarScopeApp};

fn main() -> eframe::Result {
    // Capture mode for the Store listing's screenshots (docs/store/README.md). Unset, nothing changes.
    let capture = std::env::var_os("SCALARSCOPE_CAPTURE").map(std::path::PathBuf::from);
    let size = if capture.is_some() { [1920.0, 1080.0] } else { [1200.0, 780.0] };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(size)
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
            if let Some(out) = capture {
                let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
                let page = var("SCALARSCOPE_CAPTURE_PAGE").unwrap_or_else(|| "compare".to_string());
                let view = var("SCALARSCOPE_CAPTURE_VIEW").and_then(|value| value.parse().ok());
                let runs = var("SCALARSCOPE_CAPTURE_RUNS").map(std::path::PathBuf::from);
                let scale = var("SCALARSCOPE_CAPTURE_SCALE").and_then(|value| value.parse().ok());
                app.capture(out, &page, view, var("SCALARSCOPE_CAPTURE_WHY"), runs.as_deref(), scale);
            }
            Ok(Box::new(app))
        }),
    )
}
