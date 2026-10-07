//! The History page: one project's measure across its logged reviews, with the program's change
//! points and the code or environment changes beside them.

use eframe::egui::{self, RichText};
use egui_plot::{Line, LineStyle, Plot, PlotPoints, Points, VLine};

use crate::history;
use crate::trends::{self, Project};

use super::ScalarScopeApp;

/// What the History page has chosen.
#[derive(Default)]
pub struct HistoryState {
    pub project: usize,
    pub measure: usize,
}

impl ScalarScopeApp {
    pub(super) fn draw_history(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        ui.heading(RichText::new("History").color(paint.text));
        ui.label(
            RichText::new(
                "How a project's measures moved across its logged reviews. A project is side B's dataset and model. \
The program finds where a measure shifts, and marks the reviews where the code or the environment changed. A shift beside a change is a coincidence the page names, not a cause it proves.",
            )
            .color(paint.note),
        );
        let Some(dir) = self.history_dir.clone() else {
            ui.label(RichText::new("This build is not the Store package, so there is no comparison log to read.").color(paint.note));
            return;
        };
        let projects: Vec<Project> = trends::projects(&history::read(&dir));
        self.draw_history_projects(ui, &projects);
    }

    pub(super) fn draw_history_projects(&mut self, ui: &mut egui::Ui, projects: &[Project]) {
        let paint = self.paint;
        if projects.is_empty() {
            ui.label(RichText::new("No inference reviews are logged yet. Each comparison you finish is added.").color(paint.note));
            return;
        }
        self.history.project = self.history.project.min(projects.len() - 1);
        self.history.measure = self.history.measure.min(trends::MEASURES.len() - 1);
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Project")
                .selected_text(format!("{} ({} reviews)", projects[self.history.project].label, projects[self.history.project].entries.len()))
                .show_ui(ui, |ui| {
                    for (index, project) in projects.iter().enumerate() {
                        ui.selectable_value(&mut self.history.project, index, format!("{} ({} reviews)", project.label, project.entries.len()));
                    }
                });
            egui::ComboBox::from_label("Measure").selected_text(trends::MEASURES[self.history.measure].1).show_ui(ui, |ui| {
                for (index, (_, label)) in trends::MEASURES.iter().enumerate() {
                    ui.selectable_value(&mut self.history.measure, index, *label);
                }
            });
        });
        let project = &projects[self.history.project];
        let (measure, label) = trends::MEASURES[self.history.measure];
        let trend = trends::trend(project, measure);
        if trend.points.is_empty() {
            ui.label(RichText::new(format!("No review in this project recorded {label}. Reviews logged by 2.0 carry no measures.")).color(paint.note));
            return;
        }
        let points: Vec<[f64; 2]> = trend.points.iter().map(|(at, value)| [*at as f64, *value]).collect();
        Plot::new("history-plot").height(260.0).x_axis_label("review, oldest first").y_axis_label(label).show(ui, |plot| {
            plot.line(Line::new(label, PlotPoints::from(points.clone())).color(paint.right));
            plot.points(Points::new("reviews", PlotPoints::from(points)).radius(3.0).color(paint.right));
            for shift in &trend.shifts {
                plot.vline(VLine::new("shift", *shift as f64 - 0.5).color(paint.mark).width(2.0));
            }
            for mark in &trend.marks {
                plot.vline(VLine::new(format!("{} change", mark.what), mark.at as f64 - 0.5).color(paint.note).style(LineStyle::dashed_loose()));
            }
        });
        ui.label(
            RichText::new(format!(
                "Solid lines: where the program finds a new level (PELT, mean shift on the logarithm, penalty {} · ln n, segments of at least {} reviews). Dashed lines: a code or environment change.",
                trends::PENALTY,
                trends::MIN_SEGMENT
            ))
            .color(paint.note),
        );
        if !trend.searched {
            ui.label(RichText::new(format!("Too few reviews for change points: {} of the {} needed.", trend.points.len(), trends::MIN_REVIEWS)).color(paint.note));
        } else if trend.sentences.is_empty() {
            ui.label(RichText::new(format!("{label} held one level across these reviews.")).color(paint.text));
        }
        for sentence in &trend.sentences {
            ui.label(RichText::new(sentence).color(paint.text));
        }
    }
}
