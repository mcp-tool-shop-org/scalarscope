//! The five 2.0 geometry deltas, ported from `CanonicalDeltaService.cs`. The oracle is what 2.0
//! computed for its own samples (also in `tests/Fixtures/Bundles/dotnet-geometry-compare.scbundle`).

use scalarscope::geometry::{self, Failure, GeometryRun, Timestep};
use scalarscope::geometry_deltas::*;

const TOLERANCE: f64 = 1e-9;

fn sample(name: &str) -> GeometryRun {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/ScalarScope/Resources/Raw/Samples").join(name);
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    geometry::read(&value).unwrap().run
}

fn close(actual: f64, expected: f64, what: &str) {
    assert!((actual - expected).abs() <= TOLERANCE, "{what}: {actual} vs {expected}");
}

fn find<'a>(deltas: &'a [GeometryDelta], id: &str) -> &'a GeometryDelta {
    deltas.iter().find(|delta| delta.id == id).unwrap_or_else(|| panic!("no {id}"))
}

fn present(deltas: &[GeometryDelta]) -> Vec<&GeometryDelta> {
    deltas.iter().filter(|delta| delta.status == DeltaStatus::Present).collect()
}

fn expect(delta: &GeometryDelta, left: f64, right: f64, change: f64, confidence: f64, explanation: &str) {
    assert_eq!(delta.status, DeltaStatus::Present, "{}", delta.id);
    close(delta.left_value, left, &format!("{} left", delta.id));
    close(delta.right_value, right, &format!("{} right", delta.id));
    close(delta.delta, change, &format!("{} delta", delta.id));
    close(delta.confidence, confidence, &format!("{} confidence", delta.id));
    assert_eq!(delta.explanation, explanation, "{}", delta.id);
}

/// A run from (speed, curvature) per step, eigenvalue rows and recorded failure times.
fn run(steps: &[(f64, f64)], eigen: &[&[f64]], failures: &[f64]) -> GeometryRun {
    let mut run = GeometryRun::default();
    run.trajectory.timesteps = steps
        .iter()
        .enumerate()
        .map(|(i, (speed, curvature))| Timestep { t: i as f64, state_2d: vec![0.0, 0.0], velocity: vec![*speed, 0.0], curvature: *curvature, effective_dim: 0.0 })
        .collect();
    run.geometry.eigenvalues = eigen.iter().enumerate().map(|(i, values)| geometry::EigenStep { t: i as f64, values: values.to_vec() }).collect();
    run.failures = failures.iter().map(|t| Failure { t: *t, category: "spike".to_string(), severity: "high".to_string(), description: String::new() }).collect();
    run
}

fn calm(len: usize) -> Vec<(f64, f64)> {
    vec![(1.0, 0.0); len]
}

/// Speed alternates 5, 0, 5, 0: no window stays inside the epsilon band.
fn jumpy() -> Vec<(f64, f64)> {
    (0..20).map(|i| (if i % 2 == 0 { 5.0 } else { 0.0 }, 0.0)).collect()
}

fn flat_eigen<'a>(len: usize, values: &'a [f64]) -> Vec<&'a [f64]> {
    vec![values; len]
}

fn compute_default(left: &GeometryRun, right: &GeometryRun) -> Vec<GeometryDelta> {
    compute(left, right, Alignment::ByStep, 1.0, &DeltaConfig::default())
}

#[test]
fn the_2_0_oracle_orthogonal_against_correlated() {
    let left = sample("orthogonal_professors.json");
    let right = sample("correlated_professors.json");
    let deltas = compute(&left, &right, Alignment::ByStep, 1.0, &DeltaConfig::default());

    assert_eq!(
        deltas.iter().map(|delta| delta.id.as_str()).collect::<Vec<_>>(),
        ["failurePresence", "convergenceTiming", "structuralEmergence", "evaluatorAlignment", "stabilityOscillation"]
    );
    assert_eq!(deltas.iter().map(|delta| delta.symbol.as_str()).collect::<Vec<_>>(), ["ΔF", "ΔTc", "ΔTd", "Δ\u{0100}", "ΔO"]);
    assert_eq!(
        present(&deltas).iter().map(|delta| delta.id.as_str()).collect::<Vec<_>>(),
        ["failurePresence", "structuralEmergence", "evaluatorAlignment", "stabilityOscillation"]
    );

    // 2.0 said "correctness_dip near step 6": the first failure's kind at the third's time. G8 takes
    // both from the third. The 2.0 bundle in tests/Fixtures/Bundles keeps 2.0's words.
    expect(find(&deltas, "failurePresence"), 1.0, 0.0, -1.0, 1.0, "Only Path A experienced tradeoffs_failure near step 6");
    expect(find(&deltas, "structuralEmergence"), -1.0, 0.2, 1.0, 1.0, "Path B developed dominant direction; Path A remained distributed");
    // 2.0 said "Path B maintained higher evaluator agreement over 6 steps". G5 names the measure.
    expect(find(&deltas, "evaluatorAlignment"), 0.32000000000000006, 0.6333333333333333, 0.31333333333333324, 1.0, "Path B kept a more concentrated spectrum over 6 steps");
    expect(find(&deltas, "stabilityOscillation"), 0.0, 0.23000000000000004, 0.23000000000000004, 1.0, "Path B showed sustained instability during training (4 steps)");
    assert_ne!(find(&deltas, "convergenceTiming").status, DeltaStatus::Present);

    // The rest of what the 2.0 bundle carries.
    let failure = find(&deltas, "failurePresence");
    assert_eq!((failure.name.as_str(), failure.delta_type, failure.visual_anchor_time), ("Failure Events", DeltaType::Event, Some(0.6)));
    assert_eq!((failure.t_fail_a, failure.t_fail_b, failure.failed_a, failure.failed_b), (Some(6), None, Some(true), Some(false)));
    let alignment = find(&deltas, "evaluatorAlignment");
    assert_eq!(alignment.summary_sentence.as_deref(), Some("Path B had a more concentrated spectrum (sustained 6 steps)"));
    assert_eq!(alignment.name, "Spectrum concentration");
    close(alignment.magnitude, 0.3133333333333333, "alignment magnitude");
    assert_eq!((alignment.delta_type, alignment.visual_anchor_time), (DeltaType::Structure, Some(0.0)));
    let stability = find(&deltas, "stabilityOscillation");
    assert_eq!(stability.summary_sentence.as_deref(), Some("Path B showed 0.23 more oscillation"));
    close(stability.visual_anchor_time.unwrap(), 0.18181818181818182, "stability anchor");
    assert_eq!(find(&deltas, "structuralEmergence").delta_type, DeltaType::Timing);
    assert!(deltas.iter().all(|delta| delta.is_meaningful() == (delta.status == DeltaStatus::Present)));

    // The summary names the first meaningful delta with a modifier from the second.
    let result = compute_with_summary(&left, &right, Alignment::ByStep, 1.0, &DeltaConfig::default());
    assert_eq!(result.deltas, deltas);
    assert_eq!(result.alignment.description, "Aligned by training step");
    assert_eq!(result.comparative_summary, "Only Path A experienced tradeoffs_failure near step 6 and distinct structural emergence.");
}

#[test]
fn the_reverse_pair_mirrors_the_oracle() {
    let orthogonal = sample("orthogonal_professors.json");
    let correlated = sample("correlated_professors.json");
    let forward = compute_default(&orthogonal, &correlated);
    let reverse = compute_default(&correlated, &orthogonal);

    assert_eq!(present(&reverse).len(), 4);
    assert_ne!(find(&reverse, "convergenceTiming").status, DeltaStatus::Present);
    for id in ["failurePresence", "structuralEmergence", "evaluatorAlignment", "stabilityOscillation"] {
        let (a, b) = (find(&forward, id), find(&reverse, id));
        close(b.left_value, a.right_value, &format!("{id} left"));
        close(b.right_value, a.left_value, &format!("{id} right"));
        close(b.confidence, a.confidence, &format!("{id} confidence"));
        close(b.delta, -a.delta, &format!("{id} delta"));
        close(b.magnitude, a.magnitude, &format!("{id} magnitude"));
    }
    assert_eq!(find(&reverse, "failurePresence").explanation, "Only Path B experienced tradeoffs_failure near step 6");
    assert_eq!(find(&reverse, "structuralEmergence").explanation, "Path A developed dominant direction; Path B remained distributed");
    assert_eq!(find(&reverse, "evaluatorAlignment").explanation, "Path A kept a more concentrated spectrum over 6 steps");
    assert_eq!(find(&reverse, "stabilityOscillation").explanation, "Path A showed sustained instability during training (4 steps)");
    assert_eq!(find(&reverse, "stabilityOscillation").summary_sentence.as_deref(), Some("Path A showed 0.23 more oscillation"));
}

#[test]
fn failure_presence_is_suppressed_when_neither_run_fails() {
    let quiet = run(&calm(11), &[], &[]);
    let deltas = compute_default(&quiet, &quiet);
    let failure = find(&deltas, "failurePresence");
    assert_eq!(failure.status, DeltaStatus::Suppressed);
    assert_eq!(failure.explanation, "No failure detected");
    assert_eq!(failure.notes, ["Neither run experienced failure events"]);
    assert_eq!((failure.failed_a, failure.failed_b), (Some(false), Some(false)));
}

#[test]
fn failure_presence_needs_three_recorded_failures_and_names_both_runs() {
    let two = run(&calm(11), &[], &[0.2, 0.4]);
    assert_eq!(find(&compute_default(&two, &two), "failurePresence").status, DeltaStatus::Suppressed);

    let early = run(&calm(11), &[], &[0.1, 0.2, 0.3]);
    let late = run(&calm(11), &[], &[0.5, 0.6, 0.7]);
    let both = compute_default(&early, &late);
    expect(find(&both, "failurePresence"), 1.0, 1.0, 0.0, 1.0, "Both paths experienced instability; Path A first");
    let same = compute_default(&early, &early);
    assert_eq!(find(&same, "failurePresence").explanation, "Both paths experienced instability; the failures were simultaneous");
}

#[test]
fn convergence_is_suppressed_when_neither_converges_or_both_settle_within_the_resolution() {
    let neither = compute_default(&run(&jumpy(), &[], &[]), &run(&jumpy(), &[], &[]));
    let tc = find(&neither, "convergenceTiming");
    assert_eq!((tc.status, tc.explanation.as_str()), (DeltaStatus::Suppressed, "No convergence detected"));
    assert_eq!(tc.notes, ["Neither run converged within observation window"]);
    assert_eq!((tc.tc_a, tc.tc_b, tc.window_used), (None, None, Some(5)));

    // Both settle at step 5 on constant speed: |ΔTc| = 0 < ResolutionSteps.
    let steady = run(&calm(20), &[], &[]);
    let both = compute_default(&steady, &steady);
    let tc = find(&both, "convergenceTiming");
    assert_eq!((tc.status, tc.explanation.as_str()), (DeltaStatus::Suppressed, "Similar convergence timing"));
    assert_eq!(tc.notes, ["Both runs converged near step 5 (Δ=0 < 3 steps)"]);
    assert_eq!((tc.tc_a, tc.tc_b, tc.delta_tc_steps), (Some(5), Some(5), Some(0)));
    close(tc.left_value, 5.0 / 19.0, "time");
}

#[test]
fn convergence_when_only_one_run_settles_is_present() {
    let steady = run(&calm(20), &[], &[]);
    let deltas = compute_default(&steady, &run(&jumpy(), &[], &[]));
    let tc = find(&deltas, "convergenceTiming");
    assert_eq!(tc.status, DeltaStatus::Present);
    assert_eq!(tc.explanation, "Path A stabilized; Path B did not within observed steps");
    assert_eq!((tc.tc_a, tc.tc_b, tc.delta, tc.magnitude), (Some(5), None, 1.0, 1.0));
    assert_eq!(tc.anchors[0].range_b, Some((15, 20)));
    // Tail 10 steps against max(2 * window, 10) -> 1; no violations -> 1; epsilon 0.02 -> 0.8.
    close(tc.confidence, 0.4 * 1.0 + 0.4 * 1.0 + 0.2 * 0.8, "confidence");
}

#[test]
fn structural_emergence_is_suppressed_without_dominance_or_when_both_emerge_together() {
    let flat = flat_eigen(11, &[1.0, 1.0, 1.0]);
    let none = compute_default(&run(&calm(11), &flat, &[]), &run(&calm(11), &flat, &[]));
    let td = find(&none, "structuralEmergence");
    assert_eq!((td.status, td.explanation.as_str()), (DeltaStatus::Suppressed, "No structural dominance"));
    assert_eq!(td.notes, ["Neither run achieved eigenvalue dominance"]);
    assert_eq!(td.dominance_ratio_k, Some(1.5));

    let dominant = flat_eigen(11, &[3.0, 1.0, 1.0]);
    let together = compute_default(&run(&calm(11), &dominant, &[]), &run(&calm(11), &dominant, &[]));
    let td = find(&together, "structuralEmergence");
    assert_eq!((td.status, td.explanation.as_str()), (DeltaStatus::Suppressed, "Similar emergence timing"));
    assert_eq!(td.notes, ["Both runs achieved dominance near step 0"]);
}

#[test]
fn structural_emergence_names_the_earlier_run_and_reads_recurrence() {
    // A is dominant from step 0; B only from step 6.
    let a_rows = flat_eigen(11, &[3.0, 1.0]);
    let b_rows: Vec<&[f64]> = (0..11).map(|i| if i >= 6 { &[3.0, 1.0][..] } else { &[1.0, 1.0][..] }).collect();
    let deltas = compute_default(&run(&calm(11), &a_rows, &[]), &run(&calm(11), &b_rows, &[]));
    let td = find(&deltas, "structuralEmergence");
    assert_eq!(td.explanation, "Path A developed dominant direction 6 steps earlier");
    assert_eq!(td.summary_sentence.as_deref(), Some("Path A achieved structural dominance 6 steps before the other"));
    assert_eq!((td.td_a, td.td_b), (Some(0), Some(6)));
    close(td.delta, 0.6, "delta");

    // Two segments of 2 inside 7 steps, never 3 in a row: recurrence.
    let flicker: Vec<&[f64]> = (0..11).map(|i| if matches!(i, 2 | 3 | 5 | 6) { &[3.0, 1.0][..] } else { &[1.0, 1.0][..] }).collect();
    let deltas = compute_default(&run(&calm(11), &flat_eigen(11, &[1.0, 1.0]), &[]), &run(&calm(11), &flicker, &[]));
    let td = find(&deltas, "structuralEmergence");
    assert_eq!(td.explanation, "Path B developed dominant direction; Path A remained distributed (detected via recurrence)");
    assert_eq!(td.anchors[0].range_b, Some((2, 9)));
}

#[test]
fn evaluator_alignment_is_suppressed_below_the_floor_or_when_the_difference_is_brief() {
    let same = flat_eigen(11, &[2.0, 1.0, 1.0]);
    let deltas = compute_default(&run(&calm(11), &same, &[]), &run(&calm(11), &same, &[]));
    let alignment = find(&deltas, "evaluatorAlignment");
    assert_eq!((alignment.status, alignment.explanation.as_str()), (DeltaStatus::Suppressed, "Similar spectrum concentration"));
    assert_eq!(alignment.notes, ["Persistence-weighted concentration difference below threshold"]);
    close(alignment.magnitude, 0.0, "magnitude");

    // A large difference for only 2 steps: over the floor but under MinPersistenceSteps (4).
    let other: Vec<&[f64]> = (0..11).map(|i| if i == 3 || i == 4 { &[3.0, 1.0][..] } else { &[2.0, 1.0, 1.0][..] }).collect();
    let deltas = compute_default(&run(&calm(11), &same, &[]), &run(&calm(11), &other, &[]));
    let alignment = find(&deltas, "evaluatorAlignment");
    assert_eq!((alignment.status, alignment.explanation.as_str()), (DeltaStatus::Suppressed, "Brief concentration difference"));
    assert_eq!(alignment.notes, ["Sustained segment (2 steps) below minimum (4)"]);

    // No eigenvalues at all: nothing paired.
    let deltas = compute_default(&run(&calm(11), &[], &[]), &run(&calm(11), &[], &[]));
    let alignment = find(&deltas, "evaluatorAlignment");
    assert_eq!((alignment.status, alignment.explanation.as_str()), (DeltaStatus::Suppressed, "No aligned samples"));
}

/// Curvature 0 with four steps of `spike` from `spike_at`.
fn spiky(len: usize, spike_at: usize, spike: f64) -> Vec<(f64, f64)> {
    (0..len).map(|i| (1.0, if (spike_at..spike_at + 4).contains(&i) { spike } else { 0.0 })).collect()
}

#[test]
fn stability_is_suppressed_when_both_are_calm_or_the_difference_is_under_the_floor() {
    let calm_run = run(&calm(20), &[], &[]);
    let both_calm = compute_default(&calm_run, &calm_run);
    let stability = find(&both_calm, "stabilityOscillation");
    assert_eq!((stability.status, stability.explanation.as_str()), (DeltaStatus::Suppressed, "Both runs stable"));
    assert_eq!(stability.notes, ["Both runs maintained stable trajectories (scores below noise floor)"]);
    assert_eq!((stability.score_a, stability.score_b, stability.min_duration_used), (Some(0.0), Some(0.0), Some(4)));

    // Both wobble (scores about 0.36 and 0.38, over the noise floor) but |Δ| = 0.02 < 0.05.
    let left = run(&spiky(20, 5, 0.1), &[], &[]);
    let right = run(&spiky(20, 5, 0.105), &[], &[]);
    let deltas = compute_default(&left, &right);
    let stability = find(&deltas, "stabilityOscillation");
    assert_eq!((stability.status, stability.explanation.as_str()), (DeltaStatus::Suppressed, "Similar oscillation levels"));
    assert_eq!(stability.notes, ["Oscillation difference below floor (|Δ|=0.020 < 0.05)"]);
    close(stability.score_a.unwrap(), 0.36, "score a");
    close(stability.threshold_used.unwrap(), 0.01, "theta");

    // A clear difference is present, with the peak episode's range.
    let louder = run(&spiky(20, 5, 0.5), &[], &[]);
    let deltas = compute_default(&left, &louder);
    let stability = find(&deltas, "stabilityOscillation");
    assert_eq!(stability.status, DeltaStatus::Present);
    assert_eq!(stability.explanation, "Path B showed sustained instability during training (4 steps)");
    assert_eq!(stability.anchors[0].range_b, Some((5, 8)));
    assert_eq!(stability.anchors[0].range_a, None);
    close(stability.visual_anchor_time.unwrap(), 5.0 / 20.0, "anchor");
}

#[test]
fn robust_sigma_is_zero_below_five_values() {
    assert_eq!(robust_sigma(&[]), 0.0);
    assert_eq!(robust_sigma(&[1.0, 2.0, 3.0, 4.0]), 0.0);
    assert_eq!(robust_sigma(&[1.0, 100.0]), 0.0);
    // Sorted median 3; deviations 0 1 1 2 97 -> median 1.
    close(robust_sigma(&[1.0, 2.0, 3.0, 4.0, 100.0]), 1.4826, "sigma");
    assert_eq!(robust_sigma(&[2.0; 7]), 0.0);
}

/// Speed jumps for `ramp` steps, then holds at 1.
fn ramp_then_steady(ramp: usize, len: usize) -> Vec<(f64, f64)> {
    (0..len).map(|i| (if i < ramp { if i % 2 == 0 { 5.0 } else { 0.0 } } else { 1.0 }, 0.0)).collect()
}

#[test]
fn by_convergence_alignment_anchors_both_runs_at_their_convergence_step() {
    let a = run(&ramp_then_steady(8, 20), &[], &[]);
    let b = run(&ramp_then_steady(4, 20), &[], &[]);
    let map = create_alignment_map(&a, &b, Alignment::ByConvergence);
    assert_eq!(map.mode, Alignment::ByConvergence);
    assert_eq!(map.description, "Aligned at convergence (A: step 8, B: step 5)");
    assert_eq!(map.compare_index.len(), 23);
    assert_eq!(map.idx_to_step_a[0], Some(0));
    assert_eq!(map.idx_to_step_b[0], None);
    assert_eq!(map.idx_to_step_a[8], Some(8));
    assert_eq!(map.idx_to_step_b[8], Some(5));
    assert_eq!(map.idx_to_step_a[19], Some(19));
    assert_eq!(map.idx_to_step_a[20], None);
    assert_eq!(map.idx_to_step_b[22], Some(19));
    assert_eq!(map.idx_to_step_b[3], Some(0));

    let deltas = compute(&a, &b, Alignment::ByConvergence, 1.0, &DeltaConfig::default());
    let tc = find(&deltas, "convergenceTiming");
    assert_eq!(tc.status, DeltaStatus::Present);
    assert_eq!(tc.explanation, "Path B converged 3 steps earlier");
    assert_eq!(tc.summary_sentence.as_deref(), Some("Path B settled 3 steps before the other path"));
    assert_eq!((tc.tc_a, tc.tc_b, tc.delta_tc_steps), (Some(8), Some(5), Some(-3)));
    // The weaker side sets the confidence: A's tail is 7 steps (B's is 10, scoring 0.96).
    close(tc.confidence, 0.4 * 0.7 + 0.4 + 0.2 * 0.8, "confidence");
}

#[test]
fn alignment_falls_back_to_steps_when_nothing_anchors_and_by_instability_anchors_on_a_spike() {
    let steady = run(&calm(20), &[], &[]);
    let map = create_alignment_map(&run(&jumpy(), &[], &[]), &run(&jumpy(), &[], &[]), Alignment::ByConvergence);
    assert_eq!(map.description, "Neither path converged; using step alignment");
    assert_eq!(map.mode, Alignment::ByStep);
    let map = create_alignment_map(&steady, &steady, Alignment::ByFirstInstability);
    assert_eq!(map.description, "Neither path showed instability; using step alignment");

    let spike_early = run(&spiky(20, 2, 0.9), &[], &[]);
    let spike_late = run(&spiky(20, 6, 0.9), &[], &[]);
    let map = create_alignment_map(&spike_early, &spike_late, Alignment::ByFirstInstability);
    assert_eq!(map.description, "Aligned at first change (A: step 2, B: step 6)");
    assert_eq!(map.idx_to_step_a[6], Some(2));
    assert_eq!(map.idx_to_step_b[6], Some(6));

    let empty = GeometryRun::default();
    let map = create_alignment_map(&empty, &steady, Alignment::ByStep);
    assert_eq!((map.description.as_str(), map.compare_index.len()), ("No data available", 0));
    let deltas = compute(&empty, &steady, Alignment::ByStep, 1.0, &DeltaConfig::default());
    assert_eq!(deltas.len(), 5);
    assert_eq!(find(&deltas, "stabilityOscillation").explanation, "No aligned samples");
}

#[test]
fn the_auto_summary_composes_the_first_two_meaningful_deltas() {
    assert_eq!(auto_summary(&[]), "No meaningful divergence observed between paths.");
    let quiet = run(&calm(11), &[], &[]);
    assert_eq!(auto_summary(&compute_default(&quiet, &quiet)), "No meaningful divergence observed between paths.");
    let a = run(&calm(20), &[], &[]);
    let louder = run(&spiky(20, 5, 0.5), &[], &[]);
    // Only the stability delta is meaningful: its summary sentence stands alone.
    assert_eq!(auto_summary(&compute_default(&a, &louder)), "Path B showed 1.96 more oscillation");
}

#[test]
fn dotnet_number_formatting() {
    assert_eq!(fixed(0.0625, 3), "0.063");
    assert_eq!(fixed(0.25, 1), "0.3");
    assert_eq!(fixed(0.23000000000000004, 2), "0.23");
    assert_eq!(fixed(0.0201, 3), "0.020");
    assert_eq!(dotnet_double(0.05), "0.05");
    assert_eq!(dotnet_double(3.0), "3");
    assert_eq!(dotnet_double(1e-7), "1E-07");
    assert_eq!(dotnet_double(1.5e15), "1.5E+15");
}

// The rulings of 2026-10-06 (spec "Decision brief") on the port's open questions.

#[test]
fn g1_convergence_alignment_follows_the_config_delta_tc_reads() {
    // Speed settles at step 3 within a window of 3, but jumps again at step 7, so a window of 5
    // first holds from step 8.
    let speeds = [5.0, 0.0, 5.0, 1.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let a = run(&speeds.iter().map(|speed| (*speed, 0.0)).collect::<Vec<_>>(), &[], &[]);
    let b = run(&calm(16), &[], &[]);
    let default = create_alignment_map(&a, &b, Alignment::ByConvergence);
    assert_eq!(default.description, "Aligned at convergence (A: step 8, B: step 5)");
    let short = ConvergenceConfig { window: 3, ..ConvergenceConfig::default() };
    let moved = create_alignment_map_with(&a, &b, Alignment::ByConvergence, &short);
    assert_eq!(moved.description, "Aligned at convergence (A: step 3, B: step 3)");
    // compute takes its alignment from the same config.
    let config = DeltaConfig { convergence: short, ..DeltaConfig::default() };
    let result = compute_with_summary(&a, &b, Alignment::ByConvergence, 1.0, &config);
    assert_eq!(result.alignment.description, moved.description);
}

#[test]
fn g2_first_instability_reads_absolute_curvature() {
    // A run that bends one way: 2.0's signed mean is negative, its threshold 0.3, and no signed
    // curvature passes it. On absolute curvature the threshold is 2 × 0.65 and step 6 passes.
    let mut bends: Vec<(f64, f64)> = vec![(1.0, -0.5); 11];
    bends[6] = (1.0, -2.0);
    let a = run(&bends, &[], &[]);
    let b = run(&calm(11), &[], &[]);
    let map = create_alignment_map(&a, &b, Alignment::ByFirstInstability);
    assert_eq!(map.description, "Aligned at first change (A: step 6, B: step 2)");
}

#[test]
fn g3_a_failure_on_the_last_step_is_on_the_last_step() {
    // Eleven steps, the third recorded failure at normalised time 1: step 10, not 11.
    let failing = run(&calm(11), &[], &[0.2, 0.5, 1.0]);
    let deltas = compute_default(&failing, &run(&calm(11), &[], &[]));
    let failure = find(&deltas, "failurePresence");
    assert_eq!(failure.t_fail_a, Some(10));
    assert_eq!(failure.explanation, "Only Path A experienced spike near step 10");
}

#[test]
fn g4_the_sign_says_which_run_settled() {
    let steady = run(&calm(20), &[], &[]);
    let jumpy = run(&jumpy(), &[], &[]);
    let only_a = find(&compute_default(&steady, &jumpy), "convergenceTiming").delta;
    let only_b = find(&compute_default(&jumpy, &steady), "convergenceTiming").clone();
    assert_eq!(only_a, 1.0);
    assert_eq!((only_b.status, only_b.delta, only_b.magnitude), (DeltaStatus::Present, -1.0, 1.0));
    assert_eq!(only_b.explanation, "Path B stabilized; Path A did not within observed steps");
}

#[test]
fn g8_the_persistence_window_is_read_and_one_failure_gives_time_and_kind() {
    let mut failing = run(&calm(11), &[], &[]);
    failing.failures = [(0.1, "dip"), (0.4, "spike"), (0.8, "collapse")]
        .iter()
        .map(|(t, category)| Failure { t: *t, category: category.to_string(), severity: "high".to_string(), description: String::new() })
        .collect();
    let quiet = run(&calm(11), &[], &[]);
    let default = find(&compute_default(&failing, &quiet), "failurePresence").clone();
    assert_eq!(default.explanation, "Only Path A experienced collapse near step 8");
    let two = DeltaConfig { failure: FailureConfig { persistence_window: 2, ..FailureConfig::default() }, ..DeltaConfig::default() };
    let windowed = find(&compute(&failing, &quiet, Alignment::ByStep, 1.0, &two), "failurePresence").clone();
    assert_eq!(windowed.explanation, "Only Path A experienced spike near step 4");
    let four = DeltaConfig { failure: FailureConfig { persistence_window: 4, ..FailureConfig::default() }, ..DeltaConfig::default() };
    assert_eq!(find(&compute(&failing, &quiet, Alignment::ByStep, 1.0, &four), "failurePresence").status, DeltaStatus::Suppressed);
}

#[test]
fn g9_a_step_short_of_eigenvalues_is_left_out() {
    // Both runs concentrate half their spectrum in the first direction, except one step of B that
    // has a single eigenvalue. 2.0 counted it as 0 and saw a difference; it is left out.
    let full = flat_eigen(11, &[2.0, 1.0, 1.0]);
    let mut sparse = full.clone();
    sparse[5] = &[1.0];
    let deltas = compute_default(&run(&calm(11), &full, &[]), &run(&calm(11), &sparse, &[]));
    let alignment = find(&deltas, "evaluatorAlignment");
    assert_eq!(alignment.status, DeltaStatus::Suppressed);
    close(alignment.magnitude, 0.0, "magnitude");
    close(alignment.mean_align_b.unwrap(), 0.5, "B's mean over the steps it has");
}
