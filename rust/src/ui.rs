//! The window. egui_plot draws the readings. It does not decide them.

use eframe::egui::{self, Color32, RichText};
use egui_plot::{Line, Plot, PlotPoints, Points, Polygon};

use crate::bundle::{self, OpenedBundle};
use crate::open::{open_path, Loaded, Side};
use crate::readings::Band;
use crate::review::{self, InferenceReview, Pair, TrainingReview};

const LEFT: Color32 = Color32::from_rgb(0x4e, 0xcd, 0xc4);
const RIGHT: Color32 = Color32::from_rgb(0xff, 0x6b, 0x6b);
const MARK: Color32 = Color32::from_rgb(0xff, 0xd9, 0x3d);
const NOTE: Color32 = Color32::from_rgb(0x9a, 0xa0, 0xb4);

pub struct ScalarScopeApp {
    left: Option<Loaded>,
    right: Option<Loaded>,
    opened: Option<OpenedBundle>,
    note: String,
    distribution: bool,
}

impl Default for ScalarScopeApp {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
            opened: None,
            note: String::new(),
            distribution: false,
        }
    }
}

impl eframe::App for ScalarScopeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("ScalarScope").color(MARK));
            if ui.button("Open path A").clicked() {
                self.load(true);
            }
            if ui.button("Open path B").clicked() {
                self.load(false);
            }
            if ui.button("Open bundle").clicked() {
                self.open_bundle();
            }
            if self.opened.is_none() && ui.button("Save bundle").clicked() {
                self.save_bundle();
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(side_name(&self.left, "Path A")).color(LEFT));
            ui.label("vs");
            ui.label(RichText::new(side_name(&self.right, "Path B")).color(RIGHT));
        });

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
            ui.label(RichText::new(&self.note).color(MARK));
        }

        ui.add_space(8.0);
        if let Some(opened) = opened {
            ui.label(RichText::new(format!("Stored review · {}", &opened.bundle_hash[..16])).color(NOTE));
            ui.label(RichText::new(bundle::CONTENT_CHECK).color(NOTE));
            ui.add_space(6.0);
            match bundle::stored_pair(&opened.review) {
                Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
                Some(Pair::Training(review)) => draw_training(ui, &review),
                None => {
                    if !opened.review.verdict.is_empty() {
                        ui.label(RichText::new(&opened.review.verdict).color(Color32::WHITE));
                    }
                    ui.label(RichText::new(&opened.review.caption).color(NOTE));
                }
            }
            return;
        }

        let built = match (&self.left, &self.right) {
            (Some(left), Some(right)) => review::pair(&left.side, &right.side).ok(),
            _ => None,
        };
        match built {
            Some(Pair::Inference(review)) => self.draw_inference(ui, &review),
            Some(Pair::Training(review)) => draw_training(ui, &review),
            None => {
                ui.label(
                    RichText::new("Open two inference traces, or two backpropagate run histories. Or open a .scbundle.")
                        .color(NOTE),
                );
            }
        }
    }
}

impl ScalarScopeApp {
    fn load(&mut self, left: bool) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Run", &["json", "csv", "log"])
            .pick_file()
        else {
            return;
        };
        match open_path(&path) {
            Ok(loaded) => {
                self.opened = None;
                if left {
                    self.left = Some(loaded);
                } else {
                    self.right = Some(loaded);
                }
                self.note.clear();
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
        let Some(path) = rfd::FileDialog::new()
            .add_filter("ScalarScope bundle", &["scbundle"])
            .pick_file()
        else {
            return;
        };
        match bundle::open_file(&path) {
            Ok(opened) => {
                self.opened = Some(opened);
                self.note.clear();
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
        let Some(path) = rfd::FileDialog::new()
            .add_filter("ScalarScope bundle", &["scbundle"])
            .set_file_name("review.scbundle")
            .save_file()
        else {
            return;
        };
        let document = bundle::document_from_pair(&built);
        match bundle::seal(&document, &bundle::utc_now()) {
            Ok(sealed) => match bundle::write_file(&path, &sealed) {
                Ok(()) => {
                    self.note = format!("Saved the stored review. Content check {}.", sealed.bundle_hash);
                }
                Err(error) => self.note = error,
            },
            Err(error) => self.note = error,
        }
    }

    fn draw_inference(&mut self, ui: &mut egui::Ui, review: &InferenceReview) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Series").color(LEFT));
            ui.toggle_value(&mut self.distribution, "Distribution");
        });
        ui.label(RichText::new(&review.left_text).color(NOTE));
        ui.label(RichText::new(&review.right_text).color(NOTE));
        ui.add_space(4.0);
        ui.label(RichText::new(&review.verdict).color(Color32::WHITE));
        let distribution = self.distribution;
        Plot::new("review")
            .height(360.0)
            .legend(egui_plot::Legend::default())
            .x_axis_label(if distribution { "latency_ms" } else { "step" })
            .y_axis_label(if distribution { "empirical CDF" } else { "ms" })
            .show(ui, |plot| {
                if distribution {
                    plot.line(series_line("A distribution", LEFT, cdf_points(&review.left_cdf)));
                    plot.line(series_line("B distribution", RIGHT, cdf_points(&review.right_cdf)));
                } else {
                    for (name, color, band) in [("A spread", LEFT, &review.left_band), ("B spread", RIGHT, &review.right_band)] {
                        for (index, polygon) in band_polygons(band).into_iter().enumerate() {
                            let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 72);
                            plot.polygon(Polygon::new(format!("{name} {index}"), PlotPoints::new(polygon)).fill_color(fill).width(0.0));
                        }
                    }
                    plot.line(series_line("A", LEFT, value_points(&review.left)));
                    plot.line(series_line("B", RIGHT, value_points(&review.right)));
                    plot.points(mark_points("A marks", &review.left, &review.left_marks));
                    plot.points(mark_points("B marks", &review.right, &review.right_marks));
                }
            });
        if !distribution && !review.left_throughput.is_empty() && !review.right_throughput.is_empty() {
            ui.label(RichText::new("Throughput, items/s, same steps, own scale.").color(NOTE));
            Plot::new("throughput")
                .height(140.0)
                .y_axis_label("items/s")
                .show(ui, |plot| {
                    plot.line(series_line("A throughput", LEFT, plain_points(&review.left_throughput)));
                    plot.line(series_line("B throughput", RIGHT, plain_points(&review.right_throughput)));
                });
        }
        ui.add_space(6.0);
        ui.label(RichText::new(&review.caption).color(NOTE));
    }
}

fn draw_training(ui: &mut egui::Ui, review: &TrainingReview) {
    ui.label(RichText::new(&review.left_text).color(LEFT));
    ui.label(RichText::new(&review.right_text).color(RIGHT));
    Plot::new("training-loss")
        .height(420.0)
        .legend(egui_plot::Legend::default())
        .x_axis_label("stored sample")
        .y_axis_label("training loss")
        .show(ui, |plot| {
            plot.line(series_line(&review.left.run_id, LEFT, plain_points(&review.left.loss)));
            plot.line(series_line(&review.right.run_id, RIGHT, plain_points(&review.right.loss)));
        });
    ui.add_space(6.0);
    ui.label(RichText::new(&review.caption).color(NOTE));
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

fn mark_points(name: &str, values: &[Option<f64>], marks: &[usize]) -> Points<'static> {
    let points = marks
        .iter()
        .filter_map(|index| values.get(*index).copied().flatten().map(|sample| [*index as f64, sample]))
        .collect::<Vec<_>>();
    Points::new(name, PlotPoints::new(points)).color(MARK).radius(4.0)
}

fn band_polygons(band: &[Option<Band>]) -> Vec<Vec<[f64; 2]>> {
    let mut polygons = Vec::new();
    let mut start = None;
    for (index, point) in band.iter().enumerate() {
        if point.is_some() && start.is_none() {
            start = Some(index);
        }
        let ended = point.is_none() || index + 1 == band.len();
        if ended {
            if let Some(from) = start.take() {
                let to = if point.is_some() { index } else { index - 1 };
                if to > from {
                    let mut polygon = Vec::new();
                    for cursor in from..=to {
                        if let Some(sample) = band[cursor] {
                            polygon.push([cursor as f64, sample.low]);
                        }
                    }
                    for cursor in (from..=to).rev() {
                        if let Some(sample) = band[cursor] {
                            polygon.push([cursor as f64, sample.high]);
                        }
                    }
                    polygons.push(polygon);
                }
            }
        }
        if point.is_none() {
            start = None;
        }
    }
    polygons
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
