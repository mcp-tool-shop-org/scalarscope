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
    for symbol in ["ΔTc", "ΔTd", "ΔO", "ΔF"] {
        let tile = status(&review, symbol);
        assert_eq!(tile.status, "withheld", "{symbol}");
        assert!(tile.why.contains("steps are checkpoint × item, not time"), "{symbol}: {}", tile.why);
        assert!(tile.why.contains("2.0's reading was"), "{symbol}");
    }
    assert_eq!(status(&review, "Δ\u{0100}").status, "fired");
    // The scores are fixed per item, so the one delta that fires is about the evaluators.
    assert_eq!(review.verdict, "No meaningful divergence between the runs; the evaluator setups differ (spectrum concentration).");
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

// The fine-tune-then-ASPIRE run (aspire-si docs/runs/2026-10-07-sft-then-aspire.md): the same
// student, teachers and 32 exchanges as the real-* fixtures, fine-tuned on teacher answers first.

#[test]
fn the_fine_tuned_exports_open_in_full_and_state_their_layout() {
    for teacher in ["local", "composite"] {
        let professors = if teacher == "local" { 1 } else { 2 };
        for (kind, steps, axis, checkpoints, source) in [
            ("", 96, "training_step", None, "replayed"),
            (".drift-from-base", 128, "checkpoint_by_item", Some(4), "fixed_per_item"),
            (".drift-from-sft", 96, "checkpoint_by_item", Some(3), "fixed_per_item"),
        ] {
            let name = format!("sft-{teacher}-teacher{kind}.geometry.json");
            let side = geometry(&name);
            assert!(side.warnings.is_empty(), "{name}: {:?}", side.warnings);
            let run = &side.run;
            assert_eq!(run.metadata.run_id, format!("sft-{teacher}-teacher{kind}"), "{name}");
            assert_eq!(run.trajectory.timesteps.len(), steps, "{name}");
            assert_eq!(run.geometry.eigenvalues.len(), steps, "{name}");
            assert_eq!(run.evaluators.professors.len(), professors, "{name}");
            let meta = &run.metadata;
            assert_eq!((meta.step_axis.as_deref(), meta.checkpoints, meta.scalar_source.as_deref()), (Some(axis), checkpoints, Some(source)), "{name}");
            for step in &run.trajectory.timesteps {
                assert!(step.curvature.is_finite() && step.state_2d.iter().all(|value| value.is_finite()), "{name}");
            }
        }
    }
}

#[test]
fn four_checkpoints_withhold_like_three_and_pair_with_the_control_with_emergence_withheld_too() {
    // Four checkpoints from the base student, side by side.
    let review = reviewed("sft-local-teacher.drift-from-base.geometry.json", "sft-composite-teacher.drift-from-base.geometry.json");
    for symbol in ["ΔTc", "ΔTd", "ΔO", "ΔF"] {
        let tile = status(&review, symbol);
        assert_eq!(tile.status, "withheld", "{symbol}");
        assert!(tile.why.contains("steps are checkpoint × item, not time"), "{symbol}: {}", tile.why);
    }
    // ASPIRE's drift after the fine-tune against the control's ASPIRE drift: both three checkpoints.
    let review = reviewed("real-local-teacher.drift.geometry.json", "sft-local-teacher.drift-from-sft.geometry.json");
    for symbol in ["ΔTc", "ΔTd", "ΔO", "ΔF"] {
        assert_eq!(status(&review, symbol).status, "withheld", "{symbol}");
    }
    assert!(!review.verdict.is_empty());
    // A four-checkpoint file against a three-checkpoint one still pairs; nothing is invented.
    let mixed = reviewed("real-local-teacher.drift.geometry.json", "sft-local-teacher.drift-from-base.geometry.json");
    assert_eq!(status(&mixed, "ΔTc").status, "withheld");
    assert_eq!(status(&mixed, "ΔTd").status, "withheld");
}

// ΔĀ on checkpoint × item exports (ruled 2026-10-07): compared checkpoint by checkpoint, and
// fired only when B − A clears the floor with the same sign at every checkpoint.

#[test]
fn concentration_is_compared_by_checkpoint_on_the_control_drift_pair() {
    let review = reviewed("real-local-teacher.drift.geometry.json", "real-composite-teacher.drift.geometry.json");
    let tile = status(&review, "Δ\u{0100}");
    assert_eq!(tile.status, "fired");
    assert_eq!(tile.headline, "Evaluator setups differ: B's evaluators' scores are more concentrated at all 3 checkpoints");
    assert!(
        tile.why.starts_with("Evaluator setups differ: B's evaluators' scores are more concentrated at all 3 checkpoints. The scores are fixed per item, so this compares the evaluators, not the training runs."),
        "{}",
        tile.why
    );
    assert!(!tile.why.contains("sustained "), "{}", tile.why);
    for checkpoint in 1..=3 {
        assert!(tile.why.contains(&format!("Checkpoint {checkpoint}: A ")), "{}", tile.why);
    }
    assert!(tile.why.contains("3.1's rule, compared checkpoint by checkpoint"), "{}", tile.why);
    assert!(tile.anchor.is_none());
}

#[test]
fn four_checkpoints_fire_at_all_four_and_unequal_counts_are_withheld() {
    let review = reviewed("sft-local-teacher.drift-from-base.geometry.json", "sft-composite-teacher.drift-from-base.geometry.json");
    assert_eq!(review.verdict, "No meaningful divergence between the runs; the evaluator setups differ (spectrum concentration).");
    assert_eq!(status(&review, "Δ\u{0100}").headline, "Evaluator setups differ: B's evaluators' scores are more concentrated at all 4 checkpoints");
    let mixed = reviewed("real-local-teacher.drift.geometry.json", "sft-local-teacher.drift-from-base.geometry.json");
    let tile = status(&mixed, "Δ\u{0100}");
    assert_eq!(tile.status, "withheld");
    assert!(tile.why.contains("the runs have 3 and 4 checkpoints"), "{}", tile.why);
    // A checkpoint export against a time-ordered one has no blocks to pair.
    let layouts = reviewed("real-local-teacher.drift.geometry.json", "real-composite-teacher.geometry.json");
    assert_eq!(status(&layouts, "Δ\u{0100}").status, "withheld");
    // The same teacher with and without the fine-tune: the evaluators' spectrum is the same.
    let same = reviewed("real-local-teacher.drift.geometry.json", "sft-local-teacher.drift-from-sft.geometry.json");
    let tile = status(&same, "Δ\u{0100}");
    assert_eq!((tile.status.as_str(), tile.headline.as_str()), ("quiet", "Similar spectrum concentration at every checkpoint"));
    assert!(tile.why.contains("this compares the evaluators, not the training runs"), "{}", tile.why);
    assert_eq!(same.verdict, "No meaningful divergence observed between paths.");
}

#[test]
fn a_sign_that_flips_between_checkpoints_is_quiet() {
    use scalarscope::geometry_deltas::{concentration_by_checkpoint, AlignmentDetectionConfig};
    let left = geometry("real-local-teacher.drift.geometry.json").run;
    let size = left.geometry.eigenvalues.len() / 3;
    let scaled = |factor: &dyn Fn(usize) -> f64| {
        let mut right = left.clone();
        for (index, step) in right.geometry.eigenvalues.iter_mut().enumerate() {
            step.values[0] *= factor(index / size);
        }
        right
    };
    let config = AlignmentDetectionConfig::default();

    // B is more concentrated at checkpoint 1, less at checkpoint 2, and the same at checkpoint 3.
    let flips = scaled(&|block| [3.0, 0.3, 1.0][block]);
    let delta = concentration_by_checkpoint(&left, &flips, &config);
    assert_eq!(delta.status, DeltaStatus::Suppressed);
    assert_eq!(delta.explanation, "Spectrum concentration differs by checkpoint");
    assert!(delta.summary_sentence.is_none());

    // The same sign everywhere, but too small at one checkpoint: quiet, naming it.
    let short = scaled(&|block| if block == 2 { 1.01 } else { 3.0 });
    let delta = concentration_by_checkpoint(&left, &short, &config);
    assert_eq!(delta.status, DeltaStatus::Suppressed);
    assert_eq!(delta.explanation, "Similar spectrum concentration at checkpoint 3");

    // Lower everywhere: Path A is the more concentrated.
    let lower = scaled(&|_| 0.3);
    let delta = concentration_by_checkpoint(&left, &lower, &config);
    assert_eq!(delta.status, DeltaStatus::Present);
    assert_eq!(delta.summary_sentence.as_deref(), Some("Path A had a more concentrated spectrum at all 3 checkpoints"));
}

#[test]
fn time_ordered_exports_keep_the_sustained_stretch_rule() {
    let review = reviewed("real-local-teacher.geometry.json", "real-composite-teacher.geometry.json");
    let tile = status(&review, "Δ\u{0100}");
    assert!(tile.why.contains("2.0's geometry rule"), "{}", tile.why);
    assert!(!tile.headline.contains("checkpoint"), "{}", tile.headline);
}

// Seeds 43 and 44 of the fine-tune-then-ASPIRE run (aspire-si PR #30): the same conditions as the
// sft-* and real-* fixtures, run again with new dialogues. These tests pin how the reader treats
// replicates; they change no reading.

const SEEDS: [i64; 2] = [43, 44];

fn seeded(stem: &str, seed: i64, kind: &str) -> String {
    format!("{stem}-s{seed}{kind}.geometry.json")
}

#[test]
fn every_seeded_export_opens_in_full_and_states_its_seed_and_layout() {
    for seed in SEEDS {
        for teacher in ["local", "composite"] {
            let professors = if teacher == "local" { 1 } else { 2 };
            for (stem, kind, steps, axis, checkpoints, source) in [
                ("control", "", 96, "training_step", None, "replayed"),
                ("sft", "", 96, "training_step", None, "replayed"),
                ("control", ".drift-from-base", 96, "checkpoint_by_item", Some(3), "fixed_per_item"),
                ("sft", ".drift-from-base", 128, "checkpoint_by_item", Some(4), "fixed_per_item"),
                ("sft", ".drift-from-sft", 96, "checkpoint_by_item", Some(3), "fixed_per_item"),
            ] {
                let name = seeded(&format!("{stem}-{teacher}-teacher"), seed, kind);
                let side = geometry(&name);
                assert!(side.warnings.is_empty(), "{name}: {:?}", side.warnings);
                let run = &side.run;
                let meta = &run.metadata;
                assert_eq!(meta.run_id, name.trim_end_matches(".geometry.json"), "{name}");
                assert_eq!(meta.seed, seed, "{name}");
                assert_eq!((meta.step_axis.as_deref(), meta.checkpoints, meta.scalar_source.as_deref()), (Some(axis), checkpoints, Some(source)), "{name}");
                assert_eq!(run.trajectory.timesteps.len(), steps, "{name}");
                assert_eq!(run.geometry.eigenvalues.len(), steps, "{name}");
                assert_eq!(run.evaluators.professors.len(), professors, "{name}");
                assert_eq!(run.dimension_names().len(), 9, "{name}");
            }
        }
    }
}

/// The mean over every score of every step: one number per file, to tell score sets apart.
fn mean_score(run: &scalarscope::geometry::GeometryRun) -> f64 {
    let scores: Vec<f64> = run.scalars.values.iter().flat_map(|step| step.scores.values().copied()).collect();
    scores.iter().sum::<f64>() / scores.len() as f64
}

#[test]
fn per_step_scores_belong_to_their_seed_not_to_the_condition() {
    // New dialogues at each seed mean new teacher scores, so two seeds of one condition differ.
    for stem in ["control-local-teacher", "control-composite-teacher", "sft-composite-teacher"] {
        let (a, b) = (geometry(&seeded(stem, 43, "")).run, geometry(&seeded(stem, 44, "")).run);
        assert!((mean_score(&a) - mean_score(&b)).abs() > 1e-4, "{stem}");
        let review = reviewed(&seeded(stem, 43, ""), &seeded(stem, 44, ""));
        assert_eq!((review.left.metadata.seed, review.right.metadata.seed), (43, 44));
        // Replayed scores withhold ΔF on replicates as on any pair.
        assert_eq!(status(&review, "ΔF").status, "withheld", "{stem}");
    }
}

#[test]
fn drift_replicates_read_alike_and_mixed_checkpoint_counts_do_not_pair() {
    // Drift scores are fixed per item: the same items at every seed, so the evaluators' spectrum
    // is the same, and ΔĀ is quiet at every checkpoint between replicates.
    for (stem, kind) in [
        ("control-local-teacher", ".drift-from-base"),
        ("control-composite-teacher", ".drift-from-base"),
        ("sft-local-teacher", ".drift-from-base"),
        ("sft-composite-teacher", ".drift-from-base"),
        ("sft-local-teacher", ".drift-from-sft"),
        ("sft-composite-teacher", ".drift-from-sft"),
    ] {
        let review = reviewed(&seeded(stem, 43, kind), &seeded(stem, 44, kind));
        let tile = status(&review, "Δ\u{0100}");
        assert_eq!((tile.status.as_str(), tile.headline.as_str()), ("quiet", "Similar spectrum concentration at every checkpoint"), "{stem}{kind}");
        assert_eq!(review.verdict, "No meaningful divergence observed between paths.", "{stem}{kind}");
        for symbol in ["ΔTc", "ΔTd", "ΔO", "ΔF"] {
            assert_eq!(status(&review, symbol).status, "withheld", "{stem}{kind} {symbol}");
        }
    }
    // The seeded control drift reads like the unseeded one.
    let review = reviewed("real-local-teacher.drift.geometry.json", &seeded("control-local-teacher", 43, ".drift-from-base"));
    assert_eq!(status(&review, "Δ\u{0100}").status, "quiet");
    // Three checkpoints against four: ΔĀ is withheld, the blocks do not pair.
    let mixed = reviewed(&seeded("control-local-teacher", 43, ".drift-from-base"), &seeded("sft-local-teacher", 43, ".drift-from-base"));
    assert!(status(&mixed, "Δ\u{0100}").why.contains("the runs have 3 and 4 checkpoints"));
    // Across teachers at one seed, the difference is the evaluator setups'.
    let teachers = reviewed(&seeded("control-local-teacher", 43, ".drift-from-base"), &seeded("control-composite-teacher", 43, ".drift-from-base"));
    assert_eq!(teachers.verdict, "No meaningful divergence between the runs; the evaluator setups differ (spectrum concentration).");
}
