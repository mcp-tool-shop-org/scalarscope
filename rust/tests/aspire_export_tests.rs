//! The contract with aspire-si's training-dynamics export (`aspire/geometry.py`). The fixtures
//! are what its `examples/geometry_demo.py` writes: two simulated runs with every ASPIRE
//! evaluation dimension, two teachers, and windowed eigenvalues on every step. When the
//! exporter changes, rerun the demo and copy its two files here.

use std::path::PathBuf;

use scalarscope::geometry_deltas::DeltaStatus;
use scalarscope::open::{open_path, Side};
use scalarscope::review::{pair, Pair};

fn export(name: &str) -> Side {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aspire-si").join(name);
    open_path(&path).unwrap_or_else(|error| panic!("{name}: {error}")).side
}

#[test]
fn an_aspire_si_export_opens_in_full_without_warnings() {
    let Side::Geometry(side) = export("steady.json") else { panic!("not read as a geometry export") };
    assert!(side.warnings.is_empty(), "{:?}", side.warnings);
    let run = &side.run;
    assert_eq!(run.name(), "steady");
    assert_eq!(run.dimension_names().len(), 9, "every ASPIRE dimension, not 2.0's five");
    assert!(run.dimension_names().contains(&"intellectual_honesty".to_string()));
    let steps = run.trajectory.timesteps.len();
    assert_eq!(steps, 60);
    // One eigenvalue row per step, so the step-indexed rules read the whole run.
    assert_eq!(run.geometry.eigenvalues.len(), steps);
    assert_eq!(run.geometry.anisotropy.len(), steps);
    for row in &run.geometry.eigenvalues {
        let total: f64 = row.values.iter().sum();
        assert!((total - 1.0).abs() < 1e-9 || total == 0.0, "eigenvalues are fractions: {total}");
    }
    assert_eq!(run.evaluators.professors.len(), 2);
    assert!(run.evaluators.professors.iter().any(|professor| professor.holdout));
}

#[test]
fn two_aspire_si_runs_compare_with_all_five_deltas_decided() {
    let built = pair(&export("steady.json"), &export("regressing.json")).unwrap();
    let Pair::Geometry(review) = built else { panic!("geometry") };
    assert_eq!(review.deltas.len(), 5);
    assert_eq!(review.explanations.len(), 5);
    // The regressing run has several recorded dips; the steady one has one. 2.0's failure rule
    // needs three, so only one side fails.
    let failure = review.deltas.iter().find(|delta| delta.symbol == "ΔF").expect("ΔF");
    assert_eq!(failure.status, DeltaStatus::Present, "{}", failure.explanation);
    assert!(!review.verdict.is_empty());
    for delta in &review.deltas {
        assert!(delta.left_value.is_finite() && delta.right_value.is_finite(), "{}", delta.id);
    }
}

// Real exports from aspire-si training runs (2026-10-07, aspire-si 0972ef5): Qwen2.5-1.5B-Instruct
// with LoRA, 32 prompts × 3 epochs, against a local Qwen2.5-32B teacher or a composite of it and
// Gemma 4 31B. The `.drift` files are each checkpoint's hidden state minus the base student's, per
// prompt, checkpoint-major. Their steps are checkpoint × prompt, not time.

fn geometry(name: &str) -> scalarscope::open::GeometrySide {
    let Side::Geometry(side) = export(name) else { panic!("{name}: not read as a geometry export") };
    side
}

#[test]
fn the_real_exports_open_in_full_without_warnings() {
    for (name, professors) in [
        ("real-local-teacher.geometry.json", 1),
        ("real-composite-teacher.geometry.json", 2),
        ("real-local-teacher.drift.geometry.json", 1),
        ("real-composite-teacher.drift.geometry.json", 2),
    ] {
        let side = geometry(name);
        assert!(side.warnings.is_empty(), "{name}: {:?}", side.warnings);
        let run = &side.run;
        assert_eq!(run.trajectory.timesteps.len(), 96, "{name}");
        assert_eq!(run.geometry.eigenvalues.len(), 96, "{name}");
        assert_eq!(run.dimension_names().len(), 9, "{name}");
        assert_eq!(run.evaluators.professors.len(), professors, "{name}");
        for step in &run.trajectory.timesteps {
            assert!(step.curvature.is_finite() && step.state_2d.iter().all(|value| value.is_finite()), "{name}");
        }
    }
}

#[test]
fn the_real_pairs_compare_as_the_runs_say() {
    use scalarscope::geometry_deltas::{compute, Alignment, DeltaConfig};
    let delta = |deltas: &[scalarscope::geometry_deltas::GeometryDelta], symbol: &str| {
        deltas.iter().find(|delta| delta.symbol == symbol).unwrap().clone()
    };
    // Per step: the trajectory follows which prompt each step drew, so nothing settles.
    let (local, composite) = (geometry("real-local-teacher.geometry.json").run, geometry("real-composite-teacher.geometry.json").run);
    let deltas = compute(&local, &composite, Alignment::ByStep, 1.0, &DeltaConfig::default());
    assert_eq!(deltas.len(), 5);
    assert_eq!(delta(&deltas, "ΔTc").status, DeltaStatus::Suppressed);
    // The composite's two disagreeing teachers concentrate its evaluator spectrum.
    let concentration = delta(&deltas, "Δ\u{0100}");
    assert_eq!(concentration.status, DeltaStatus::Present);
    assert!(concentration.right_value > concentration.left_value, "{:?}", concentration.explanation);

    // Drift: the same reading of the spectrum holds.
    let (local, composite) = (geometry("real-local-teacher.drift.geometry.json").run, geometry("real-composite-teacher.drift.geometry.json").run);
    let deltas = compute(&local, &composite, Alignment::ByStep, 1.0, &DeltaConfig::default());
    let concentration = delta(&deltas, "Δ\u{0100}");
    assert_eq!(concentration.status, DeltaStatus::Present);
    assert!(concentration.right_value > concentration.left_value);
    for delta in &deltas {
        assert!(delta.left_value.is_finite() && delta.right_value.is_finite(), "{}", delta.id);
    }
}

// The export contract (spec, "Geometry export contract"): the schema 1.1 fixtures state their
// step axis and score source, and the review withholds what the layout invalidates.

fn reviewed(left: &str, right: &str) -> scalarscope::review::GeometryReview {
    let Pair::Geometry(review) = pair(&export(left), &export(right)).unwrap() else { panic!("geometry") };
    review
}

fn status<'a>(review: &'a scalarscope::review::GeometryReview, symbol: &str) -> &'a scalarscope::review::Explanation {
    review.explanations.iter().find(|tile| tile.symbol == symbol).unwrap()
}

#[test]
fn the_fixtures_state_their_layout() {
    let per_step = geometry("real-local-teacher.geometry.json").run.metadata;
    assert_eq!((per_step.step_axis.as_deref(), per_step.scalar_source.as_deref()), (Some("training_step"), Some("replayed")));
    assert!(per_step.steps_are_time());
    let drift = geometry("real-local-teacher.drift.geometry.json").run.metadata;
    assert_eq!((drift.step_axis.as_deref(), drift.checkpoints, drift.scalar_source.as_deref()), (Some("checkpoint_by_item"), Some(3), Some("fixed_per_item")));
    assert!(!drift.steps_are_time());
}

#[test]
fn drift_withholds_timing_oscillation_and_failure_and_speaks_from_what_stands() {
    let review = reviewed("real-local-teacher.drift.geometry.json", "real-composite-teacher.drift.geometry.json");
    for symbol in ["ΔTc", "ΔO", "ΔF"] {
        let tile = status(&review, symbol);
        assert_eq!(tile.status, "withheld", "{symbol}");
        assert!(tile.why.contains("steps are checkpoint × item, not time"), "{symbol}: {}", tile.why);
        assert!(tile.why.contains("2.0's reading was"), "{symbol}");
    }
    assert_eq!(status(&review, "Δ\u{0100}").status, "fired");
    assert_eq!(review.verdict, "Path B had a more concentrated spectrum (sustained 31 steps)");
}

#[test]
fn replayed_scores_withhold_the_failure_reading_only() {
    let review = reviewed("real-local-teacher.geometry.json", "real-composite-teacher.geometry.json");
    let failure = status(&review, "ΔF");
    assert_eq!(failure.status, "withheld");
    assert!(failure.why.contains("the scores repeat (replayed after the first epoch)"), "{}", failure.why);
    // Steps are time here, so oscillation still reads (prompt to prompt, as the receipt says).
    assert_eq!(status(&review, "ΔO").status, "fired");
    assert_eq!(status(&review, "ΔTc").status, "quiet");
    assert!(!review.verdict.contains("instability; Path B first"), "{}", review.verdict);
}

#[test]
fn an_unknown_layout_value_warns_and_reads_as_before() {
    let mut value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aspire-si/steady.json")).unwrap()).unwrap();
    value["run_metadata"]["step_axis"] = serde_json::json!("wall_clock");
    value["run_metadata"]["scalar_source"] = serde_json::json!("guessed");
    let opened = scalarscope::geometry::read(&value).unwrap();
    assert!(opened.warnings.iter().any(|warning| warning.contains("Unknown step_axis \"wall_clock\"")));
    assert!(opened.warnings.iter().any(|warning| warning.contains("Unknown scalar_source \"guessed\"")));
    assert!(opened.run.metadata.steps_are_time() && opened.run.metadata.repeated_scores().is_none());
    // A file that states nothing keeps every 2.0 reading: the simulated pair is unchanged.
    let review = reviewed("steady.json", "regressing.json");
    assert!(review.explanations.iter().all(|tile| tile.status != "withheld"));
}
