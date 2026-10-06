//! The window. egui_plot draws the readings. It does not decide them.

use std::path::Path;

use eframe::egui::{self, Color32, RichText};
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints, Points, Polygon, VLine};

use crate::bundle::{self, OpenedBundle};
use crate::history::{self, LogEntry};
use crate::open::{open_path, open_paths, Loaded, Side};
use crate::prefs::{self, SavedView};
use crate::readings::Band;
use crate::review::{self, InferenceReview, Pair, TrainingReview};
use crate::views;

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
}

impl View {
    pub const ALL: [(View, &'static str); 5] = [
        (View::Series, "Series"),
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
    /// Set only when this process is the Store package. An unpackaged run leaves LocalState alone.
    history_dir: Option<std::path::PathBuf>,
    recent: Vec<LogEntry>,
    files: Vec<prefs::RecentFile>,
    views: Vec<SavedView>,
    paint: Paint,
    text_scale: f32,
    sitting_key: String,
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
            history_dir,
            recent,
            files: saved.recent,
            views,
            paint: Paint::from_palette(prefs::series_palette(saved.color_vision, saved.high_contrast)),
            text_scale: saved.text_scale,
            sitting_key: String::new(),
        }
    }
}

impl eframe::App for ScalarScopeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The page is taller than the window once throughput and memory are drawn.
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.page(ui));
    }
}

impl ScalarScopeApp {
    fn page(&mut self, ui: &mut egui::Ui) {
        ui.ctx().set_zoom_factor(self.text_scale);
        if self.paint.background != Color32::from_rgb(0x12, 0x12, 0x1f) {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = self.paint.background;
            visuals.window_fill = self.paint.background;
            visuals.extreme_bg_color = self.paint.background;
            ui.ctx().set_visuals(visuals);
        }
        let paint = self.paint;
        ui.horizontal(|ui| {
            ui.heading(RichText::new("ScalarScope").color(paint.mark));
            if ui.button("Open path A").clicked() {
                self.load(true, false);
            }
            if ui.button("Folder A").on_hover_text("Open a run folder for side A").clicked() {
                self.load(true, true);
            }
            if ui.button("Open path B").clicked() {
                self.load(false, false);
            }
            if ui.button("Folder B").on_hover_text("Open a run folder for side B").clicked() {
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

        let opened = self.opened.clone();
        if opened.is_none() {
            let mismatch = match (&self.left, &self.right) {
                (Some(left), Some(right)) => review::pair(&left.side, &right.side).err(),
                _ => None,
            };
            if let Some(error) = mismatch {
                self.note = error;
            }
        }
        if !self.note.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.note).color(paint.mark));
        }

        let built = if self.opened.is_none() {
            match (&self.left, &self.right) {
                (Some(left), Some(right)) => review::pair(&left.side, &right.side).ok(),
                _ => None,
            }
        } else {
            None
        };
        if let Some(pair) = &built {
            let left_path = self.left.as_ref().map(|item| item.path.clone()).unwrap_or_default();
            let right_path = self.right.as_ref().map(|item| item.path.clone()).unwrap_or_default();
            self.remember(&left_path, &right_path, pair);
        }

        ui.add_space(8.0);
        if let Some(opened) = opened {
            ui.label(RichText::new(format!("Stored review · {}", &opened.bundle_hash[..16])).color(paint.note));
            ui.label(RichText::new(bundle::CONTENT_CHECK).color(paint.note));
            ui.add_space(6.0);
            match bundle::stored_pair(&opened.review) {
                Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
                Some(Pair::Training(review)) => draw_training(ui, &review, paint),
                None => {
                    if !opened.review.verdict.is_empty() {
                        ui.label(RichText::new(&opened.review.verdict).color(Color32::WHITE));
                    }
                    ui.label(RichText::new(&opened.review.caption).color(paint.note));
                }
            }
        } else {
            match built {
                Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
                Some(Pair::Training(review)) => draw_training(ui, &review, paint),
                None => {
                    ui.label(
                        RichText::new("Open two inference traces, or two backpropagate run histories. Or open a .scbundle.")
                            .color(paint.note),
                    );
                }
            }
        }
        self.draw_recent(ui);
    }
}

impl ScalarScopeApp {
    fn load(&mut self, left: bool, folder: bool) {
        let picked = if folder { pick_run_folder().map(|path| vec![path]) } else { pick_runs() };
        let Some(paths) = picked.filter(|paths| !paths.is_empty()) else {
            return;
        };
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
        });
        ui.label(RichText::new(&review.left_text).color(paint.note));
        ui.label(RichText::new(&review.right_text).color(paint.note));
        ui.add_space(4.0);
        if !review.headline.is_empty() {
            ui.label(RichText::new(&review.headline).color(Color32::WHITE).strong());
        }
        ui.label(RichText::new(&review.verdict).color(Color32::WHITE));
        for line in &review.notices {
            ui.label(RichText::new(line).color(paint.note));
        }
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
        }
        Plot::new("review")
            .height(360.0)
            .legend(egui_plot::Legend::default())
            .x_axis_label("step")
            .y_axis_label("ms")
            .show(ui, |plot| {
                {
                    for (name, color, band) in [("A spread", paint.left, &review.left_band), ("B spread", paint.right, &review.right_band)] {
                        let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 72);
                        for polygon in band_polygons(band) {
                            plot.polygon(Polygon::new(name, PlotPoints::new(polygon)).fill_color(fill).stroke(egui::Stroke::new(0.0, color)));
                        }
                    }
                    plot.line(series_line("A", paint.left, value_points(&review.left)));
                    plot.line(series_line("B", paint.right, value_points(&review.right)));
                    plot.points(mark_points("A marks", &review.left, &review.left_marks, paint.mark));
                    plot.points(mark_points("B marks", &review.right, &review.right_marks, paint.mark));
                    if let Some(step) = review.left_steady {
                        plot.vline(VLine::new("A steady", step as f64).color(paint.note).width(1.0));
                    }
                    if let Some(step) = review.right_steady {
                        plot.vline(VLine::new("B steady", step as f64).color(paint.note).width(1.0));
                    }
                    for (name, color, segments) in [("A levels", paint.left, &review.left_segments), ("B levels", paint.right, &review.right_segments)] {
                        for segment in segments.iter() {
                            let points = vec![[segment.start as f64, segment.level], [segment.end.saturating_sub(1) as f64, segment.level]];
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
            .color(Color32::WHITE),
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

fn mark_points(name: &str, values: &[Option<f64>], marks: &[usize], color: Color32) -> Points<'static> {
    let points = marks
        .iter()
        .filter_map(|index| values.get(*index).copied().flatten().map(|sample| [*index as f64, sample]))
        .collect::<Vec<_>>();
    Points::new(name, PlotPoints::new(points)).color(color).radius(4.0)
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
                let name = match &loaded.side {
                    Side::Inference(run) => run.label.clone(),
                    Side::Training(entry) => entry.run_id.clone(),
                };
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
        let name = match &loaded.side {
            Side::Inference(run) => run.label.clone(),
            Side::Training(entry) => entry.run_id.clone(),
        };
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
    }
}

fn side_name(loaded: &Option<Loaded>, empty: &str) -> String {
    let Some(loaded) = loaded else {
        return empty.to_string();
    };
    match &loaded.side {
        Side::Inference(run) => run.label.clone(),
        Side::Training(entry) => entry.run_id.clone(),
    }
}

pub fn install_style(cc: &eframe::CreationContext<'_>) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(0x12, 0x12, 0x1f);
    visuals.window_fill = Color32::from_rgb(0x12, 0x12, 0x1f);
    cc.egui_ctx.set_visuals(visuals);
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
