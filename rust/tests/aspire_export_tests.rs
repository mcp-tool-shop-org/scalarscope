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
