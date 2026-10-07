//! The geometry page: two ASPIRE training-dynamics runs as static views with one time scrub.
//!
//! 2.0 animated these. Animation is the least accurate way to compare (V7), so here every view is
//! static and a single slider moves a marker through all of them at once.

use eframe::egui::{self, Color32, RichText};
use egui_plot::{Arrows, Line, LineStyle, Plot, PlotPoint, PlotPoints, Points, Text, VLine};

use super::{series_line, ScalarScopeApp};
use crate::geometry::GeometryRun;
use crate::review::GeometryReview;

/// The time range both runs cover.
pub fn time_range(left: &GeometryRun, right: &GeometryRun) -> Option<(f64, f64)> {
    let times = left.trajectory.timesteps.iter().chain(&right.trajectory.timesteps).map(|step| step.t).filter(|t| t.is_finite());
    let (low, high) = times.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), t| (low.min(t), high.max(t)));
    low.is_finite().then_some((low, high))
}

/// The trajectory point nearest `t`.
pub fn position_at(run: &GeometryRun, t: f64) -> Option<[f64; 2]> {
    run.trajectory
        .timesteps
        .iter()
        .filter(|step| step.state_2d.len() >= 2)
        .min_by(|a, b| (a.t - t).abs().total_cmp(&(b.t - t).abs()))
        .map(|step| [step.state_2d[0], step.state_2d[1]])
}

/// The largest distance of the trajectory from the origin, to scale the professor arrows.
pub fn reach(run: &GeometryRun) -> f64 {
    run.trajectory
        .timesteps
        .iter()
        .filter(|step| step.state_2d.len() >= 2)
        .map(|step| (step.state_2d[0].powi(2) + step.state_2d[1].powi(2)).sqrt())
        .fold(0.0, f64::max)
}

/// A professor's arrow from the origin, scaled so the longest arrow reaches `length`.
pub fn professor_tips(run: &GeometryRun, length: f64) -> Vec<(String, bool, [f64; 2])> {
    let longest = run
        .evaluators
        .professors
        .iter()
        .filter(|professor| professor.vector.len() >= 2)
        .map(|professor| (professor.vector[0].powi(2) + professor.vector[1].powi(2)).sqrt())
        .fold(0.0, f64::max);
    if longest <= 0.0 {
        return Vec::new();
    }
    run.evaluators
        .professors
        .iter()
        .filter(|professor| professor.vector.len() >= 2)
        .map(|professor| {
            let scale = length / longest;
            (professor.name.clone(), professor.holdout, [professor.vector[0] * scale, professor.vector[1] * scale])
        })
        .collect()
}

/// The export's layout, as the header states it (spec, "Geometry export contract").
fn layout_text(meta: &crate::geometry::RunMetadata) -> String {
    let mut out = String::new();
    if !meta.steps_are_time() {
        match meta.checkpoints {
            Some(count) => out.push_str(&format!(" · steps: {count} checkpoints × items, not time")),
            None => out.push_str(" · steps: checkpoints × items, not time"),
        }
    }
    match meta.repeated_scores() {
        Some("replayed") => out.push_str(" · scores replayed after the first epoch"),
        Some(_) => out.push_str(" · scores fixed per item"),
        None => {}
    }
    out
}

/// What a score panel's title adds when a run's scores repeat.
fn score_note(left: &GeometryRun, right: &GeometryRun) -> String {
    let sources: Vec<&str> = [left.metadata.repeated_scores(), right.metadata.repeated_scores()].into_iter().flatten().collect();
    if sources.contains(&"replayed") {
        " · replayed after the first epoch".to_string()
    } else if sources.is_empty() {
        String::new()
    } else {
        " · fixed per item".to_string()
    }
}

/// The steps split into equal checkpoint blocks, or one block when the count is not stated.
fn checkpoint_blocks(path: &[[f64; 2]], checkpoints: Option<i64>) -> Vec<Vec<[f64; 2]>> {
    let count = checkpoints.filter(|count| *count > 0).unwrap_or(1) as usize;
    let size = path.len().div_ceil(count).max(1);
    path.chunks(size).map(<[[f64; 2]]>::to_vec).collect()
}

/// Review mode: today's verdict, when it differs from the verdict the bundle stored.
pub(super) fn current_reading_note(current: &str) -> String {
    format!("Current reading (rules since 3.1.1): '{current}'. This bundle was saved with the reading above.")
}

/// Review mode: the bundle stored only its verdict, so the tiles come from today's rules.
pub(super) const CURRENT_TILES: &str = "Current reading: this bundle stored no tiles, so these are drawn by today's rules.";

/// A label position moved down until its line clears every label already placed. A label is
/// wide and short, so two clash when they are within `gap` vertically and eight gaps sideways.
fn clear_of(placed: &[[f64; 2]], mut at: [f64; 2], gap: f64) -> [f64; 2] {
    for _ in 0..12 {
        if placed.iter().all(|other| (other[1] - at[1]).abs() >= gap || (other[0] - at[0]).abs() >= gap * 8.0) {
            break;
        }
        at[1] -= gap;
    }
    at
}

fn over_time(run: &GeometryRun, value: impl Fn(&crate::geometry::Timestep) -> f64) -> Vec<[f64; 2]> {
    run.trajectory.timesteps.iter().map(|step| [step.t, value(step)]).filter(|[t, v]| t.is_finite() && v.is_finite()).collect()
}

impl ScalarScopeApp {
    pub(super) fn draw_geometry(&mut self, ui: &mut egui::Ui, review: &GeometryReview) {
        let paint = self.paint;
        for (run, color) in [(&review.left, paint.left), (&review.right, paint.right)] {
            let meta = &run.metadata;
            let holdout = meta.holdout_professor.as_deref().map(|name| format!(" · holdout {name}")).unwrap_or_default();
            ui.label(
                RichText::new(format!(
                    "{} · {} · {} timesteps · {} cycles · conscience tier {}{holdout}{}",
                    run.name(),
                    if meta.condition.is_empty() { "no condition" } else { meta.condition.as_str() },
                    run.trajectory.timesteps.len(),
                    meta.cycles,
                    meta.conscience_tier,
                    layout_text(meta)
                ))
                .color(color),
            );
        }
        for warning in &review.warnings {
            ui.label(RichText::new(warning).color(paint.note));
        }
        ui.label(RichText::new(&review.verdict).color(paint.text));
        if let Some(current) = &review.current_verdict {
            ui.label(RichText::new(current_reading_note(current)).color(paint.mark));
        }
        if review.tiles_are_current {
            ui.label(RichText::new(CURRENT_TILES).color(paint.note));
        }
        self.draw_tiles(ui, &review.explanations);
        let Some((start, end)) = time_range(&review.left, &review.right) else {
            ui.label(RichText::new("Neither run has a trajectory to draw.").color(paint.note));
            return;
        };
        let mut t = self.scrub.unwrap_or(end).clamp(start, end);
        ui.horizontal(|ui| {
            ui.label("Time");
            if ui.add(egui::Slider::new(&mut t, start..=end)).on_hover_text("Moves the marker through every view; nothing moves on its own").changed() {
                self.scrub = Some(t);
            }
        });

        // The trajectory, with the professors.
        let ordered = review.left.metadata.steps_are_time() && review.right.metadata.steps_are_time();
        ui.label(
            RichText::new(if ordered {
                "Trajectory in the run's 2-D projection, with each evaluator's direction. A fainter arrow is a held-out evaluator."
            } else {
                "Points in the run's 2-D projection, with each evaluator's direction. Unordered: a run's steps are checkpoint × item, not time, so the points are not joined; each checkpoint is one shade, darker for later ones."
            })
            .color(paint.note),
        );
        let length = reach(&review.left).max(reach(&review.right)).max(1e-9);
        // Label positions placed so far, so two evaluators that point the same way do not overlap.
        let mut placed: Vec<[f64; 2]> = Vec::new();
        Plot::new("geometry-trajectory")
            .height(340.0)
            .data_aspect(1.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot| {
                for (name, color, run) in [(&review.left_label, paint.left, &review.left), (&review.right_label, paint.right, &review.right)] {
                    let path: Vec<[f64; 2]> = run.trajectory.timesteps.iter().filter(|step| step.state_2d.len() >= 2).map(|step| [step.state_2d[0], step.state_2d[1]]).collect();
                    if run.metadata.steps_are_time() {
                        plot.line(series_line(name, color, path.clone()));
                    } else {
                        for (block, points) in checkpoint_blocks(&path, run.metadata.checkpoints).into_iter().enumerate() {
                            let count = run.metadata.checkpoints.unwrap_or(1).max(1) as usize;
                            let alpha = 90 + (165 * (block + 1) / count.max(1)).min(165) as u8;
                            let shade = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
                            plot.points(Points::new(format!("{name} checkpoint {}", block + 1), PlotPoints::new(points)).color(shade).radius(2.5));
                        }
                    }
                    if let Some(first) = path.first().filter(|_| run.metadata.steps_are_time()) {
                        plot.points(Points::new("start", PlotPoints::new(vec![*first])).color(color).radius(3.0));
                    }
                    if let Some(now) = position_at(run, t) {
                        plot.points(Points::new("at the chosen time", PlotPoints::new(vec![now])).color(paint.mark).radius(6.0));
                    }
                    for (professor, holdout, tip) in professor_tips(run, length) {
                        let faded = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), if holdout { 110 } else { 200 });
                        let legend = format!("{name} evaluators");
                        plot.arrows(Arrows::new(legend.clone(), PlotPoints::new(vec![[0.0, 0.0]]), PlotPoints::new(vec![tip])).color(faded).tip_length(10.0));
                        let label = if holdout { format!("{professor} (held out)") } else { professor };
                        let at = clear_of(&placed, [tip[0] * 1.06, tip[1] * 1.06], length * 0.06);
                        placed.push(at);
                        plot.text(Text::new(legend, PlotPoint::new(at[0], at[1]), RichText::new(label).color(faded).small()));
                    }
                }
            });

        // Small multiples over time, each with the scrub line.
        let mut panels: Vec<(String, Vec<[f64; 2]>, Vec<[f64; 2]>)> = Vec::new();
        let dimensions = {
            let mut names = review.left.dimension_names();
            for name in review.right.dimension_names() {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            names
        };
        for dimension in &dimensions {
            let scores = |run: &GeometryRun| run.scalars.values.iter().map(|step| [step.t, step.score(dimension)]).collect::<Vec<_>>();
            panels.push((format!("{dimension} (evaluator score){}", score_note(&review.left, &review.right)), scores(&review.left), scores(&review.right)));
        }
        panels.push(("speed".to_string(), over_time(&review.left, |step| step.speed()), over_time(&review.right, |step| step.speed())));
        panels.push(("curvature".to_string(), over_time(&review.left, |step| step.curvature), over_time(&review.right, |step| step.curvature)));
        panels.push(("effective dimension".to_string(), over_time(&review.left, |step| step.effective_dim), over_time(&review.right, |step| step.effective_dim)));
        let anisotropy = |run: &GeometryRun| run.geometry.anisotropy.iter().map(|step| [step.t, step.ratio]).collect::<Vec<_>>();
        panels.push(("anisotropy (λ1 / λ2)".to_string(), anisotropy(&review.left), anisotropy(&review.right)));
        // The page width is read before the grid: inside a cell it is only the cell's.
        let width = (ui.available_width() / 2.0 - 18.0).max(240.0);
        egui::Grid::new("geometry-panels").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            for (index, (title, left, right)) in panels.iter().enumerate() {
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).color(paint.note));
                    Plot::new(format!("geometry-panel-{index}"))
                        .height(130.0)
                        .width(width)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .show(ui, |plot| {
                            plot.line(series_line(&review.left_label, paint.left, left.clone()));
                            plot.line(series_line(&review.right_label, paint.right, right.clone()));
                            plot.vline(VLine::new("time", t).color(paint.mark).width(1.0));
                        });
                });
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });

        // The eigen spectrum: each eigenvalue over time, A solid and B dashed.
        ui.label(RichText::new("Eigen spectrum over time: each line is one eigenvalue of the state covariance; solid is A, dashed is B.").color(paint.note));
        Plot::new("geometry-eigen").height(200.0).show(ui, |plot| {
            for (name, color, run, style) in [
                (&review.left_label, paint.left, &review.left, LineStyle::Solid),
                (&review.right_label, paint.right, &review.right, LineStyle::dashed_loose()),
            ] {
                let count = run.geometry.eigenvalues.iter().map(|step| step.values.len()).max().unwrap_or(0);
                for rank in 0..count {
                    let points: Vec<[f64; 2]> = run.geometry.eigenvalues.iter().filter_map(|step| step.values.get(rank).map(|value| [step.t, *value])).collect();
                    plot.line(Line::new(format!("{name} λ{}", rank + 1), PlotPoints::new(points)).color(color).style(style).width(1.5));
                }
            }
            plot.vline(VLine::new("time", t).color(paint.mark).width(1.0));
        });

        // Failures on a shared timeline.
        let failures: Vec<(String, Color32, &crate::geometry::Failure, f64)> = [(&review.left_label, paint.left, &review.left, 1.0), (&review.right_label, paint.right, &review.right, 0.0)]
            .into_iter()
            .flat_map(|(name, color, run, row)| run.failures.iter().map(move |failure| (name.clone(), color, failure, row)))
            .collect();
        if failures.is_empty() {
            ui.label(RichText::new("Neither run recorded a failure.").color(paint.note));
        } else {
            ui.label(RichText::new("Failures over time: A on the top row, B on the bottom. Larger marks are more severe.").color(paint.note));
            Plot::new("geometry-failures")
                .height(90.0)
                .show_axes([true, false])
                .include_y(-0.5)
                .include_y(1.5)
                .allow_drag(false)
                .show(ui, |plot| {
                    for (name, color, failure, row) in &failures {
                        let radius = match failure.severity.to_ascii_uppercase().as_str() {
                            "HIGH" | "CRITICAL" => 7.0,
                            "MEDIUM" => 5.0,
                            _ => 3.5,
                        };
                        plot.points(Points::new(format!("{name} failures"), PlotPoints::new(vec![[failure.t, *row]])).color(*color).radius(radius));
                    }
                    plot.vline(VLine::new("time", t).color(paint.mark).width(1.0));
                });
            for (name, color, failure, _) in &failures {
                ui.label(RichText::new(format!("{name} · t {:.2} · {} · {} · {}", failure.t, failure.severity, failure.category, failure.description)).color(*color));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> GeometryRun {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/ScalarScope/Resources/Raw/Samples").join(name);
        crate::geometry::read(&serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()).unwrap().run
    }

    #[test]
    fn the_samples_give_a_time_range_positions_and_arrows() {
        let (left, right) = (sample("orthogonal_professors.json"), sample("correlated_professors.json"));
        let (start, end) = time_range(&left, &right).unwrap();
        assert!(start < end);
        assert!(position_at(&left, start).is_some());
        let tips = professor_tips(&left, 2.0);
        assert_eq!(tips.len(), 3);
        let longest = tips.iter().map(|(_, _, [x, y])| (x * x + y * y).sqrt()).fold(0.0, f64::max);
        assert!((longest - 2.0).abs() < 1e-9);
        assert!(reach(&left) > 0.0);
        assert!(professor_tips(&GeometryRun::default(), 1.0).is_empty());
        assert!(time_range(&GeometryRun::default(), &GeometryRun::default()).is_none());
    }

    #[test]
    fn the_layout_is_stated_and_drawn_as_the_contract_says() {
        let mut meta = crate::geometry::RunMetadata::default();
        assert_eq!(layout_text(&meta), "");
        meta.step_axis = Some("checkpoint_by_item".into());
        meta.checkpoints = Some(3);
        meta.scalar_source = Some("fixed_per_item".into());
        assert_eq!(layout_text(&meta), " · steps: 3 checkpoints × items, not time · scores fixed per item");
        meta.checkpoints = None;
        meta.scalar_source = Some("replayed".into());
        assert_eq!(layout_text(&meta), " · steps: checkpoints × items, not time · scores replayed after the first epoch");

        let mut left = GeometryRun::default();
        let right = GeometryRun::default();
        assert_eq!(score_note(&left, &right), "");
        left.metadata.scalar_source = Some("fixed_per_item".into());
        assert_eq!(score_note(&left, &right), " · fixed per item");
        left.metadata.scalar_source = Some("replayed".into());
        assert_eq!(score_note(&right, &left), " · replayed after the first epoch");

        let path: Vec<[f64; 2]> = (0..7).map(|i| [i as f64, 0.0]).collect();
        assert_eq!(checkpoint_blocks(&path, Some(3)).iter().map(Vec::len).collect::<Vec<_>>(), vec![3, 3, 1]);
        assert_eq!(checkpoint_blocks(&path, None).len(), 1);
        assert_eq!(checkpoint_blocks(&path, Some(0)).len(), 1);
    }

    #[test]
    fn a_label_close_to_another_moves_clear_of_it() {
        let placed = vec![[1.0, 1.0]];
        assert_eq!(clear_of(&placed, [3.0, 3.0], 0.5), [3.0, 3.0]);
        let moved = clear_of(&placed, [1.05, 1.0], 0.5);
        assert!(moved[1] < 1.0 && (moved[0] - 1.05).abs() < 1e-12);
        assert!((moved[1] - 1.0).abs() >= 0.5);
        // A long label beside another on the same line also moves; one far to the side does not.
        assert!(clear_of(&placed, [2.5, 1.1], 0.5)[1] < 1.1);
        assert_eq!(clear_of(&placed, [6.0, 1.1], 0.5), [6.0, 1.1]);
    }

}
