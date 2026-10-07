//! The window. egui_plot draws the readings. It does not decide them.

use std::path::Path;

use eframe::egui::{self, Color32, RichText};
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints, Points, Polygon, VLine};

use crate::bundle::{self, OpenedBundle};
use crate::history::{self, LogEntry};
use crate::open::{open_path, open_paths, Loaded};
use crate::prefs::{self, SavedView};
use crate::readings::Band;
use crate::review::{self, InferenceReview, Pair, TrainingReview};
use crate::views;

/// The app's pages, as 2.0's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Welcome,
    Compare,
    Guide,
    Workbench,
    Settings,
}

/// The issue tracker "Report an issue" opens.
pub const ISSUES_URL: &str = "https://github.com/mcp-tool-shop-org/scalarscope/issues";
pub const PRIVACY_URL: &str = "https://github.com/mcp-tool-shop-org/scalarscope/blob/main/PRIVACY.md";
pub const SOURCE_URL: &str = "https://github.com/mcp-tool-shop-org/scalarscope";

/// The inference views. Each answers one reading question; none animates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Latency by step, with the spread band, anomaly marks and each run's levels.
    Series,
    /// The empirical CDF, with a threshold to read P(latency > x), and the quantile dots.
    Distribution,
    /// B − A by percentile, with intervals and a zero line.
    Difference,
    /// Latency against percentile on a log tail axis.
    Spectrum,
    /// Step × latency, per run, on a shared scale.
    HeatMap,
    /// Each run from its first sample, with where it settles shaded.
    Warmup,
}

impl View {
    pub const ALL: [(View, &'static str); 6] = [
        (View::Series, "Series"),
        (View::Warmup, "Warmup"),
        (View::Distribution, "Distribution"),
        (View::Difference, "Difference"),
        (View::Spectrum, "Spectrum"),
        (View::HeatMap, "Heat map"),
    ];
}

#[cfg(test)]
thread_local! {
    static NEXT_PICK: std::cell::RefCell<Option<Option<std::path::PathBuf>>> = const { std::cell::RefCell::new(None) };
    static NEXT_SAVE: std::cell::RefCell<Option<Option<std::path::PathBuf>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn take_pick() -> Option<Option<std::path::PathBuf>> {
    NEXT_PICK.with(|slot| slot.borrow_mut().take())
}

#[cfg(test)]
fn take_save() -> Option<Option<std::path::PathBuf>> {
    NEXT_SAVE.with(|slot| slot.borrow_mut().take())
}

#[cfg(not(test))]
fn take_pick() -> Option<Option<std::path::PathBuf>> {
    None
}

#[cfg(not(test))]
fn take_save() -> Option<Option<std::path::PathBuf>> {
    None
}

#[cfg(test)]
fn queue_pick(path: Option<std::path::PathBuf>) {
    NEXT_PICK.with(|slot| *slot.borrow_mut() = Some(path));
}

#[cfg(test)]
fn queue_save(path: Option<std::path::PathBuf>) {
    NEXT_SAVE.with(|slot| *slot.borrow_mut() = Some(path));
}

/// One or more run files. Several picked together are repeats of one side.
fn pick_runs() -> Option<Vec<std::path::PathBuf>> {
    if let Some(queued) = take_pick() {
        return queued.map(|path| vec![path]);
    }
    rfd::FileDialog::new().add_filter("Run", &["json", "csv", "log", "gz"]).pick_files()
}

fn pick_run_folder() -> Option<std::path::PathBuf> {
    if let Some(queued) = take_pick() {
        return queued;
    }
    rfd::FileDialog::new().pick_folder()
}

fn pick_bundle() -> Option<std::path::PathBuf> {
    if let Some(queued) = take_pick() {
        return queued;
    }
    rfd::FileDialog::new()
        .add_filter("ScalarScope bundle", &["scbundle"])
        .pick_file()
}

/// Where to write an exported picture, `kind` being `svg` or `png`.
fn pick_export_path(kind: &str, name: &str) -> Option<std::path::PathBuf> {
    if let Some(queued) = take_save() {
        return queued;
    }
    let filter = if kind == "svg" { "SVG image" } else { "PNG image" };
    rfd::FileDialog::new().add_filter(filter, &[kind]).set_file_name(format!("{name}.{kind}")).save_file()
}

/// A window screenshot as PNG bytes.
pub fn encode_png(image: &egui::ColorImage) -> Result<Vec<u8>, String> {
    let [width, height] = image.size;
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| format!("Could not write the PNG. {error}"))?;
        let pixels: Vec<u8> = image.pixels.iter().flat_map(|pixel| pixel.to_array()).collect();
        writer.write_image_data(&pixels).map_err(|error| format!("Could not write the PNG. {error}"))?;
    }
    Ok(bytes)
}

fn pick_save_path() -> Option<std::path::PathBuf> {
    if let Some(queued) = take_save() {
        return queued;
    }
    rfd::FileDialog::new()
        .add_filter("ScalarScope bundle", &["scbundle"])
        .set_file_name("review.scbundle")
        .save_file()
}



pub struct ScalarScopeApp {
    left: Option<Loaded>,
    right: Option<Loaded>,
    opened: Option<OpenedBundle>,
    note: String,
    view: View,
    /// The latency the distribution view reads P(latency > x) at; A's p95 until set.
    threshold: Option<f64>,
    /// Draw the series against seconds since each run's first sample instead of the step.
    elapsed_axis: bool,
    /// The delta whose "Why" panel is open.
    why: Option<String>,
    /// The stretch of the window "Show me" shaded, in sample indices.
    highlight: Option<(usize, usize)>,
    /// The review built from the two loaded sides, kept until either side changes. Building it
    /// runs the bootstraps, so it is not rebuilt every frame.
    built: Option<(String, Result<Pair, String>)>,
    /// The hash of the bundle saved last, for "Copy hash".
    saved_hash: Option<String>,
    page: Page,
    /// The Guide's search text.
    guide_query: String,
    /// What the Settings page edits, under 2.0's `preferences.json` keys.
    settings: prefs::ReviewPrefs,
    /// Where the screenshot asked for by "Export PNG" goes when it arrives.
    pending_png: Option<std::path::PathBuf>,
    /// The time the geometry page's marker is at; the end of the runs until moved.
    scrub: Option<f64>,
    /// Set only when this process is the Store package. An unpackaged run leaves LocalState alone.
    history_dir: Option<std::path::PathBuf>,
    recent: Vec<LogEntry>,
    files: Vec<prefs::RecentFile>,
    views: Vec<SavedView>,
    paint: Paint,
    text_scale: f32,
    sitting_key: String,
    /// The Workbench page.
    bench: bench_page::BenchState,
}

impl Default for ScalarScopeApp {
    fn default() -> Self {
        let history_dir = history::package_local_state();
        let mut recent = history_dir.as_ref().map(|dir| history::read(dir)).unwrap_or_default();
        recent.truncate(history::HOME_COUNT);
        let saved = history_dir.as_ref().map(|dir| prefs::read(dir)).unwrap_or_default();
        let mut views = history_dir.as_ref().map(|dir| prefs::saved_views(dir)).unwrap_or_default();
        views.truncate(12);
        Self {
            left: None,
            right: None,
            opened: None,
            note: String::new(),
            view: View::Series,
            threshold: None,
            elapsed_axis: false,
            why: None,
            highlight: None,
            built: None,
            saved_hash: None,
            page: Page::Welcome,
            guide_query: String::new(),
            settings: saved.clone(),
            pending_png: None,
            scrub: None,
            history_dir,
            recent,
            files: saved.recent,
            views,
            paint: Paint::from_palette(prefs::series_palette(saved.color_vision, saved.high_contrast)),
            text_scale: saved.text_scale,
            sitting_key: String::new(),
            bench: Default::default(),
        }
    }
}

impl eframe::App for ScalarScopeApp {
    /// eframe clears to a fixed near-black by default; the page has no panel of its own, so the
    /// theme's panel color is the background.
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The page is taller than the window once throughput and memory are drawn.
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.page(ui));
    }
}

impl ScalarScopeApp {
    fn page(&mut self, ui: &mut egui::Ui) {
        ui.ctx().set_zoom_factor(self.text_scale);
        self.apply_theme(ui.ctx());
        self.handle_shortcuts(ui.ctx());
        self.receive_screenshot(ui.ctx());
        let paint = self.paint;
        ui.horizontal(|ui| {
            ui.heading(RichText::new("ScalarScope").color(paint.mark));
            ui.selectable_value(&mut self.page, Page::Welcome, "Welcome");
            ui.selectable_value(&mut self.page, Page::Compare, "Compare");
            ui.selectable_value(&mut self.page, Page::Workbench, "Workbench");
            ui.selectable_value(&mut self.page, Page::Guide, "Guide");
            ui.selectable_value(&mut self.page, Page::Settings, "Settings");
        });
        ui.add_space(4.0);
        match self.page {
            Page::Settings => return self.draw_settings(ui),
            Page::Welcome => return self.draw_welcome(ui),
            Page::Guide => return self.draw_guide(ui),
            Page::Workbench => return self.draw_workbench(ui),
            Page::Compare => {}
        }
        let reviewing = self.opened.is_some();
        ui.horizontal(|ui| {
            if ui.add_enabled(!reviewing, egui::Button::new("Open path A")).clicked() {
                self.load(true, false);
            }
            if ui.add_enabled(!reviewing, egui::Button::new("Folder A")).on_hover_text("Open a run folder for side A").clicked() {
                self.load(true, true);
            }
            if ui.add_enabled(!reviewing, egui::Button::new("Open path B")).clicked() {
                self.load(false, false);
            }
            if ui.add_enabled(!reviewing, egui::Button::new("Folder B")).on_hover_text("Open a run folder for side B").clicked() {
                self.load(false, true);
            }
            if ui.button("Open bundle").clicked() {
                self.open_bundle();
            }
            if self.opened.is_none() && ui.button("Save bundle").clicked() {
                self.save_bundle();
            }
        });
        ui.add_space(6.0);
        let (left_name, right_name) = match self.opened.as_ref().and_then(|opened| stored_names(&opened.review)) {
            Some(names) => names,
            None => (side_name(&self.left, "Path A"), side_name(&self.right, "Path B")),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(left_name).color(paint.left));
            ui.label("vs");
            ui.label(RichText::new(right_name).color(paint.right));
        });
        if let Some(label) = paint.label {
            ui.label(RichText::new(format!("Series colors follow the saved {label} palette.")).color(paint.note));
        }
        if reviewing {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Review mode: a stored review is open. Loading runs is off until you close it.").color(paint.mark));
                    if ui.button("Close review").clicked() {
                        self.opened = None;
                    }
                });
            });
        }

        let opened = self.opened.clone();
        let current = if opened.is_none() { self.current_pair() } else { None };
        if let Some(Err(error)) = &current {
            self.note = error.clone();
        }
        if !self.note.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.note).color(paint.mark));
        }

        let built = current.and_then(Result::ok);
        if let Some(pair) = &built {
            let left_path = self.left.as_ref().map(|item| item.path.clone()).unwrap_or_default();
            let right_path = self.right.as_ref().map(|item| item.path.clone()).unwrap_or_default();
            self.remember(&left_path, &right_path, pair);
        }

        ui.add_space(8.0);
        if let Some(opened) = opened {
            if !opened.verified {
                ui.label(
                    RichText::new("Unverified 2.0 review: 2.0 saved it without integrity.json, so these bytes cannot be checked. The hash below is the one its JSON states.")
                        .color(paint.mark),
                );
            }
            ui.horizontal(|ui| {
                let hash = opened.bundle_hash.get(..16).unwrap_or(&opened.bundle_hash);
                let kind = if opened.verified { "Stored review" } else { "Stated hash" };
                ui.label(RichText::new(format!("{kind} · {hash}")).color(paint.note));
                if ui.button("Copy hash").on_hover_text("Copy the full SHA-256 content check").clicked() {
                    ui.ctx().copy_text(opened.bundle_hash.clone());
                }
            });
            ui.label(RichText::new(bundle::CONTENT_CHECK).color(paint.note));
            ui.add_space(6.0);
            match bundle::stored_pair(&opened.review) {
                Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
                Some(Pair::Training(review)) => draw_training(ui, &review, paint),
                Some(Pair::Geometry(review)) => self.draw_geometry(ui, &review),
                None => {
                    if !opened.review.left_text.is_empty() || !opened.review.right_text.is_empty() {
                        ui.label(RichText::new(format!("{} vs {}", opened.review.left_text, opened.review.right_text)).color(paint.note));
                    }
                    if !opened.review.verdict.is_empty() {
                        ui.label(RichText::new(&opened.review.verdict).color(self.paint.text));
                    }
                    for line in &opened.review.notices {
                        ui.label(RichText::new(line).color(paint.note));
                    }
                    self.draw_tiles(ui, &opened.review.explanations);
                    ui.label(RichText::new(&opened.review.caption).color(paint.note));
                }
            }
        } else {
            match built {
                Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
                Some(Pair::Training(review)) => draw_training(ui, &review, paint),
                Some(Pair::Geometry(review)) => self.draw_geometry(ui, &review),
                None => {
                    ui.label(
                        RichText::new("Open two inference traces, or two backpropagate run histories. Or open a .scbundle.")
                            .color(paint.note),
                    );
                }
            }
        }
    }
}

impl ScalarScopeApp {
    fn load(&mut self, left: bool, folder: bool) {
        let picked = if folder { pick_run_folder().map(|path| vec![path]) } else { pick_runs() };
        let Some(paths) = picked.filter(|paths| !paths.is_empty()) else {
            return;
        };
        self.built = None;
        self.highlight = None;
        self.page = Page::Compare;
        match open_paths(&paths) {
            Ok(loaded) => {
                self.opened = None;
                let remembered = self.remember_opened_file(&loaded);
                if left {
                    self.left = Some(loaded);
                } else {
                    self.right = Some(loaded);
                }
                if remembered {
                    self.note.clear();
                }
            }
            Err(error) => {
                if left {
                    self.left = None;
                } else {
                    self.right = None;
                }
                self.note = error;
            }
        }
    }

    fn open_bundle(&mut self) {
        let Some(path) = pick_bundle() else {
            return;
        };
        match bundle::open_file(&path) {
            Ok(opened) => {
                let path_text = path.display().to_string();
                let hash = opened.bundle_hash.clone();
                self.opened = Some(opened);
                self.note.clear();
                self.record_opened_bundle(&path_text, &hash);
            }
            Err(error) => {
                self.opened = None;
                self.note = error;
            }
        }
    }

    fn save_bundle(&mut self) {
        let (Some(left), Some(right)) = (&self.left, &self.right) else {
            self.note = "Open both sides before saving a bundle.".to_string();
            return;
        };
        let built = match review::pair(&left.side, &right.side) {
            Ok(pair) => pair,
            Err(error) => {
                self.note = error;
                return;
            }
        };
        let Some(path) = pick_save_path() else {
            return;
        };
        let document = bundle::document_from_pair(&built);
        match bundle::seal(&document, &bundle::utc_now()) {
            Ok(sealed) => match bundle::write_file(&path, &sealed) {
                Ok(()) => {
                    self.note = format!("Saved the stored review. Content check {}.", sealed.bundle_hash);
                    self.saved_hash = Some(sealed.bundle_hash.clone());
                    if let Some(dir) = self.history_dir.clone() {
                        match history::attach_bundle(&path.display().to_string(), Some(&sealed.bundle_hash), &dir) {
                            Ok(_) => self.refresh_recent(),
                            Err(error) => self.note = format!("{} The history was not stamped. {error}", self.note),
                        }
                    }
                }
                Err(error) => self.note = error,
            },
            Err(error) => self.note = error,
        }
    }

    fn draw_inference(&mut self, ui: &mut egui::Ui, review: &InferenceReview) {
        let paint = self.paint;
        ui.horizontal(|ui| {
            for (view, name) in View::ALL {
                ui.selectable_value(&mut self.view, view, name);
            }
            ui.separator();
            let vector = self.view != View::HeatMap;
            if ui
                .add_enabled(vector, egui::Button::new("Export SVG"))
                .on_hover_text("This view as a vector drawing")
                .on_disabled_hover_text("The heat map exports as PNG")
                .clicked()
            {
                self.export_svg(review);
            }
            if ui.button("Export PNG").on_hover_text("A picture of the window").clicked() {
                self.pending_png = pick_export_path("png", "scalarscope");
                if self.pending_png.is_some() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                    ui.ctx().request_repaint();
                }
            }
        });
        ui.label(RichText::new(&review.left_text).color(paint.note));
        ui.label(RichText::new(&review.right_text).color(paint.note));
        ui.add_space(4.0);
        if !review.headline.is_empty() {
            ui.label(RichText::new(&review.headline).color(self.paint.text).strong());
        }
        ui.label(RichText::new(&review.verdict).color(self.paint.text));
        for line in &review.notices {
            ui.label(RichText::new(line).color(paint.note));
        }
        self.draw_tiles(ui, &review.explanations);
        match self.view {
            View::Series => {}
            View::Distribution => {
                self.draw_distribution(ui, review);
                return self.draw_caption(ui, review);
            }
            View::Difference => {
                draw_difference(ui, review, paint);
                return self.draw_caption(ui, review);
            }
            View::Spectrum => {
                draw_spectrum(ui, review, paint);
                return self.draw_caption(ui, review);
            }
            View::HeatMap => {
                draw_heat_maps(ui, review, paint);
                return self.draw_caption(ui, review);
            }
            View::Warmup => {
                draw_warmup(ui, review, paint);
                return self.draw_caption(ui, review);
            }
        }
        let timed = !review.left_elapsed.is_empty() && !review.right_elapsed.is_empty();
        if timed {
            ui.checkbox(&mut self.elapsed_axis, "Elapsed seconds on x (each run from its own first sample)");
        }
        let (left_x, right_x) = if timed && self.elapsed_axis {
            (Some(review.left_elapsed.as_slice()), Some(review.right_elapsed.as_slice()))
        } else {
            (None, None)
        };
        Plot::new("review")
            .height(360.0)
            .legend(egui_plot::Legend::default())
            .x_axis_label(if left_x.is_some() { "elapsed s" } else { "step" })
            .y_axis_label("ms")
            .show(ui, |plot| {
                {
                    for (name, color, band, xs) in [("A spread", paint.left, &review.left_band, left_x), ("B spread", paint.right, &review.right_band, right_x)] {
                        let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 72);
                        for polygon in band_polygons(band) {
                            plot.polygon(Polygon::new(name, PlotPoints::new(at_x(polygon, xs))).fill_color(fill).stroke(egui::Stroke::new(0.0, color)));
                        }
                    }
                    if let Some((from, to)) = self.highlight {
                        let range = x_of(from, left_x)..=x_of(to.saturating_sub(1).max(from), left_x);
                        plot.span(egui_plot::Span::new("shown", range).fill(Color32::from_rgba_unmultiplied(paint.mark.r(), paint.mark.g(), paint.mark.b(), 40)));
                    }
                    plot.line(series_line("A", paint.left, at_x(value_points(&review.left), left_x)));
                    plot.line(series_line("B", paint.right, at_x(value_points(&review.right), right_x)));
                    plot.points(Points::new("A marks", PlotPoints::new(at_x(mark_coords(&review.left, &review.left_marks), left_x))).color(paint.mark).radius(4.0));
                    plot.points(Points::new("B marks", PlotPoints::new(at_x(mark_coords(&review.right, &review.right_marks), right_x))).color(paint.mark).radius(4.0));
                    if let Some(step) = review.left_steady {
                        plot.vline(VLine::new("A steady", x_of(step, left_x)).color(paint.note).width(1.0));
                    }
                    if let Some(step) = review.right_steady {
                        plot.vline(VLine::new("B steady", x_of(step, right_x)).color(paint.note).width(1.0));
                    }
                    for (name, color, segments, xs) in [("A levels", paint.left, &review.left_segments, left_x), ("B levels", paint.right, &review.right_segments, right_x)] {
                        for segment in segments.iter() {
                            let points = vec![[x_of(segment.start, xs), segment.level], [x_of(segment.end.saturating_sub(1), xs), segment.level]];
                            plot.line(Line::new(name, PlotPoints::new(points)).color(color).width(1.0).style(LineStyle::dashed_loose()));
                        }
                    }
                }
            });
        if !review.left_throughput.is_empty() && !review.right_throughput.is_empty() {
            ui.label(RichText::new("Throughput, items/s, same steps, own scale.").color(paint.note));
            Plot::new("throughput")
                .height(140.0)
                .y_axis_label("items/s")
                .show(ui, |plot| {
                    plot.line(series_line("A throughput", paint.left, plain_points(&review.left_throughput)));
                    plot.line(series_line("B throughput", paint.right, plain_points(&review.right_throughput)));
                });
        }
        let utilization = (!review.left_cpu.is_empty() && !review.right_cpu.is_empty()) || (!review.left_gpu.is_empty() && !review.right_gpu.is_empty());
        if utilization {
            ui.label(RichText::new("Utilization, %, same steps: CPU solid, GPU dashed.").color(paint.note));
            Plot::new("utilization")
                .height(120.0)
                .include_y(0.0)
                .include_y(100.0)
                .y_axis_label("%")
                .show(ui, |plot| {
                    plot.line(series_line("A CPU", paint.left, plain_points(&review.left_cpu)));
                    plot.line(series_line("B CPU", paint.right, plain_points(&review.right_cpu)));
                    plot.line(series_line("A GPU", paint.left, plain_points(&review.left_gpu)).style(LineStyle::dashed_loose()));
                    plot.line(series_line("B GPU", paint.right, plain_points(&review.right_gpu)).style(LineStyle::dashed_loose()));
                });
        }
        if !review.left_memory.is_empty() && !review.right_memory.is_empty() {
            ui.label(RichText::new("Memory, MiB, same steps, own scale.").color(paint.note));
            Plot::new("memory")
                .height(120.0)
                .y_axis_label("MiB")
                .show(ui, |plot| {
                    plot.line(series_line("A memory", paint.left, plain_points(&review.left_memory)));
                    plot.line(series_line("B memory", paint.right, plain_points(&review.right_memory)));
                });
        }
        self.draw_caption(ui, review);
    }

    /// The two sides' review, built when either side changed since the last frame.
    fn current_pair(&mut self) -> Option<Result<Pair, String>> {
        let (Some(left), Some(right)) = (&self.left, &self.right) else {
            self.built = None;
            return None;
        };
        let options = self.options();
        let key = format!("{}|{}|{:?}|{:?}|{options:?}", left.path, right.path, side_name(&self.left, ""), side_name(&self.right, ""));
        if self.built.as_ref().map(|(cached, _)| cached) != Some(&key) {
            self.built = Some((key, review::pair_with(&left.side, &right.side, options)));
        }
        self.built.as_ref().map(|(_, pair)| pair.clone())
    }

    fn svg_colors(&self) -> crate::svg::Colors {
        let rgb = |color: Color32| [color.r(), color.g(), color.b()];
        crate::svg::Colors {
            left: rgb(self.paint.left),
            right: rgb(self.paint.right),
            mark: rgb(self.paint.mark),
            note: rgb(self.paint.note),
            text: rgb(self.paint.text),
            background: rgb(self.paint.background),
        }
    }

    /// The current view as an SVG file.
    fn export_svg(&mut self, review: &InferenceReview) {
        let colors = self.svg_colors();
        let (name, svg) = match self.view {
            View::Series | View::HeatMap => ("series", crate::svg::series(review, colors)),
            View::Warmup => ("warmup", crate::svg::warmup(review, colors)),
            View::Distribution => ("distribution", crate::svg::distribution(review, colors, self.threshold.or(review.left_p95))),
            View::Difference => ("difference", crate::svg::difference(review, colors)),
            View::Spectrum => ("spectrum", crate::svg::spectrum(review, colors)),
        };
        let Some(path) = pick_export_path("svg", name) else {
            return;
        };
        self.note = match std::fs::write(&path, svg) {
            Ok(()) => format!("Saved the {name} view as SVG."),
            Err(error) => format!("Could not write the SVG. {error}"),
        };
    }

    fn receive_screenshot(&mut self, ctx: &egui::Context) {
        if self.pending_png.is_none() {
            return;
        }
        let image = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        let Some(image) = image else {
            // The screenshot arrives in a later frame; keep frames coming until it does.
            ctx.request_repaint();
            return;
        };
        let Some(path) = self.pending_png.take() else {
            return;
        };
        self.note = match encode_png(&image).and_then(|bytes| std::fs::write(&path, bytes).map_err(|error| format!("Could not write the PNG. {error}"))) {
            Ok(()) => "Saved a PNG of the window.".to_string(),
            Err(error) => error,
        };
    }

    fn options(&self) -> review::Options {
        review::Options {
            anomaly: crate::stats::AnomalyRule::from_code(self.settings.anomaly_rule),
        }
    }

    /// Light or dark, from the setting or from Windows, with the matching chart colors.
    /// egui keeps one look per theme and picks by its theme preference, so the preference is set
    /// from the setting and each theme gets its own panel colors.
    fn apply_theme(&mut self, ctx: &egui::Context) {
        let preference = match self.settings.theme {
            1 => egui::ThemePreference::Light,
            2 => egui::ThemePreference::Dark,
            _ => egui::ThemePreference::System,
        };
        ctx.set_theme(preference);
        let palette = prefs::series_palette(self.settings.color_vision, self.settings.high_contrast);
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let light = theme == egui::Theme::Light;
            let paint = Paint::themed(palette, light);
            let mut visuals = if light { egui::Visuals::light() } else { egui::Visuals::dark() };
            visuals.panel_fill = paint.background;
            visuals.window_fill = paint.background;
            // Text fields and plot backgrounds sit a shade deeper than the page, so a field is visible.
            visuals.extreme_bg_color = if light { egui::Color32::WHITE } else { paint.background.gamma_multiply(0.6) };
            ctx.set_visuals_of(theme, visuals);
        }
        self.paint = Paint::themed(palette, ctx.theme() == egui::Theme::Light);
    }

    /// Save the settings and rebuild what depends on them.
    fn settings_changed(&mut self) {
        self.text_scale = self.settings.text_scale;
        self.built = None;
        if let Some(dir) = self.history_dir.clone() {
            if let Err(error) = prefs::write_settings(&dir, &self.settings) {
                self.note = error;
            }
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        let before = self.settings.clone();
        ui.heading("Appearance");
        ui.horizontal(|ui| {
            ui.label("Theme");
            ui.radio_value(&mut self.settings.theme, 0, "Follow Windows");
            ui.radio_value(&mut self.settings.theme, 1, "Light");
            ui.radio_value(&mut self.settings.theme, 2, "Dark");
        });
        ui.horizontal(|ui| {
            ui.label("Series colors");
            let names = ["Default", "Deuteranopia", "Protanopia", "Tritanopia", "High contrast", "Monochrome"];
            egui::ComboBox::from_id_salt("color-vision")
                .selected_text(names[usize::from(self.settings.color_vision.min(5))])
                .show_ui(ui, |ui| {
                    for (code, name) in names.iter().enumerate() {
                        ui.selectable_value(&mut self.settings.color_vision, code as u8, *name);
                    }
                });
        });
        ui.checkbox(&mut self.settings.high_contrast, "High contrast (when the series colors are Default)");
        ui.add(egui::Slider::new(&mut self.settings.text_scale, 0.75..=2.0).text("Text scale"));
        ui.add_space(8.0);
        ui.heading("Analysis");
        ui.label(RichText::new("Which samples count as anomalies, for the marks and ΔF. A stored review's caption says which rule made it.").color(paint.note));
        ui.radio_value(&mut self.settings.anomaly_rule, 0, "More than 5 robust deviations (1.4826 × MAD) from the median (recommended)");
        ui.radio_value(&mut self.settings.anomaly_rule, 1, "More than 3 standard deviations from the mean (the 2.0 rule; the spikes inflate the standard deviation)");
        ui.add_space(8.0);
        ui.heading("Recent files");
        ui.horizontal(|ui| {
            let mut limit = self.settings.recent_limit as u32;
            ui.label("Keep");
            if ui.add(egui::DragValue::new(&mut limit).range(1..=40)).changed() {
                self.settings.recent_limit = limit as usize;
            }
            ui.label("files");
            if ui.button("Clear recent files").clicked() {
                if let Some(dir) = self.history_dir.clone() {
                    match prefs::clear_recent(&dir) {
                        Ok(()) => self.files.clear(),
                        Err(error) => self.note = error,
                    }
                } else {
                    self.files.clear();
                }
            }
        });
        if self.history_dir.is_none() {
            ui.label(RichText::new("This unpackaged run keeps settings for this session only. The Store app keeps them in its LocalState folder.").color(paint.note));
        }
        if self.settings != before {
            self.settings_changed();
        }
        ui.add_space(8.0);
        ui.heading("About");
        ui.label(format!("ScalarScope {} (package 3.0.0.0, Microsoft Store 9P3HT1PHBKQK)", env!("CARGO_PKG_VERSION")));
        ui.label(RichText::new("Privacy: ScalarScope reads only the files you open and writes only the bundles you save and its own settings and history in its package folder. It sends nothing anywhere: no account, no telemetry, no analytics.").color(paint.note));
        ui.horizontal(|ui| {
            ui.hyperlink_to("Report an issue", ISSUES_URL);
            ui.hyperlink_to("Privacy policy", PRIVACY_URL);
            ui.hyperlink_to("Source", SOURCE_URL);
        });
        if !self.note.is_empty() {
            ui.label(RichText::new(&self.note).color(paint.mark));
        }
    }

    /// One tile per delta. A tile opens its "Why" panel.
    fn draw_tiles(&mut self, ui: &mut egui::Ui, explanations: &[review::Explanation]) {
        if explanations.is_empty() {
            return;
        }
        let paint = self.paint;
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for tile in explanations {
                let color = match tile.status.as_str() {
                    "fired" => paint.mark,
                    "withheld" => paint.note,
                    _ => paint.note,
                };
                let open = self.why.as_deref() == Some(tile.symbol.as_str());
                let text = RichText::new(format!("{} {} · {}", tile.symbol, tile.status, tile.headline)).color(color);
                if ui.selectable_label(open, text).on_hover_text("Why did this fire, or not?").clicked() {
                    self.why = if open { None } else { Some(tile.symbol.clone()) };
                }
            }
        });
        let Some(tile) = explanations.iter().find(|tile| Some(tile.symbol.as_str()) == self.why.as_deref()) else {
            return;
        };
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(format!("{} {}: {}", tile.symbol, tile.status, tile.headline)).color(self.paint.text).strong());
            ui.add(egui::Label::new(RichText::new(&tile.why).color(paint.note)).wrap());
            egui::Grid::new(format!("why-{}", tile.symbol)).num_columns(2).show(ui, |ui| {
                for [name, value] in &tile.parameters {
                    ui.label(RichText::new(name).color(paint.note));
                    ui.label(value);
                    ui.end_row();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Copy finding").clicked() {
                    let parameters: Vec<String> = tile.parameters.iter().map(|[name, value]| format!("{name}: {value}")).collect();
                    ui.ctx().copy_text(format!("{} {}: {}\n{}\n{}", tile.symbol, tile.status, tile.headline, tile.why, parameters.join("\n")));
                }
                if let Some(anchor) = &tile.anchor {
                    if ui.button("Show me").clicked() {
                        if anchor.view == "geometry" {
                            self.scrub = anchor.time;
                        }
                        self.view = match anchor.view.as_str() {
                            "distribution" => View::Distribution,
                            "warmup" => View::Warmup,
                            _ => View::Series,
                        };
                        self.highlight = (anchor.view == "series").then_some((anchor.from, anchor.to));
                    }
                }
                if ui.button("Close").clicked() {
                    self.why = None;
                    self.highlight = None;
                }
            });
        });
    }

    fn draw_caption(&self, ui: &mut egui::Ui, review: &InferenceReview) {
        ui.add_space(6.0);
        ui.label(RichText::new(&review.caption).color(self.paint.note));
    }

    /// The CDF with a threshold the reader drags, its exceedance read-out, and the quantile dots.
    fn draw_distribution(&mut self, ui: &mut egui::Ui, review: &InferenceReview) {
        let paint = self.paint;
        let threshold = self.threshold.or(review.left_p95).unwrap_or(0.0);
        let mut moved = None;
        Plot::new("distribution")
            .height(320.0)
            .legend(egui_plot::Legend::default())
            .allow_drag(false)
            .x_axis_label("latency_ms")
            .y_axis_label("empirical CDF")
            .show(ui, |plot| {
                plot.line(series_line("A distribution", paint.left, cdf_points(&review.left_cdf)));
                plot.line(series_line("B distribution", paint.right, cdf_points(&review.right_cdf)));
                plot.vline(VLine::new("threshold", threshold).color(paint.mark).width(1.5));
                let response = plot.response();
                if response.clicked() || response.dragged() {
                    moved = plot.pointer_coordinate().map(|point| point.x);
                }
            });
        if let Some(x) = moved {
            self.threshold = Some(x);
        }
        let share = |values: &[Option<f64>]| {
            views::exceedance(values, threshold).map_or("none".to_string(), |share| format!("{:.1}%", share * 100.0))
        };
        ui.label(
            RichText::new(format!(
                "P(latency > {threshold:.3} ms): A {} · B {}. Click or drag on the plot to move the threshold.",
                share(&review.left),
                share(&review.right)
            ))
            .color(self.paint.text),
        );
        let (left_dots, right_dots) = (views::quantile_strip(&review.left), views::quantile_strip(&review.right));
        if !left_dots.is_empty() && !right_dots.is_empty() {
            ui.label(RichText::new("Quantile dots: each dot is 5% of the samples in this window.").color(paint.note));
            Plot::new("quantile-dots")
                .height(90.0)
                .allow_drag(false)
                .show_axes([true, false])
                .include_y(-0.5)
                .include_y(1.5)
                .x_axis_label("latency_ms")
                .show(ui, |plot| {
                    let row = |dots: &[f64], y: f64| dots.iter().map(|dot| [*dot, y]).collect::<Vec<_>>();
                    plot.points(Points::new("A dots", PlotPoints::new(row(&left_dots, 1.0))).color(paint.left).radius(4.0));
                    plot.points(Points::new("B dots", PlotPoints::new(row(&right_dots, 0.0))).color(paint.right).radius(4.0));
                });
        }
    }
}

/// A percentile label for a position on the nines axis: 0.3 is p50, 1 is p90, 2 is p99.
fn percentile_label(nines: f64) -> String {
    let percent = 100.0 * (1.0 - 10f64.powf(-nines));
    let text = format!("{percent:.1}");
    format!("p{}", text.trim_end_matches('0').trim_end_matches('.'))
}

/// Grid marks on the nines axis at the percentiles people read: p0, p50, p75, p90, p95, p99, p99.9.
fn percentile_marks(input: egui_plot::GridInput) -> Vec<egui_plot::GridMark> {
    [0.0, 0.5, 0.75, 0.9, 0.95, 0.99, 0.999]
        .into_iter()
        .map(views::nines)
        .filter(|value| *value >= input.bounds.0 && *value <= input.bounds.1)
        .map(|value| egui_plot::GridMark { value, step_size: 1.0 })
        .collect()
}

fn draw_difference(ui: &mut egui::Ui, review: &InferenceReview, paint: Paint) {
    if review.difference.is_empty() {
        ui.label(RichText::new("No percentile has enough steady samples on both sides for a difference.").color(paint.note));
        return;
    }
    ui.label(
        RichText::new("B − A in ms by percentile, over the steady samples. The bar is the 95% interval, within one run per side; below zero, B is faster.")
            .color(paint.note),
    );
    Plot::new("difference")
        .height(320.0)
        .x_axis_label("percentile")
        .y_axis_label("B − A, ms")
        .x_grid_spacer(percentile_marks)
        .x_axis_formatter(|mark, _| percentile_label(mark.value))
        .show(ui, |plot| {
            plot.hline(HLine::new("no difference", 0.0).color(paint.note).width(1.0));
            for point in &review.difference {
                let x = views::nines(point.probability);
                plot.line(Line::new("interval", PlotPoints::new(vec![[x, point.low], [x, point.high]])).color(paint.right).width(3.0));
            }
            let estimates: Vec<[f64; 2]> = review.difference.iter().map(|point| [views::nines(point.probability), point.estimate]).collect();
            plot.points(Points::new("B − A", PlotPoints::new(estimates)).color(paint.mark).radius(5.0));
        });
}

fn draw_spectrum(ui: &mut egui::Ui, review: &InferenceReview, paint: Paint) {
    ui.label(
        RichText::new("Latency by percentile on a log tail axis. Each line stops at the highest percentile its sample count bounds.")
            .color(paint.note),
    );
    Plot::new("spectrum")
        .height(320.0)
        .legend(egui_plot::Legend::default())
        .x_axis_label("percentile")
        .y_axis_label("ms")
        .x_grid_spacer(percentile_marks)
        .x_axis_formatter(|mark, _| percentile_label(mark.value))
        .show(ui, |plot| {
            plot.line(series_line("A", paint.left, views::spectrum(&review.left)));
            plot.line(series_line("B", paint.right, views::spectrum(&review.right)));
        });
}

/// Each run from its first sample: the warmup the series view leaves out, and where ΔTc reads
/// each run as settled, shaded in that run's color.
fn draw_warmup(ui: &mut egui::Ui, review: &InferenceReview, paint: Paint) {
    ui.label(
        RichText::new("Each run from its first sample. The shaded stretch is where it settles: a range when it is detected, one step when the file states it.")
            .color(paint.note),
    );
    let full = |lead: &[f64], window: &[Option<f64>]| -> Vec<Option<f64>> { lead.iter().map(|value| Some(*value)).chain(window.iter().copied()).collect() };
    let (left, right) = (full(&review.left_lead, &review.left), full(&review.right_lead, &review.right));
    Plot::new("warmup")
        .height(360.0)
        .legend(egui_plot::Legend::default())
        .x_axis_label("sample from the run's first")
        .y_axis_label("ms")
        .show(ui, |plot| {
            for (name, color, settle) in [("A settles", paint.left, review.left_settle), ("B settles", paint.right, review.right_settle)] {
                if let Some((low, high)) = settle {
                    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 50);
                    plot.span(egui_plot::Span::new(name, low as f64..=high.max(low) as f64 + 0.5).fill(fill));
                }
            }
            plot.line(series_line("A", paint.left, value_points(&left)));
            plot.line(series_line("B", paint.right, value_points(&right)));
        });
}

fn draw_heat_maps(ui: &mut egui::Ui, review: &InferenceReview, paint: Paint) {
    let Some(range) = views::heat_range(&review.left, &review.right) else {
        ui.label(RichText::new("Too few samples for a heat map.").color(paint.note));
        return;
    };
    ui.label(
        RichText::new("Where the samples fall, step by step. Each column is that stretch's own distribution; the scale is shared, and samples outside p0.5–p99.5 sit on the edge rows.")
            .color(paint.note),
    );
    for (name, color, values) in [("A", paint.left, &review.left), ("B", paint.right, &review.right)] {
        ui.label(RichText::new(name).color(color));
        Plot::new(format!("heat-{name}"))
            .height(160.0)
            .include_y(range.0)
            .include_y(range.1)
            .x_axis_label("step")
            .y_axis_label("ms")
            .show(ui, |plot| {
                for cell in views::heat_cells(values, range) {
                    let alpha = (40.0 + 215.0 * cell.share).round() as u8;
                    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
                    let corners = vec![[cell.x0, cell.y0], [cell.x1, cell.y0], [cell.x1, cell.y1], [cell.x0, cell.y1]];
                    plot.polygon(Polygon::new(name, PlotPoints::new(corners)).fill_color(fill).stroke(egui::Stroke::new(0.0, color)));
                }
            });
    }
}

fn draw_training(ui: &mut egui::Ui, review: &TrainingReview, paint: Paint) {
    ui.label(RichText::new(&review.left_text).color(paint.left));
    ui.label(RichText::new(&review.right_text).color(paint.right));
    Plot::new("training-loss")
        .height(420.0)
        .legend(egui_plot::Legend::default())
        .x_axis_label("stored sample")
        .y_axis_label("training loss")
        .show(ui, |plot| {
            plot.line(series_line(&review.left.run_id, paint.left, plain_points(&review.left.loss)));
            plot.line(series_line(&review.right.run_id, paint.right, plain_points(&review.right.loss)));
        });
    ui.add_space(6.0);
    ui.label(RichText::new(&review.caption).color(paint.note));
}

fn series_line(name: &str, color: Color32, points: Vec<[f64; 2]>) -> Line<'static> {
    Line::new(name, PlotPoints::new(points)).color(color).width(2.0)
}

fn value_points(values: &[Option<f64>]) -> Vec<[f64; 2]> {
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.map(|sample| [index as f64, sample]))
        .collect()
}

fn plain_points(values: &[f64]) -> Vec<[f64; 2]> {
    values.iter().enumerate().map(|(index, sample)| [index as f64, *sample]).collect()
}

fn cdf_points(points: &[(f64, f64)]) -> Vec<[f64; 2]> {
    points.iter().map(|(value, probability)| [*value, *probability]).collect()
}

/// The x of a sample index: its elapsed seconds when the axis is time, else the index.
fn x_of(index: usize, xs: Option<&[f64]>) -> f64 {
    xs.and_then(|xs| xs.get(index).copied()).unwrap_or(index as f64)
}

/// Points whose x is a sample index, moved onto the elapsed-time axis when there is one.
fn at_x(points: Vec<[f64; 2]>, xs: Option<&[f64]>) -> Vec<[f64; 2]> {
    if xs.is_none() {
        return points;
    }
    points.into_iter().map(|[x, y]| [x_of(x.max(0.0) as usize, xs), y]).collect()
}

fn mark_coords(values: &[Option<f64>], marks: &[usize]) -> Vec<[f64; 2]> {
    marks
        .iter()
        .filter_map(|index| values.get(*index).copied().flatten().map(|sample| [*index as f64, sample]))
        .collect()
}

/// The band as one trapezoid per pair of neighbouring steps that both have a band.
/// A trapezoid is convex, so the plot fills it exactly; a gap in the band leaves a gap.
fn band_polygons(band: &[Option<Band>]) -> Vec<Vec<[f64; 2]>> {
    band.windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            let (Some(here), Some(next)) = (pair[0], pair[1]) else {
                return None;
            };
            let (x, x_next) = (index as f64, (index + 1) as f64);
            Some(vec![[x, here.low], [x_next, next.low], [x_next, next.high], [x, here.high]])
        })
        .collect()
}

impl ScalarScopeApp {
    fn remember(&mut self, left_path: &str, right_path: &str, pair: &Pair) {
        let Some(dir) = self.history_dir.clone() else {
            return;
        };
        let (alignment, deltas, left_name, right_name, left_run, right_run) = match pair {
            Pair::Inference(review) => (
                "latency".to_string(),
                review.fired.clone(),
                review.left_label.clone(),
                review.right_label.clone(),
                None,
                None,
            ),
            Pair::Geometry(review) => (
                "geometry".to_string(),
                Vec::new(),
                review.left_label.clone(),
                review.right_label.clone(),
                Some(review.left.metadata.run_id.clone()),
                Some(review.right.metadata.run_id.clone()),
            ),
            Pair::Training(review) => (
                "loss".to_string(),
                Vec::new(),
                review.left.run_id.clone(),
                review.right.run_id.clone(),
                Some(review.left.run_id.clone()),
                Some(review.right.run_id.clone()),
            ),
        };
        let key = format!("{left_path}|{right_path}|{alignment}|{}", deltas.join(","));
        if key == self.sitting_key {
            return;
        }
        let entry = LogEntry {
            id: String::new(),
            finished_at: String::new(),
            left_name,
            right_name,
            left_path: Some(left_path.to_string()),
            right_path: Some(right_path.to_string()),
            left_run_id: left_run,
            right_run_id: right_run,
            bundle_path: None,
            bundle_hash: None,
            alignment,
            deltas_fired: deltas,
            kind: "compare".to_string(),
        };
        match history::record(entry, &dir) {
            Ok(_) => {
                self.sitting_key = key;
                self.refresh_recent();
            }
            Err(error) => self.note = error,
        }
    }

    fn record_opened_bundle(&mut self, path: &str, hash: &str) {
        let Some(dir) = self.history_dir.clone() else {
            return;
        };
        let name = Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("bundle")
            .to_string();
        let entry = LogEntry {
            id: String::new(),
            finished_at: String::new(),
            left_name: name,
            right_name: String::new(),
            left_path: None,
            right_path: None,
            left_run_id: None,
            right_run_id: None,
            bundle_path: Some(path.to_string()),
            bundle_hash: Some(hash.to_string()),
            alignment: String::new(),
            deltas_fired: Vec::new(),
            kind: "bundle".to_string(),
        };
        if history::record(entry, &dir).is_ok() {
            self.refresh_recent();
        }
    }

    fn refresh_recent(&mut self) {
        let Some(dir) = &self.history_dir else {
            self.recent.clear();
            return;
        };
        let mut entries = history::read(dir);
        entries.truncate(history::HOME_COUNT);
        self.recent = entries;
    }

    fn draw_recent(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        if self.history_dir.is_none() {
            ui.label(
                RichText::new("Recent reviews stay in the Store package folder. This unpackaged run leaves that file alone.")
                    .color(self.paint.note),
            );
            return;
        }
        ui.label(RichText::new("Recent").color(self.paint.mark));
        if self.recent.is_empty() {
            ui.label(RichText::new("No reviews in this package folder yet.").color(self.paint.note));
        } else {
            let recent = self.recent.clone();
            for entry in recent {
                let label = format!("{}   {}", entry.title(), entry.subtitle());
                if ui.button(label).clicked() {
                    self.reopen(&entry);
                }
            }
        }
        self.draw_files(ui);
        self.draw_views(ui);
    }

    fn draw_files(&mut self, ui: &mut egui::Ui) {
        let present: Vec<_> = self.files.iter().filter(|file| Path::new(&file.path).is_file()).cloned().collect();
        let missing = self.files.len().saturating_sub(present.len());
        if present.is_empty() && missing == 0 {
            return;
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Files").color(self.paint.mark));
        if missing > 0 {
            ui.label(RichText::new(format!("{missing} saved files are not on this machine.")).color(self.paint.note));
        }
        for file in present {
            let path = file.path.clone();
            if ui.button(file.name).clicked() {
                self.open_saved_path(&path);
            }
        }
    }

    fn draw_views(&mut self, ui: &mut egui::Ui) {
        if self.views.is_empty() {
            return;
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Saved views").color(self.paint.mark));
        let views = self.views.clone();
        for view in views {
            if ui.button(view.title).clicked() {
                match view.source {
                    Some(path) => self.open_saved_path(&path),
                    None => self.note = "That saved view has no file.".to_string(),
                }
            }
        }
    }

    fn open_saved_path(&mut self, path: &str) {
        match open_path(Path::new(path)) {
            Ok(loaded) => {
                self.opened = None;
                let name = loaded.side.name();
                let slot = if self.left.is_none() { "A" } else { "B" };
                let preferences_note = if self.remember_opened_file(&loaded) {
                    None
                } else {
                    Some(self.note.clone())
                };
                if self.left.is_none() {
                    self.left = Some(loaded);
                } else {
                    self.right = Some(loaded);
                }
                self.sitting_key.clear();
                self.note = match preferences_note {
                    Some(error) => format!("Opened {name} as path {slot}. {error}"),
                    None => format!("Opened {name} as path {slot}."),
                };
            }
            Err(error) => self.note = error,
        }
    }

    fn remember_opened_file(&mut self, loaded: &Loaded) -> bool {
        let Some(dir) = self.history_dir.clone() else {
            return true;
        };
        let name = loaded.side.name();
        match prefs::remember_file(&dir, &loaded.path, &name) {
            Ok(()) => {
                self.files = prefs::read(&dir).recent;
                true
            }
            Err(error) => {
                self.note = error;
                false
            }
        }
    }

    fn reopen(&mut self, entry: &LogEntry) {
        // A recent review is a whole comparison, so it opens on Compare.
        self.page = Page::Compare;
        if let Some(path) = entry.bundle_path.as_deref().filter(|path| !path.trim().is_empty()) {
            match bundle::open_file(Path::new(path)) {
                Ok(opened) => {
                    self.opened = Some(opened);
                    self.note.clear();
                    return;
                }
                Err(error) => self.note = error,
            }
        }
        if let (Some(left), Some(right)) = (entry.left_path.as_deref(), entry.right_path.as_deref()) {
            match (open_path(Path::new(left)), open_path(Path::new(right))) {
                (Ok(left_loaded), Ok(right_loaded)) => {
                    self.opened = None;
                    let remembered = self.remember_opened_file(&left_loaded) && self.remember_opened_file(&right_loaded);
                    self.left = Some(left_loaded);
                    self.right = Some(right_loaded);
                    self.sitting_key.clear();
                    if remembered {
                        self.note.clear();
                    }
                }
                (Err(error), _) | (_, Err(error)) => self.note = error,
            }
            return;
        }
        if self.note.is_empty() {
            self.note = "That review has no file to reopen.".to_string();
        }
    }
}

#[derive(Clone, Copy)]
struct Paint {
    left: Color32,
    right: Color32,
    mark: Color32,
    note: Color32,
    background: Color32,
    /// Strong text: the headline, the verdict, a tile's title.
    text: Color32,
    label: Option<&'static str>,
}

impl Paint {
    fn from_palette(palette: prefs::SeriesPalette) -> Self {
        Self {
            left: hex_color(palette.left),
            right: hex_color(palette.right),
            mark: hex_color(palette.mark),
            note: hex_color(palette.note),
            background: hex_color(palette.background),
            text: Color32::WHITE,
            label: palette.label,
        }
    }

    /// The palette on a light or dark page. On light, the background and text invert, and the
    /// default and monochrome series darken so they keep their contrast.
    fn themed(palette: prefs::SeriesPalette, light: bool) -> Self {
        let dark = Self::from_palette(palette);
        if !light {
            return dark;
        }
        let (left, right, mark) = match palette.label {
            None => (hex_color("138a83"), hex_color("c83e4d"), hex_color("a86b00")),
            Some("monochrome") => (hex_color("000000"), hex_color("555555"), hex_color("888888")),
            Some("high contrast") => (hex_color("005f87"), hex_color("b00020"), hex_color("6a1b9a")),
            _ => (dark.left, dark.right, dark.mark),
        };
        Self {
            left,
            right,
            mark,
            note: hex_color("4f5566"),
            background: hex_color("f7f7fa"),
            text: hex_color("15171c"),
            label: palette.label,
        }
    }
}

fn hex_color(hex: &str) -> Color32 {
    let number = u32::from_str_radix(hex, 16).unwrap_or(0);
    Color32::from_rgb((number >> 16) as u8, (number >> 8) as u8, number as u8)
}

/// The run names a stored review was saved with.
fn stored_names(review: &bundle::StoredReview) -> Option<(String, String)> {
    match bundle::stored_pair(review)? {
        Pair::Inference(review) => Some((review.left_label, review.right_label)),
        Pair::Training(review) => Some((review.left.run_id, review.right.run_id)),
        Pair::Geometry(review) => Some((review.left_label, review.right_label)),
    }
}

fn side_name(loaded: &Option<Loaded>, empty: &str) -> String {
    let Some(loaded) = loaded else {
        return empty.to_string();
    };
    loaded.side.name()
}

pub fn install_style(cc: &eframe::CreationContext<'_>) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(0x12, 0x12, 0x1f);
    visuals.window_fill = Color32::from_rgb(0x12, 0x12, 0x1f);
    cc.egui_ctx.set_visuals(visuals);
}

#[path = "ui_pages.rs"]
mod pages;

#[path = "ui_geometry.rs"]
mod geometry_views;

#[path = "ui_workbench.rs"]
mod bench_page;

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
