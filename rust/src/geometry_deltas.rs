//! The five canonical deltas 2.0 computed between two geometry runs, ported from
//! `Services/CanonicalDeltaService.cs`, `Services/DeltaTypes.cs` and `Services/AlignmentMapper.cs`.
//!
//! The arithmetic, thresholds, tie-breaking and sentences are 2.0's. Where this port differs it
//! says so:
//!
//! * `compute` returns every detector's delta, suppressed ones included and marked
//!   `DeltaStatus::Suppressed`. 2.0's `ComputeDeltas` dropped them from its list. 2.0's own list
//!   is `compute(..).into_iter().filter(GeometryDelta::is_meaningful)`.
//! * `visual_anchor_time` is `None` where 2.0 left its default of 0 (suppressed deltas).
//! * 2.0 hashed its inputs and outputs (`DeterminismService`); that is not ported.
//!
//! Numbers formatted into sentences use 2.0's rules (`F2`/`F3` round half away from zero, and a
//! bare double prints as .NET prints it), see [`fixed`] and [`dotnet_double`].
//!
//! Behaviour believed to be a bug in 2.0 was ported as it was and listed as G1-G10 in the spec.
//! The rulings of 2026-10-06 (spec "Decision brief") are applied and marked `G<n> (ruled ...)` where they
//! change 2.0's answer, or `kept as 2.0` where they do not. G10 waits for real exports and is
//! still marked `C# BUG?`.

use std::cmp::Ordering;

use crate::geometry::{GeometryRun, Timestep};

/// 2.0's `TemporalAlignment`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    /// Align by training step (the default).
    ByStep,
    /// Align at convergence onset.
    ByConvergence,
    /// Align at the first curvature spike.
    ByFirstInstability,
}

/// 2.0's `DeltaStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaStatus {
    /// Present and meaningful.
    Present,
    /// Below a threshold or not meaningful.
    Suppressed,
    /// Cannot be determined (2.0 declared it and never produced it).
    Indeterminate,
}

/// 2.0's `DeltaType`, for grouping and styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaType {
    /// When something happens.
    Timing,
    /// How paths behave.
    Behavior,
    /// Geometric properties.
    Structure,
    /// Discrete occurrences.
    Event,
}

/// 2.0's `DeltaIds`, the camelCase wire tokens.
pub mod delta_ids {
    pub const FAILURE_PRESENCE: &str = "failurePresence";
    pub const CONVERGENCE_TIMING: &str = "convergenceTiming";
    pub const STRUCTURAL_EMERGENCE: &str = "structuralEmergence";
    pub const EVALUATOR_ALIGNMENT: &str = "evaluatorAlignment";
    pub const STABILITY_OSCILLATION: &str = "stabilityOscillation";
}

const SYMBOL_FAILURE: &str = "ΔF";
const SYMBOL_CONVERGENCE: &str = "ΔTc";
const SYMBOL_EMERGENCE: &str = "ΔTd";
const SYMBOL_ALIGNMENT: &str = "Δ\u{0100}";
const SYMBOL_STABILITY: &str = "ΔO";

/// 2.0's `ConvergenceConfig`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConvergenceConfig {
    /// Stability window length in steps. C# 5.
    pub window: i64,
    /// Minimum window to accept convergence. C# 3. (2.0 never reads it.)
    pub min_window: i64,
    /// Base epsilon; the effective epsilon is `max(epsilon, epsilon_sigma_multiplier * sigma)`. C# 0.02.
    pub epsilon: f64,
    /// Robust-sigma multiplier for epsilon. C# 0.5.
    pub epsilon_sigma_multiplier: f64,
    /// Minimum step separation for |ΔTc| to be meaningful, the only suppression gate. C# 3.
    pub resolution_steps: i64,
    /// Normalised resolution for display only. C# 0.05. (2.0 never reads it.)
    pub display_resolution_norm: f64,
}

impl Default for ConvergenceConfig {
    fn default() -> Self {
        Self { window: 5, min_window: 3, epsilon: 0.02, epsilon_sigma_multiplier: 0.5, resolution_steps: 3, display_resolution_norm: 0.05 }
    }
}

/// 2.0's `StabilityConfig`.
#[derive(Clone, Debug, PartialEq)]
pub struct StabilityConfig {
    /// Fixed curvature threshold; 0 means adaptive. C# 0.0.
    pub theta: f64,
    /// Median multiplier of the adaptive threshold. C# 1.5.
    pub theta_multiplier: f64,
    /// Robust-sigma multiplier of the adaptive threshold. C# 1.0.
    pub theta_sigma_multiplier: f64,
    /// Minimum sustained duration of an episode. C# 4.
    pub min_duration: i64,
    /// Minimum |ΔO| to be meaningful. C# 0.05.
    pub delta_floor: f64,
    /// Minimum episode score to count. C# 0.1.
    pub noise_floor: f64,
    /// Deprecated multiplier. C# 1.5. (2.0 never reads it.)
    pub threshold_multiplier: f64,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self { theta: 0.0, theta_multiplier: 1.5, theta_sigma_multiplier: 1.0, min_duration: 4, delta_floor: 0.05, noise_floor: 0.1, threshold_multiplier: 1.5 }
    }
}

/// 2.0's `AlignmentDetectionConfig`.
#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentDetectionConfig {
    /// Smoothing window for A(t). C# 5. (2.0 never reads it.)
    pub smooth_window: i64,
    /// Minimum evaluator (eigenvalue) count. C# 2.
    pub min_evaluators: i64,
    /// Minimum persistence-weighted delta to be meaningful. C# 0.05.
    pub delta_floor: f64,
    /// Minimum variance. C# 0.01. (2.0 never reads it.)
    pub min_variance: f64,
    /// Minimum sustained steps. C# 4.
    pub min_persistence_steps: i64,
    /// Epsilon for a sustained difference segment. C# 0.02.
    pub segment_epsilon: f64,
}

impl Default for AlignmentDetectionConfig {
    fn default() -> Self {
        Self { smooth_window: 5, min_evaluators: 2, delta_floor: 0.05, min_variance: 0.01, min_persistence_steps: 4, segment_epsilon: 0.02 }
    }
}

/// 2.0's `EmergenceConfig`.
#[derive(Clone, Debug, PartialEq)]
pub struct EmergenceConfig {
    /// Dominance ratio k where λ1 > k·λ2. C# 1.5.
    pub k: f64,
    /// Persistence window for sustained dominance. C# 3.
    pub window: i64,
    /// Minimum step difference to be meaningful. C# 3.
    pub resolution_steps: i64,
    /// Rolling window of the recurrence rule. C# 7.
    pub recurrence_window: i64,
    /// Minimum segment length (must be 2 or more). C# 2.
    pub min_segment_length: i64,
    /// Minimum segments for the recurrence rule. C# 2.
    pub min_recurrence_count: i64,
}

impl Default for EmergenceConfig {
    fn default() -> Self {
        Self { k: 1.5, window: 3, resolution_steps: 3, recurrence_window: 7, min_segment_length: 2, min_recurrence_count: 2 }
    }
}

/// 2.0's `FailureConfig`. The detector reads none of it (see `has_persistent_failure`).
#[derive(Clone, Debug, PartialEq)]
pub struct FailureConfig {
    /// Persistent violation window. C# 3. (2.0 hard-codes 3 instead.)
    pub persistence_window: i64,
    /// Max norm threshold. C# null.
    pub norm_max: Option<f64>,
    /// Max loss threshold. C# null.
    pub loss_max: Option<f64>,
}

impl Default for FailureConfig {
    fn default() -> Self {
        Self { persistence_window: 3, norm_max: None, loss_max: None }
    }
}

/// 2.0's `DeltaDetectorConfig`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeltaConfig {
    pub convergence: ConvergenceConfig,
    pub stability: StabilityConfig,
    pub alignment: AlignmentDetectionConfig,
    pub emergence: EmergenceConfig,
    pub failure: FailureConfig,
}

/// 2.0's `VisualAnchor`, without the free-form `Meta` bag (no detector fills it).
#[derive(Clone, Debug, PartialEq)]
pub struct VisualAnchor {
    pub target_view: String,
    pub range_a: Option<(i64, i64)>,
    pub range_b: Option<(i64, i64)>,
}

/// 2.0's `AlignmentMap`: compare index to step in each run, `None` where a run has no step there.
#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentMap {
    pub mode: Alignment,
    pub idx_to_step_a: Vec<Option<i64>>,
    pub idx_to_step_b: Vec<Option<i64>>,
    pub compare_index: Vec<i64>,
    pub description: String,
}

/// 2.0's `CanonicalDelta`. Fields are 2.0's, snake_cased; `symbol` is the ΔX label 2.0's comparison
/// log gave each id.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryDelta {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub delta_type: DeltaType,
    pub status: DeltaStatus,
    pub explanation: String,
    pub summary_sentence: Option<String>,
    pub left_value: f64,
    pub right_value: f64,
    pub delta: f64,
    pub magnitude: f64,
    pub units: Option<String>,
    pub confidence: f64,
    /// Normalised time [0,1] where the difference is most visible; `None` for suppressed deltas.
    pub visual_anchor_time: Option<f64>,
    pub anchors: Vec<VisualAnchor>,
    pub notes: Vec<String>,
    // Convergence.
    pub tc_a: Option<i64>,
    pub tc_b: Option<i64>,
    pub delta_tc_steps: Option<i64>,
    pub delta_tc_normalized: Option<f64>,
    pub epsilon_used: Option<f64>,
    pub window_used: Option<i64>,
    pub convergence_confidence: Option<f64>,
    // Stability.
    pub score_a: Option<f64>,
    pub score_b: Option<f64>,
    pub threshold_used: Option<f64>,
    pub min_duration_used: Option<i64>,
    // Alignment.
    pub mean_align_a: Option<f64>,
    pub mean_align_b: Option<f64>,
    // Emergence.
    pub td_a: Option<i64>,
    pub td_b: Option<i64>,
    pub dominance_ratio_k: Option<f64>,
    // Failure.
    pub failed_a: Option<bool>,
    pub failed_b: Option<bool>,
    pub t_fail_a: Option<i64>,
    pub t_fail_b: Option<i64>,
    pub failure_kind_a: Option<String>,
    pub failure_kind_b: Option<String>,
}

impl GeometryDelta {
    /// 2.0's `IsMeaningful`.
    pub fn is_meaningful(&self) -> bool {
        self.status == DeltaStatus::Present
    }

    /// A delta with 2.0's record defaults: Present, confidence 1, Timing, everything else empty.
    fn new(id: &str, symbol: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            symbol: symbol.to_string(),
            name: name.to_string(),
            delta_type: DeltaType::Timing,
            status: DeltaStatus::Present,
            explanation: String::new(),
            summary_sentence: None,
            left_value: 0.0,
            right_value: 0.0,
            delta: 0.0,
            magnitude: 0.0,
            units: None,
            confidence: 1.0,
            visual_anchor_time: None,
            anchors: Vec::new(),
            notes: Vec::new(),
            tc_a: None,
            tc_b: None,
            delta_tc_steps: None,
            delta_tc_normalized: None,
            epsilon_used: None,
            window_used: None,
            convergence_confidence: None,
            score_a: None,
            score_b: None,
            threshold_used: None,
            min_duration_used: None,
            mean_align_a: None,
            mean_align_b: None,
            td_a: None,
            td_b: None,
            dominance_ratio_k: None,
            failed_a: None,
            failed_b: None,
            t_fail_a: None,
            t_fail_b: None,
            failure_kind_a: None,
            failure_kind_b: None,
        }
    }
}

/// 2.0's `DeltaComputationResult`, less the hashes.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryDeltaResult {
    pub alignment: AlignmentMap,
    /// All five deltas, suppressed ones marked.
    pub deltas: Vec<GeometryDelta>,
    /// 2.0's `ComparativeSummary`, made from the meaningful deltas only.
    pub comparative_summary: String,
}

// ========================================================================
// Entry points
// ========================================================================

/// 2.0's `ComputeDeltas`: the five detectors in causal order (failure, convergence, structural
/// emergence, evaluator alignment, stability). Suppressed deltas are included and marked.
pub fn compute(left: &GeometryRun, right: &GeometryRun, alignment: Alignment, current_time: f64, config: &DeltaConfig) -> Vec<GeometryDelta> {
    let map = create_alignment_map_with(left, right, alignment, &config.convergence);
    compute_with_map(left, right, current_time, config, &map)
}

/// `compute` with an alignment map already made.
pub fn compute_with_map(left: &GeometryRun, right: &GeometryRun, current_time: f64, config: &DeltaConfig, map: &AlignmentMap) -> Vec<GeometryDelta> {
    let left_steps = step_count(left);
    let right_steps = step_count(right);
    vec![
        detect_failure_presence(left, right, &config.failure, left_steps, right_steps),
        detect_convergence_timing(left, right, &config.convergence, left_steps, right_steps),
        detect_structural_emergence(left, right, &config.emergence),
        detect_evaluator_alignment(left, right, &config.alignment, current_time, map),
        detect_stability_oscillation(left, right, &config.stability, current_time, map),
    ]
}

/// 2.0's `ComputeDeltasWithAlignment`: the alignment map, the deltas and the auto summary.
pub fn compute_with_summary(left: &GeometryRun, right: &GeometryRun, alignment: Alignment, current_time: f64, config: &DeltaConfig) -> GeometryDeltaResult {
    let map = create_alignment_map_with(left, right, alignment, &config.convergence);
    let deltas = compute_with_map(left, right, current_time, config, &map);
    let comparative_summary = auto_summary(&deltas);
    GeometryDeltaResult { alignment: map, deltas, comparative_summary }
}

fn step_count(run: &GeometryRun) -> i64 {
    run.trajectory.timesteps.len() as i64
}

// ========================================================================
// Alignment (AlignmentMapper.cs)
// ========================================================================

/// 2.0's `AlignmentMapper.CreateAlignmentMap`, with the default convergence settings.
pub fn create_alignment_map(run_a: &GeometryRun, run_b: &GeometryRun, mode: Alignment) -> AlignmentMap {
    create_alignment_map_with(run_a, run_b, mode, &ConvergenceConfig::default())
}

/// The alignment map, with convergence read by the same settings ΔTc uses (G1).
pub fn create_alignment_map_with(run_a: &GeometryRun, run_b: &GeometryRun, mode: Alignment, convergence: &ConvergenceConfig) -> AlignmentMap {
    let steps_a = step_count(run_a);
    let steps_b = step_count(run_b);
    if steps_a == 0 || steps_b == 0 {
        return AlignmentMap { mode, idx_to_step_a: Vec::new(), idx_to_step_b: Vec::new(), compare_index: Vec::new(), description: "No data available".to_string() };
    }
    match mode {
        Alignment::ByStep => step_alignment(steps_a, steps_b),
        Alignment::ByConvergence => {
            let tc_a = find_convergence_step(run_a, convergence);
            let tc_b = find_convergence_step(run_b, convergence);
            if tc_a < 0 && tc_b < 0 {
                let mut fallback = step_alignment(steps_a, steps_b);
                fallback.description = "Neither path converged; using step alignment".to_string();
                return fallback;
            }
            let anchor_a = if tc_a >= 0 { tc_a } else { steps_a / 2 };
            let anchor_b = if tc_b >= 0 { tc_b } else { steps_b / 2 };
            anchored_alignment(steps_a, steps_b, anchor_a, anchor_b, Alignment::ByConvergence, format!("Aligned at convergence (A: step {anchor_a}, B: step {anchor_b})"))
        }
        Alignment::ByFirstInstability => {
            let ti_a = find_first_instability_step(run_a);
            let ti_b = find_first_instability_step(run_b);
            if ti_a < 0 && ti_b < 0 {
                let mut fallback = step_alignment(steps_a, steps_b);
                fallback.description = "Neither path showed instability; using step alignment".to_string();
                return fallback;
            }
            let anchor_a = if ti_a >= 0 { ti_a } else { steps_a / 4 };
            let anchor_b = if ti_b >= 0 { ti_b } else { steps_b / 4 };
            anchored_alignment(steps_a, steps_b, anchor_a, anchor_b, Alignment::ByFirstInstability, format!("Aligned at first change (A: step {anchor_a}, B: step {anchor_b})"))
        }
    }
}

fn step_alignment(steps_a: i64, steps_b: i64) -> AlignmentMap {
    let max_steps = steps_a.max(steps_b);
    let mut idx_to_step_a = Vec::with_capacity(max_steps as usize);
    let mut idx_to_step_b = Vec::with_capacity(max_steps as usize);
    let mut compare_index = Vec::with_capacity(max_steps as usize);
    for i in 0..max_steps {
        compare_index.push(i);
        // A one-step run has no span. Every compare index maps to that step.
        if max_steps <= 1 {
            idx_to_step_a.push(Some(0));
            idx_to_step_b.push(Some(0));
            continue;
        }
        let normalized_pos = i as f64 / (max_steps - 1) as f64;
        idx_to_step_a.push(Some(if steps_a <= 1 { 0 } else { (normalized_pos * (steps_a - 1) as f64) as i64 }));
        idx_to_step_b.push(Some(if steps_b <= 1 { 0 } else { (normalized_pos * (steps_b - 1) as f64) as i64 }));
    }
    AlignmentMap { mode: Alignment::ByStep, idx_to_step_a, idx_to_step_b, compare_index, description: "Aligned by training step".to_string() }
}

fn anchored_alignment(steps_a: i64, steps_b: i64, anchor_a: i64, anchor_b: i64, mode: Alignment, description: String) -> AlignmentMap {
    let pre_compare = anchor_a.max(anchor_b);
    let post_compare = (steps_a - 1 - anchor_a).max(steps_b - 1 - anchor_b);
    let total_compare = pre_compare + 1 + post_compare;
    let mut idx_to_step_a = Vec::new();
    let mut idx_to_step_b = Vec::new();
    let mut compare_index = Vec::new();
    for i in 0..total_compare {
        compare_index.push(i);
        let dist_from_anchor = i - pre_compare;
        let step_a = anchor_a + dist_from_anchor;
        idx_to_step_a.push(if step_a >= 0 && step_a < steps_a { Some(step_a) } else { None });
        let step_b = anchor_b + dist_from_anchor;
        idx_to_step_b.push(if step_b >= 0 && step_b < steps_b { Some(step_b) } else { None });
    }
    AlignmentMap { mode, idx_to_step_a, idx_to_step_b, compare_index, description }
}

/// 2.0's `AlignmentMapper.FindConvergenceStep`: velocity settles within the window, epsilon
/// `max(epsilon, multiplier·sigma)`.
///
/// G1 (ruled 2026-10-06): 2.0 pinned the window of 5 and epsilon `max(0.02, 0.5·sigma)` here
/// (AlignmentMapper.cs:183-186), so changing the config moved ΔTc but not this alignment. It
/// reads the config ΔTc reads. The defaults are 2.0's constants, so default results are unchanged.
fn find_convergence_step(run: &GeometryRun, config: &ConvergenceConfig) -> i64 {
    let steps = &run.trajectory.timesteps;
    if steps.len() < 10 {
        return -1;
    }
    let velocities: Vec<f64> = steps.iter().map(Timestep::speed).collect();
    let sigma = robust_sigma(&velocities);
    let epsilon = config.epsilon.max(sigma * config.epsilon_sigma_multiplier);
    let window = config.window.max(1);
    let count = steps.len() as i64;
    let mut i = window;
    while i < count - window {
        let base_value = velocities[i as usize];
        let mut stable = true;
        let mut j = 1;
        while j <= window && i + j < count {
            if (velocities[(i + j) as usize] - base_value).abs() >= epsilon {
                stable = false;
                break;
            }
            j += 1;
        }
        if stable {
            return i;
        }
        i += 1;
    }
    -1
}

/// 2.0's `AlignmentMapper.FindFirstInstabilityStep`: the first absolute curvature above
/// `max(2·mean, 0.3)` of the absolute curvature.
///
/// G2 (ruled 2026-10-06): 2.0 took the mean and the comparison on signed curvature
/// (AlignmentMapper.cs:217-219), while the stability detector uses absolute curvature, so the two
/// disagreed about "unstable" for a run that bends mostly one way. Both use absolute curvature.
fn find_first_instability_step(run: &GeometryRun) -> i64 {
    let steps = &run.trajectory.timesteps;
    if steps.len() < 3 {
        return -1;
    }
    let curvatures: Vec<f64> = steps.iter().map(|step| step.curvature.abs()).collect();
    let mean = average(&curvatures);
    let threshold = (mean * 2.0).max(0.3);
    for (i, curvature) in curvatures.iter().enumerate().skip(1) {
        if *curvature > threshold {
            return i as i64;
        }
    }
    -1
}

/// 2.0's `AlignmentMapper.RobustSigma`: 1.4826 times the median absolute deviation, and 0 for
/// fewer than 5 values. The median is the element at `len / 2` of the sorted values, as 2.0 took it.
pub fn robust_sigma(values: &[f64]) -> f64 {
    if values.len() < 5 {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(cmp_f64);
    let median = sorted[sorted.len() / 2];
    let mut deviations: Vec<f64> = values.iter().map(|value| (value - median).abs()).collect();
    deviations.sort_by(cmp_f64);
    let mad = deviations[deviations.len() / 2];
    1.4826 * mad
}

// ========================================================================
// Detectors (CanonicalDeltaService.cs)
// ========================================================================

/// ΔF. 2.0 `DetectFailurePresence` - "Did something break?"
fn detect_failure_presence(left: &GeometryRun, right: &GeometryRun, config: &FailureConfig, left_steps: i64, right_steps: i64) -> GeometryDelta {
    let (left_has, left_time, left_kind) = has_persistent_failure(left, config.persistence_window);
    let (right_has, right_time, right_kind) = has_persistent_failure(right, config.persistence_window);

    // G3 (ruled 2026-10-06): 2.0 multiplied the normalised time by the step count
    // (CanonicalDeltaService.cs:127-128) while the detectors normalise by count - 1, so a failure
    // on the last step reported one step past the end. It is count - 1 here, rounded to the
    // nearest step, so a time computed as i / (n - 1) comes back as step i.
    let fail_step = |time: f64, steps: i64| (time * (steps - 1).max(0) as f64).round() as i64;
    let left_fail_step = fail_step(left_time, left_steps);
    let right_fail_step = fail_step(right_time, right_steps);

    let mut delta = GeometryDelta::new(delta_ids::FAILURE_PRESENCE, SYMBOL_FAILURE, "Failure Events");

    // Suppression: neither run fails.
    if !left_has && !right_has {
        delta.explanation = "No failure detected".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.notes = vec!["Neither run experienced failure events".to_string()];
        delta.failed_a = Some(false);
        delta.failed_b = Some(false);
        return delta;
    }

    let explanation;
    let visual_anchor_time;
    if left_has && right_has {
        if (left_time - right_time).abs() <= 1e-12 {
            explanation = "Both paths experienced instability; the failures were simultaneous".to_string();
        } else {
            let earlier = if left_time < right_time { "Path A" } else { "Path B" };
            explanation = format!("Both paths experienced instability; {earlier} first");
        }
        visual_anchor_time = left_time.min(right_time);
    } else if left_has {
        explanation = format!("Only Path A experienced {left_kind} near step {left_fail_step}");
        visual_anchor_time = left_time;
    } else {
        explanation = format!("Only Path B experienced {right_kind} near step {right_fail_step}");
        visual_anchor_time = right_time;
    }

    delta.explanation = explanation;
    delta.status = DeltaStatus::Present;
    delta.left_value = if left_has { 1.0 } else { 0.0 };
    delta.right_value = if right_has { 1.0 } else { 0.0 };
    delta.delta = (if right_has { 1.0 } else { 0.0 }) - (if left_has { 1.0 } else { 0.0 });
    delta.magnitude = 1.0;
    delta.visual_anchor_time = Some(visual_anchor_time);
    delta.delta_type = DeltaType::Event;
    delta.anchors = vec![VisualAnchor {
        target_view: "loss".to_string(),
        range_a: if left_has { Some((left_fail_step, left_fail_step + 10)) } else { None },
        range_b: if right_has { Some((right_fail_step, right_fail_step + 10)) } else { None },
    }];
    delta.failed_a = Some(left_has);
    delta.failed_b = Some(right_has);
    delta.t_fail_a = if left_has { Some(left_fail_step) } else { None };
    delta.t_fail_b = if right_has { Some(right_fail_step) } else { None };
    delta.failure_kind_a = if left_has { Some(left_kind) } else { None };
    delta.failure_kind_b = if right_has { Some(right_kind) } else { None };
    delta
}

/// ΔTc. 2.0 `DetectConvergenceTiming` - "Did one run settle earlier?"
fn detect_convergence_timing(left: &GeometryRun, right: &GeometryRun, config: &ConvergenceConfig, left_steps: i64, right_steps: i64) -> GeometryDelta {
    let l = estimate_convergence(left, config);
    let r = estimate_convergence(right, config);
    let mut delta = GeometryDelta::new(delta_ids::CONVERGENCE_TIMING, SYMBOL_CONVERGENCE, "Convergence");

    // Suppression: neither run converges.
    if l.step < 0 && r.step < 0 {
        delta.explanation = "No convergence detected".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.notes = vec!["Neither run converged within observation window".to_string()];
        delta.window_used = Some(config.window);
        return delta;
    }

    let compare_length = 1i64.max(left_steps.min(right_steps));
    let compute_confidence = |tail_length: i64, violations: i64, epsilon: f64, total_steps: i64| -> f64 {
        if total_steps <= 0 {
            return 0.5;
        }
        // Factors: tail length, violation count, noise level.
        let tail_score = 1.0f64.min(tail_length as f64 / (config.window * 2).max(10) as f64);
        let violation_score = 0.0f64.max(1.0 - violations as f64 / config.window.max(3) as f64);
        let noise_score = 0.0f64.max(1.0 - (epsilon / 0.1).min(1.0)); // lower epsilon, higher confidence
        tail_score * 0.4 + violation_score * 0.4 + noise_score * 0.2
    };

    // One run converged and the other did not: Present, not suppressed.
    if l.step < 0 || r.step < 0 {
        let left_converged = l.step >= 0;
        let converged = if left_converged { "Path A" } else { "Path B" };
        let not_converged = if left_converged { "Path B" } else { "Path A" };
        let (tail_length, violations, used_epsilon, total_steps) =
            if left_converged { (l.tail, l.violations, l.epsilon, left_steps) } else { (r.tail, r.violations, r.epsilon, right_steps) };
        let confidence = compute_confidence(tail_length, violations, used_epsilon, total_steps);

        delta.explanation = format!("{converged} stabilized; {not_converged} did not within observed steps");
        delta.status = DeltaStatus::Present;
        delta.left_value = l.time;
        delta.right_value = r.time;
        // G4 (ruled 2026-10-06): 2.0 set +1 whichever run converged (CanonicalDeltaService.cs:273).
        // The sign follows the both-converged branch, right minus left: a run that never settles
        // is later than one that did, so +1 when only A converged and -1 when only B did.
        delta.delta = if left_converged { 1.0 } else { -1.0 };
        delta.magnitude = 1.0;
        delta.confidence = confidence;
        delta.visual_anchor_time = Some(if left_converged { l.time } else { r.time });
        delta.delta_type = DeltaType::Timing;
        delta.anchors = vec![VisualAnchor {
            target_view: "loss".to_string(),
            range_a: Some(if l.step >= 0 { (l.step, l.step + config.window) } else { (left_steps - config.window, left_steps) }),
            range_b: Some(if r.step >= 0 { (r.step, r.step + config.window) } else { (right_steps - config.window, right_steps) }),
        }];
        delta.tc_a = if l.step >= 0 { Some(l.step) } else { None };
        delta.tc_b = if r.step >= 0 { Some(r.step) } else { None };
        delta.epsilon_used = Some(used_epsilon);
        delta.convergence_confidence = Some(confidence);
        delta.window_used = Some(config.window);
        return delta;
    }

    let step_diff = r.step - l.step;
    let abs_step_diff = step_diff.abs();
    let normalized_delta = step_diff as f64 / compare_length as f64;

    // Suppression uses only ResolutionSteps, never the normalised difference.
    if abs_step_diff < config.resolution_steps {
        delta.explanation = "Similar convergence timing".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = l.time;
        delta.right_value = r.time;
        delta.delta = r.time - l.time;
        delta.magnitude = (r.time - l.time).abs();
        delta.notes = vec![format!("Both runs converged near step {} (Δ={} < {} steps)", l.step, abs_step_diff, config.resolution_steps)];
        delta.tc_a = Some(l.step);
        delta.tc_b = Some(r.step);
        delta.delta_tc_steps = Some(step_diff);
        delta.delta_tc_normalized = Some(normalized_delta);
        delta.window_used = Some(config.window);
        return delta;
    }

    let left_confidence = compute_confidence(l.tail, l.violations, l.epsilon, left_steps);
    let right_confidence = compute_confidence(r.tail, r.violations, r.epsilon, right_steps);
    let combined_confidence = left_confidence.min(right_confidence);

    // stepDiff > 0 means B converged later, so A was faster.
    let faster = if step_diff > 0 { "Path A" } else { "Path B" };
    delta.explanation = format!("{faster} converged {abs_step_diff} steps earlier");
    delta.summary_sentence = Some(format!("{faster} settled {abs_step_diff} steps before the other path"));
    delta.status = DeltaStatus::Present;
    delta.left_value = l.time;
    delta.right_value = r.time;
    delta.delta = r.time - l.time;
    delta.magnitude = (r.time - l.time).abs();
    delta.confidence = combined_confidence;
    delta.visual_anchor_time = Some(l.time.min(r.time));
    delta.delta_type = DeltaType::Timing;
    delta.anchors = vec![VisualAnchor {
        target_view: "loss".to_string(),
        range_a: Some((l.step, l.step + config.window)),
        range_b: Some((r.step, r.step + config.window)),
    }];
    delta.tc_a = Some(l.step);
    delta.tc_b = Some(r.step);
    delta.delta_tc_steps = Some(step_diff);
    delta.delta_tc_normalized = Some(normalized_delta);
    delta.epsilon_used = Some(l.epsilon);
    delta.convergence_confidence = Some(combined_confidence);
    delta.window_used = Some(config.window);
    delta
}

/// ΔTd. 2.0 `DetectStructuralEmergence` - "Did structure emerge differently?"
fn detect_structural_emergence(left: &GeometryRun, right: &GeometryRun, config: &EmergenceConfig) -> GeometryDelta {
    let (left_step, left_time, left_trigger) = find_dominance_onset(left, config);
    let (right_step, right_time, right_trigger) = find_dominance_onset(right, config);
    let mut delta = GeometryDelta::new(delta_ids::STRUCTURAL_EMERGENCE, SYMBOL_EMERGENCE, "Structure");

    // Suppression: dominance never achieved in either run.
    if left_step < 0 && right_step < 0 {
        delta.explanation = "No structural dominance".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.notes = vec!["Neither run achieved eigenvalue dominance".to_string()];
        delta.dominance_ratio_k = Some(config.k);
        return delta;
    }

    let anchor_duration = |trigger: Option<&str>| if trigger == Some("recurrence") { config.recurrence_window } else { config.window };

    // Only one run achieves dominance.
    if left_step < 0 {
        let duration = anchor_duration(right_trigger);
        let trigger_note = if right_trigger == Some("recurrence") { " (detected via recurrence)" } else { "" };
        delta.explanation = format!("Path B developed dominant direction; Path A remained distributed{trigger_note}");
        delta.status = DeltaStatus::Present;
        delta.left_value = -1.0;
        delta.right_value = right_time;
        delta.delta = 1.0;
        delta.magnitude = 1.0;
        delta.visual_anchor_time = Some(right_time);
        delta.delta_type = DeltaType::Timing;
        delta.anchors = vec![VisualAnchor { target_view: "eigenvalues".to_string(), range_a: None, range_b: Some((right_step, right_step + duration)) }];
        delta.td_b = Some(right_step);
        delta.dominance_ratio_k = Some(config.k);
        return delta;
    }
    if right_step < 0 {
        let duration = anchor_duration(left_trigger);
        let trigger_note = if left_trigger == Some("recurrence") { " (detected via recurrence)" } else { "" };
        delta.explanation = format!("Path A developed dominant direction; Path B remained distributed{trigger_note}");
        delta.status = DeltaStatus::Present;
        delta.left_value = left_time;
        delta.right_value = -1.0;
        delta.delta = -1.0;
        delta.magnitude = 1.0;
        delta.visual_anchor_time = Some(left_time);
        delta.delta_type = DeltaType::Timing;
        delta.anchors = vec![VisualAnchor { target_view: "eigenvalues".to_string(), range_a: Some((left_step, left_step + duration)), range_b: None }];
        delta.td_a = Some(left_step);
        delta.dominance_ratio_k = Some(config.k);
        return delta;
    }

    let step_diff = (right_step - left_step).abs();

    // Suppression: both runs achieve dominance together.
    if step_diff < config.resolution_steps {
        delta.explanation = "Similar emergence timing".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = left_time;
        delta.right_value = right_time;
        delta.delta = right_time - left_time;
        delta.magnitude = (right_time - left_time).abs();
        delta.notes = vec![format!("Both runs achieved dominance near step {left_step}")];
        delta.td_a = Some(left_step);
        delta.td_b = Some(right_step);
        delta.dominance_ratio_k = Some(config.k);
        return delta;
    }

    let earlier = if right_step > left_step { "Path A" } else { "Path B" };
    let earlier_trigger = if right_step > left_step { left_trigger } else { right_trigger };
    let trigger_suffix = if earlier_trigger == Some("recurrence") { " (structure forming)" } else { "" };
    delta.explanation = format!("{earlier} developed dominant direction {step_diff} steps earlier{trigger_suffix}");
    delta.summary_sentence = Some(format!("{earlier} achieved structural dominance {step_diff} steps before the other"));
    delta.status = DeltaStatus::Present;
    delta.left_value = left_time;
    delta.right_value = right_time;
    delta.delta = right_time - left_time;
    delta.magnitude = (right_time - left_time).abs();
    delta.visual_anchor_time = Some(left_time.min(right_time));
    delta.delta_type = DeltaType::Timing;
    delta.anchors = vec![VisualAnchor {
        target_view: "eigenvalues".to_string(),
        range_a: Some((left_step, left_step + anchor_duration(left_trigger))),
        range_b: Some((right_step, right_step + anchor_duration(right_trigger))),
    }];
    delta.td_a = Some(left_step);
    delta.td_b = Some(right_step);
    delta.dominance_ratio_k = Some(config.k);
    delta
}

/// ΔĀ. 2.0 `DetectEvaluatorAlignment` - "Did internal evaluators agree differently?"
///
/// G5 (ruled 2026-10-06): 2.0 called this "evaluator agreement" (CanonicalDeltaService.cs:1010,
/// 1070), but the measure is the first eigenvalue's share of the eigenvalue sum, λ1/Σλ: how
/// concentrated the spectrum is, not how much the professors agree. The words now say
/// "spectrum concentration". The id and the symbol ΔĀ are 2.0's, so 2.0 bundles still match.
fn detect_evaluator_alignment(left: &GeometryRun, right: &GeometryRun, config: &AlignmentDetectionConfig, current_time: f64, alignment: &AlignmentMap) -> GeometryDelta {
    let p = persistence_weighted_alignment_delta(left, right, current_time, config, alignment);
    let mut delta = GeometryDelta::new(delta_ids::EVALUATOR_ALIGNMENT, SYMBOL_ALIGNMENT, "Spectrum concentration");

    if p.paired == 0 {
        delta.explanation = "No aligned samples".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.notes = vec!["Agreement was not compared where either side of a compare index is null".to_string()];
        return delta;
    }

    let difference = p.right_mean - p.left_mean;
    // The persistence score is the magnitude.
    let magnitude = p.score;

    // G6, kept as 2.0 (ruled 2026-10-06): the sign and the "higher" path come from the mean over
    // every paired step, while the magnitude and the step count come from the longest sustained
    // segment (CanonicalDeltaService.cs:526-527, 569). The two can disagree, but they did not on
    // any pair measured, and taking the direction from the segment would make the change shown
    // differ from right minus left of the means shown beside it.

    // Suppression: the persistence-weighted difference is negligible.
    if magnitude < config.delta_floor {
        delta.explanation = "Similar spectrum concentration".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = p.left_mean;
        delta.right_value = p.right_mean;
        delta.delta = difference;
        delta.magnitude = magnitude;
        delta.notes = vec!["Persistence-weighted concentration difference below threshold".to_string()];
        delta.mean_align_a = Some(p.left_mean);
        delta.mean_align_b = Some(p.right_mean);
        return delta;
    }

    // Suppression: the sustained segment is too short.
    if p.segment_duration < config.min_persistence_steps {
        delta.explanation = "Brief concentration difference".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = p.left_mean;
        delta.right_value = p.right_mean;
        delta.delta = difference;
        delta.magnitude = magnitude;
        delta.notes = vec![format!("Sustained segment ({} steps) below minimum ({})", p.segment_duration, config.min_persistence_steps)];
        delta.mean_align_a = Some(p.left_mean);
        delta.mean_align_b = Some(p.right_mean);
        return delta;
    }

    let higher = if difference > 0.0 { "Path B" } else { "Path A" };
    delta.explanation = format!("{higher} kept a more concentrated spectrum over {} steps", p.segment_duration);
    delta.summary_sentence = Some(format!("{higher} had a more concentrated spectrum (sustained {} steps)", p.segment_duration));
    delta.status = DeltaStatus::Present;
    delta.left_value = p.left_mean;
    delta.right_value = p.right_mean;
    delta.delta = difference;
    delta.magnitude = magnitude;
    delta.visual_anchor_time = Some(p.segment_start as f64 / 1i64.max(alignment.idx_to_step_a.len() as i64) as f64);
    delta.delta_type = DeltaType::Structure;
    // The anchor time is a fraction of the aligned series, and each range is that run's steps.
    delta.anchors = vec![VisualAnchor { target_view: "eigenvalues".to_string(), range_a: p.range_a, range_b: p.range_b }];
    delta.mean_align_a = Some(p.left_mean);
    delta.mean_align_b = Some(p.right_mean);
    delta
}

/// ΔO. 2.0 `DetectStabilityOscillation` - "Did one run wobble more?"
fn detect_stability_oscillation(left: &GeometryRun, right: &GeometryRun, config: &StabilityConfig, current_time: f64, alignment: &AlignmentMap) -> GeometryDelta {
    let left_length = step_count(left);
    let right_length = step_count(right);
    let paired_a = paired_steps(&alignment.idx_to_step_a, &alignment.idx_to_step_b, left_length, current_time);
    let paired_b = paired_steps(&alignment.idx_to_step_b, &alignment.idx_to_step_a, right_length, current_time);
    let mut delta = GeometryDelta::new(delta_ids::STABILITY_OSCILLATION, SYMBOL_STABILITY, "Stability");

    if paired_a.is_empty() || paired_b.is_empty() {
        delta.explanation = "No aligned samples".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.notes = vec!["Oscillation was not compared where either side of a compare index is null".to_string()];
        return delta;
    }

    let left_slice = slice_curvature(left, &paired_a);
    let right_slice = slice_curvature(right, &paired_b);
    let l = oscillation_score_with_area(&left_slice, 1.0, config);
    let r = oscillation_score_with_area(&right_slice, 1.0, config);

    let difference = r.score - l.score;
    let magnitude = difference.abs();

    // Suppression: both scores below NoiseFloor.
    if l.score < config.noise_floor && r.score < config.noise_floor {
        delta.explanation = "Both runs stable".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = l.score;
        delta.right_value = r.score;
        delta.delta = difference;
        delta.magnitude = magnitude;
        delta.notes = vec!["Both runs maintained stable trajectories (scores below noise floor)".to_string()];
        delta.score_a = Some(l.score);
        delta.score_b = Some(r.score);
        delta.threshold_used = Some(l.theta);
        delta.min_duration_used = Some(config.min_duration);
        return delta;
    }

    // Suppression: |ΔO| below DeltaFloor.
    if magnitude < config.delta_floor {
        delta.explanation = "Similar oscillation levels".to_string();
        delta.status = DeltaStatus::Suppressed;
        delta.left_value = l.score;
        delta.right_value = r.score;
        delta.delta = difference;
        delta.magnitude = magnitude;
        delta.notes = vec![format!("Oscillation difference below floor (|Δ|={} < {})", fixed(magnitude, 3), dotnet_double(config.delta_floor))];
        delta.score_a = Some(l.score);
        delta.score_b = Some(r.score);
        delta.threshold_used = Some(l.theta);
        delta.min_duration_used = Some(config.min_duration);
        return delta;
    }

    // Which path showed more oscillation.
    let peak_episode = if l.score > r.score { l.peak } else { r.peak };
    let mut explanation = if l.score > r.score { "Path A showed sustained instability during training".to_string() } else { "Path B showed sustained instability during training".to_string() };
    if peak_episode.duration > 0 {
        explanation.push_str(&format!(" ({} steps)", peak_episode.duration));
    }

    // G7, kept as 2.0 and closed (ruled 2026-10-06): the owner of the anchor is the left run on a
    // tie (>=), while the words say Path B on a tie (>) (CanonicalDeltaService.cs:694 against 679,
    // 715). A tie is below the delta floor and is suppressed before this, unless DeltaFloor is 0
    // or less.
    let owner_is_left = l.score >= r.score;
    let owner_paired = if owner_is_left { &paired_a } else { &paired_b };
    let owner_length = 1i64.max(if owner_is_left { left_length } else { right_length });
    let owner_episode = if owner_is_left { l.peak } else { r.peak };
    let owner_step = if owner_episode.start >= 0 && (owner_episode.start as usize) < owner_paired.len() { owner_paired[owner_episode.start as usize] } else { -1 };
    let visual_time = if owner_step >= 0 { owner_step as f64 / owner_length as f64 } else { 0.5 };
    let range_for = |paired: &[i64], episode: Episode| -> Option<(i64, i64)> {
        if episode.start < 0 || episode.start as usize >= paired.len() || episode.duration <= 0 {
            return None;
        }
        let end = (paired.len() as i64 - 1).min(episode.start + episode.duration - 1);
        Some((paired[episode.start as usize], paired[end as usize]))
    };

    delta.explanation = explanation;
    delta.summary_sentence = Some(format!("{} showed {} more oscillation", if l.score > r.score { "Path A" } else { "Path B" }, fixed(magnitude, 2)));
    delta.status = DeltaStatus::Present;
    delta.left_value = l.score;
    delta.right_value = r.score;
    delta.delta = difference;
    delta.magnitude = magnitude;
    delta.visual_anchor_time = Some(visual_time);
    delta.delta_type = DeltaType::Behavior;
    delta.anchors = vec![VisualAnchor {
        target_view: "curvature".to_string(),
        range_a: if l.score > r.score { range_for(&paired_a, l.peak) } else { None },
        range_b: if r.score > l.score { range_for(&paired_b, r.peak) } else { None },
    }];
    delta.score_a = Some(l.score);
    delta.score_b = Some(r.score);
    delta.threshold_used = Some(l.theta);
    delta.min_duration_used = Some(config.min_duration);
    delta
}

// ========================================================================
// Helpers
// ========================================================================

/// 2.0's `HasPersistentFailure`: (failed, normalised failure time, kind). Time is 0 and the kind
/// "instability" when it did not fail. The recorded `t` is taken as already normalised, as 2.0 did.
///
/// G8 (ruled 2026-10-06): 2.0 hard-coded three where `FailureConfig.PersistenceWindow` was meant,
/// and took the time from the third recorded failure but the kind from the first
/// (CanonicalDeltaService.cs:750, 754), so its sentence named one failure's kind at another's
/// time. The window is read, and the time and the kind both come from the failure that makes
/// the run persistent. The default window is 2.0's three.
fn has_persistent_failure(run: &GeometryRun, window: i64) -> (bool, f64, String) {
    let window = window.max(1) as usize;
    // Annotated events use the same persistence bar as inferred spikes.
    if run.failures.len() >= window {
        let mut ordered: Vec<&crate::geometry::Failure> = run.failures.iter().collect();
        ordered.sort_by(|a, b| cmp_f64(&a.t, &b.t));
        let failure = ordered[window - 1];
        let kind = if failure.category.is_empty() { "instability".to_string() } else { failure.category.clone() };
        return (true, failure.t, kind);
    }

    // Divergence: the velocity explodes for three steps running.
    let steps = &run.trajectory.timesteps;
    if steps.len() > 10 {
        let mut divergence_count = 0;
        for i in 1..steps.len() {
            let current_vel = steps[i].speed();
            let prior_vel = steps[i.saturating_sub(5)].speed();
            if current_vel > 10.0 * prior_vel && current_vel > 1.0 {
                divergence_count += 1;
                if divergence_count >= window {
                    return (true, (i + 1 - window) as f64 / (steps.len() - 1) as f64, "divergence".to_string());
                }
            } else {
                divergence_count = 0;
            }
        }
    }

    // Eigenvalue collapse: the sum stays under 0.001 for three steps running.
    let eigenvalues = &run.geometry.eigenvalues;
    if eigenvalues.len() > 10 {
        let mut collapse_count = 0;
        for (i, step) in eigenvalues.iter().enumerate() {
            if sum(&step.values) < 0.001 {
                collapse_count += 1;
                if collapse_count >= window {
                    return (true, (i + 1 - window) as f64 / (eigenvalues.len() - 1) as f64, "geometry collapse".to_string());
                }
            } else {
                collapse_count = 0;
            }
        }
    }

    (false, 0.0, "instability".to_string())
}

struct Convergence {
    step: i64,
    time: f64,
    epsilon: f64,
    tail: i64,
    violations: i64,
}

/// 2.0's `EstimateConvergenceTimeWithConfidence`. Convergence is the velocity staying within an
/// epsilon band for `window` steps.
fn estimate_convergence(run: &GeometryRun, config: &ConvergenceConfig) -> Convergence {
    let steps = &run.trajectory.timesteps;
    let count = steps.len() as i64;
    if count < config.window * 2 {
        return Convergence { step: -1, time: -1.0, epsilon: 0.0, tail: 0, violations: 0 };
    }

    // Adaptive epsilon: max(base, sigma * multiplier), not the sum.
    let velocities: Vec<f64> = steps.iter().map(Timestep::speed).collect();
    let sigma = robust_sigma(&velocities);
    let epsilon = config.epsilon.max(sigma * config.epsilon_sigma_multiplier);

    let mut convergence_step: i64 = -1;
    let mut convergence_time: f64 = -1.0;
    let mut i = config.window;
    while i < count - config.window {
        let base_value = velocities[i as usize];
        let mut stable = true;
        let mut j = 1;
        while j <= config.window && i + j < count {
            if (velocities[(i + j) as usize] - base_value).abs() >= epsilon {
                stable = false;
                break;
            }
            j += 1;
        }
        if stable {
            convergence_step = i;
            convergence_time = i as f64 / (count - 1) as f64;
            break;
        }
        i += 1;
    }

    if convergence_step < 0 {
        return Convergence { step: -1, time: -1.0, epsilon, tail: 0, violations: 0 };
    }

    // Tail length: how many steps stay in the band after the window.
    let base_at_convergence = velocities[convergence_step as usize];
    let mut tail = 0;
    let mut i = convergence_step + config.window;
    while i < count {
        if (velocities[i as usize] - base_at_convergence).abs() < epsilon {
            tail += 1;
        } else {
            break;
        }
        i += 1;
    }

    // Violations: how many times the signal leaves the band after convergence.
    let mut violations = 0;
    let mut in_band = true;
    let mut i = convergence_step + 1;
    while i < count {
        let now_in_band = (velocities[i as usize] - base_at_convergence).abs() < epsilon;
        if in_band && !now_in_band {
            violations += 1;
        }
        in_band = now_in_band;
        i += 1;
    }

    Convergence { step: convergence_step, time: convergence_time, epsilon, tail, violations }
}

#[derive(Clone, Copy, Debug)]
struct Episode {
    start: i64,
    duration: i64,
    score: f64,
}

struct Oscillation {
    score: f64,
    theta: f64,
    peak: Episode,
}

/// 2.0's `ComputeOscillationScoreWithArea` on a run's curvature series: the area above an adaptive
/// threshold, summed over episodes of at least `min_duration` steps. `up_to_time` is a fraction of
/// the series.
fn oscillation_score_with_area(curvature: &[f64], up_to_time: f64, config: &StabilityConfig) -> Oscillation {
    let count = curvature.len() as i64;
    if count < config.min_duration + 2 {
        return Oscillation { score: 0.0, theta: 0.0, peak: Episode { start: -1, duration: 0, score: 0.0 } };
    }

    let max_idx = ((up_to_time * (count - 1) as f64) as i64).max(config.min_duration).min(count - 1);

    // Absolute curvatures.
    let curvatures: Vec<f64> = curvature.iter().take((max_idx + 1) as usize).map(|c| c.abs()).collect();

    // Adaptive threshold: max(median x multiplier, sigma x multiplier), or the fixed theta.
    let mut sorted = curvatures.clone();
    sorted.sort_by(cmp_f64);
    let median_abs = sorted[sorted.len() / 2];
    let sigma_abs = robust_sigma(&curvatures);

    let theta_eff = if config.theta > 0.0 {
        config.theta
    } else {
        let from_median = median_abs * config.theta_multiplier;
        let from_sigma = sigma_abs * config.theta_sigma_multiplier;
        // A minimum threshold keeps noise from triggering.
        from_median.max(from_sigma).max(0.01)
    };

    // Episodes: contiguous segments where |C| > theta for at least MinDuration.
    let mut episodes: Vec<Episode> = Vec::new();
    let mut episode_start: i64 = -1;
    let mut current_area = 0.0;
    let mut current_duration: i64 = 0;

    for i in 0..=max_idx {
        let value = curvatures[i as usize];
        let excess = 0.0f64.max(value - theta_eff);
        if value > theta_eff {
            if episode_start < 0 {
                episode_start = i;
            }
            current_duration += 1;
            current_area += excess;
        } else {
            // The episode ended: does it qualify?
            if current_duration >= config.min_duration && current_area >= config.noise_floor {
                episodes.push(Episode { start: episode_start, duration: current_duration, score: current_area });
            }
            episode_start = -1;
            current_duration = 0;
            current_area = 0.0;
        }
    }
    // The trailing episode.
    if current_duration >= config.min_duration && current_area >= config.noise_floor {
        episodes.push(Episode { start: episode_start, duration: current_duration, score: current_area });
    }

    let mut total = 0.0;
    for episode in &episodes {
        total += episode.score;
    }

    // Peak episode: the highest score, the first of equals (a stable descending sort's first).
    let mut peak = Episode { start: -1, duration: 0, score: 0.0 };
    if let Some(first) = episodes.first() {
        peak = *first;
        for episode in &episodes[1..] {
            if cmp_f64(&episode.score, &peak.score) == Ordering::Greater {
                peak = *episode;
            }
        }
    }

    Oscillation { score: total, theta: theta_eff, peak }
}

struct PersistenceDelta {
    score: f64,
    segment_start: i64,
    segment_duration: i64,
    left_mean: f64,
    right_mean: f64,
    range_a: Option<(i64, i64)>,
    range_b: Option<(i64, i64)>,
    paired: i64,
}

/// 2.0's `ComputePersistenceWeightedAlignmentDelta`: the mean |A_B(t) - A_A(t)| over the longest
/// stretch where it exceeds `segment_epsilon`, on the compare indices both runs have.
fn persistence_weighted_alignment_delta(left: &GeometryRun, right: &GeometryRun, current_time: f64, config: &AlignmentDetectionConfig, alignment: &AlignmentMap) -> PersistenceDelta {
    let none = PersistenceDelta { score: 0.0, segment_start: 0, segment_duration: 0, left_mean: 0.0, right_mean: 0.0, range_a: None, range_b: None, paired: 0 };
    let left_eigen = &left.geometry.eigenvalues;
    let right_eigen = &right.geometry.eigenvalues;
    if left_eigen.is_empty() || right_eigen.is_empty() {
        return none;
    }

    let compare_count = alignment.idx_to_step_a.len().min(alignment.idx_to_step_b.len()) as i64;
    let mut limit = if compare_count <= 1 { 0 } else { (current_time * (compare_count - 1) as f64) as i64 };
    limit = limit.max(0).min(0i64.max(compare_count - 1));

    // D(t) only where both sides of the compare index exist.
    let mut left_alignments: Vec<f64> = Vec::new();
    let mut right_alignments: Vec<f64> = Vec::new();
    let mut differences: Vec<f64> = Vec::new();
    let mut paired_compare: Vec<i64> = Vec::new();

    let mut i = 0;
    while i <= limit && i < compare_count {
        let step_a = alignment.idx_to_step_a[i as usize];
        let step_b = alignment.idx_to_step_b[i as usize];
        i += 1;
        let (Some(step_a), Some(step_b)) = (step_a, step_b) else { continue };
        if step_a < 0 || step_a >= left_eigen.len() as i64 || step_b < 0 || step_b >= right_eigen.len() as i64 {
            continue;
        }
        let left_values = &left_eigen[step_a as usize].values;
        let right_values = &right_eigen[step_b as usize].values;

        // G9 (ruled 2026-10-06): 2.0 counted a step without enough eigenvalues, or with a sum of
        // 0.001 or less, as alignment 0 (CanonicalDeltaService.cs:1064-1078), which pulled the
        // mean and the difference toward the other run. Such a step is left out.
        let concentration = |values: &[f64]| {
            let total = sum(values);
            (values.len() as i64 >= config.min_evaluators && total > 0.001).then(|| values[0] / total)
        };
        let (Some(left_alignment), Some(right_alignment)) = (concentration(left_values), concentration(right_values)) else {
            continue;
        };

        left_alignments.push(left_alignment);
        right_alignments.push(right_alignment);
        differences.push((right_alignment - left_alignment).abs());
        paired_compare.push(i - 1);
    }

    if differences.is_empty() {
        return none;
    }

    // The longest stretch where D(t) > SegmentEpsilon; the first of equals wins.
    let mut longest_start: i64 = 0;
    let mut longest_duration: i64 = 0;
    let mut longest_area = 0.0;
    let mut current_start: i64 = -1;
    let mut current_duration: i64 = 0;
    let mut current_area = 0.0;
    for (i, difference) in differences.iter().enumerate() {
        if *difference > config.segment_epsilon {
            if current_start < 0 {
                current_start = i as i64;
            }
            current_duration += 1;
            current_area += difference;
            if current_duration > longest_duration {
                longest_start = current_start;
                longest_duration = current_duration;
                longest_area = current_area;
            }
        } else {
            current_start = -1;
            current_duration = 0;
            current_area = 0.0;
        }
    }

    // The score is the average difference over that stretch.
    let score = if longest_duration > 0 { longest_area / longest_duration as f64 } else { 0.0 };
    let left_mean = if left_alignments.is_empty() { 0.0 } else { average(&left_alignments) };
    let right_mean = if right_alignments.is_empty() { 0.0 } else { average(&right_alignments) };

    let compare_start = paired_compare[longest_start as usize];
    let end_index = (paired_compare.len() as i64 - 1).min(longest_start + longest_duration.max(1) - 1);
    let compare_end = paired_compare[end_index as usize];
    let mut range_a = None;
    let mut range_b = None;
    if longest_duration > 0 {
        let a0 = alignment.idx_to_step_a[compare_start as usize];
        let a1 = alignment.idx_to_step_a[compare_end as usize];
        let b0 = alignment.idx_to_step_b[compare_start as usize];
        let b1 = alignment.idx_to_step_b[compare_end as usize];
        if let (Some(start), Some(end)) = (a0, a1) {
            range_a = Some((start, end));
        }
        if let (Some(start), Some(end)) = (b0, b1) {
            range_b = Some((start, end));
        }
    }

    PersistenceDelta { score, segment_start: compare_start, segment_duration: longest_duration, left_mean, right_mean, range_a, range_b, paired: paired_compare.len() as i64 }
}

/// 2.0's `PairedSteps`: this run's steps at the compare indices the other run also has, up to
/// `up_to_time` of this run.
fn paired_steps(mine: &[Option<i64>], other: &[Option<i64>], owner_length: i64, up_to_time: f64) -> Vec<i64> {
    let mut paired = Vec::new();
    let count = mine.len().min(other.len());
    let max_step = if owner_length <= 1 { 0 } else { (up_to_time.clamp(0.0, 1.0) * (owner_length - 1) as f64) as i64 };
    for i in 0..count {
        let (Some(step), Some(_)) = (mine[i], other[i]) else { continue };
        if step < 0 || step > max_step {
            continue;
        }
        paired.push(step);
    }
    paired
}

/// 2.0's `SliceToSteps`, as the curvature series of the chosen steps (the only thing the
/// oscillation score reads).
fn slice_curvature(run: &GeometryRun, steps: &[i64]) -> Vec<f64> {
    let source = &run.trajectory.timesteps;
    steps.iter().filter(|step| **step >= 0 && (**step as usize) < source.len()).map(|step| source[*step as usize].curvature).collect()
}

/// 2.0's `FindDominanceOnsetTimeWithRecurrence`: (step, normalised time, trigger), or (-1, -1,
/// None). Accepts emergence on sustained dominance (a segment of `window` steps) or, failing that,
/// on recurrence (`min_recurrence_count` segments starting within `recurrence_window`).
fn find_dominance_onset(run: &GeometryRun, config: &EmergenceConfig) -> (i64, f64, Option<&'static str>) {
    let eigenvalues = &run.geometry.eigenvalues;
    if (eigenvalues.len() as i64) < config.min_segment_length {
        return (-1, -1.0, None);
    }

    // Every dominance segment of at least MinSegmentLength consecutive steps.
    let mut segments: Vec<(i64, i64)> = Vec::new();
    let mut segment_start: i64 = -1;
    let mut segment_length: i64 = 0;
    for (i, step) in eigenvalues.iter().enumerate() {
        let values = &step.values;
        let mut is_dominant = false;
        if values.len() >= 2 {
            let lambda1 = values[0];
            let lambda2 = values[1];
            is_dominant = lambda2 > 0.001 && lambda1 > config.k * lambda2;
        }
        if is_dominant {
            if segment_start < 0 {
                segment_start = i as i64;
            }
            segment_length += 1;
        } else {
            if segment_length >= config.min_segment_length {
                segments.push((segment_start, segment_length));
            }
            segment_start = -1;
            segment_length = 0;
        }
    }
    if segment_length >= config.min_segment_length {
        segments.push((segment_start, segment_length));
    }

    if segments.is_empty() {
        return (-1, -1.0, None);
    }

    let last = (eigenvalues.len() - 1) as f64;

    // Condition A: sustained dominance.
    for (start, length) in &segments {
        if *length >= config.window {
            return (*start, *start as f64 / last, Some("sustained"));
        }
    }

    // Condition B: recurrence.
    for (window_start, _) in &segments {
        let window_end = window_start + config.recurrence_window;
        let in_window: Vec<&(i64, i64)> = segments.iter().filter(|(start, _)| start >= window_start && *start < window_end).collect();
        if in_window.len() as i64 >= config.min_recurrence_count {
            let start = in_window[0].0;
            return (start, start as f64 / last, Some("recurrence"));
        }
    }

    (-1, -1.0, None)
}

/// 2.0's `GenerateAutoSummary`, from the meaningful deltas (suppressed ones are left out, as 2.0's
/// list left them out). At most 25 words; past that it falls back to the first delta alone.
pub fn auto_summary(deltas: &[GeometryDelta]) -> String {
    // A withheld delta (`Indeterminate`, which 2.0 never produced) does not speak either.
    let deltas: Vec<&GeometryDelta> = deltas.iter().filter(|delta| delta.status == DeltaStatus::Present).collect();
    if deltas.is_empty() {
        return "No meaningful divergence observed between paths.".to_string();
    }
    let primary = deltas[0];
    let alone = || primary.summary_sentence.clone().unwrap_or_else(|| format!("{}.", primary.explanation));
    if deltas.len() == 1 {
        return alone();
    }

    // A modifier from the second delta.
    let modifier = match deltas[1].id.as_str() {
        delta_ids::CONVERGENCE_TIMING => " with different convergence timing",
        delta_ids::STABILITY_OSCILLATION => " alongside stability differences",
        delta_ids::STRUCTURAL_EMERGENCE => " and distinct structural emergence",
        delta_ids::EVALUATOR_ALIGNMENT => " while spectrum concentration differed",
        delta_ids::FAILURE_PRESENCE => " compounded by failure events",
        _ => "",
    };
    let primary_text = primary.summary_sentence.clone().unwrap_or_else(|| primary.explanation.clone());
    let summary = format!("{primary_text}{modifier}.");
    if summary.split(' ').filter(|word| !word.is_empty()).count() > 25 {
        return alone();
    }
    summary
}

// ========================================================================
// .NET number behaviour
// ========================================================================

/// `Double.CompareTo`: NaN sorts before every number, and NaN equals NaN. 2.0's `OrderBy` on doubles.
fn cmp_f64(a: &f64, b: &f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
    }
}

/// LINQ `Sum()` over doubles: added in order from 0.
fn sum(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |total, value| total + value)
}

/// LINQ `Average()` over doubles: the in-order sum over the count.
fn average(values: &[f64]) -> f64 {
    sum(values) / values.len() as f64
}

/// .NET's `ToString("F<digits>")`: fixed digits, a tie rounding away from zero (Rust rounds a tie
/// to even, so 0.0625 would print as 0.062 where .NET prints 0.063).
pub fn fixed(value: f64, digits: usize) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() };
    }
    let expansion = format!("{:.*}", digits + 25, value.abs());
    let tail = &expansion[expansion.len() - 25..];
    let is_tie = tail.starts_with('5') && tail[1..].bytes().all(|byte| byte == b'0');
    if is_tie {
        let scale = 10f64.powi(digits as i32);
        let rounded = (value.abs() * scale).ceil() / scale;
        let text = format!("{:.*}", digits, rounded);
        return if value < 0.0 { format!("-{text}") } else { text };
    }
    format!("{:.*}", digits, value)
}

/// .NET's default `double.ToString()`: the shortest round-trip digits, in exponent form (`1E-07`,
/// `1.5E+15`) below 1e-5 and from 1e15 up, plain otherwise.
pub fn dotnet_double(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() };
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0".to_string() } else { "0".to_string() };
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((scientific.as_str(), "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    if exponent < -5 || exponent >= 15 {
        let sign = if exponent < 0 { '-' } else { '+' };
        return format!("{mantissa}E{sign}{:02}", exponent.abs());
    }
    format!("{value}")
}

// ========================================================================
// Export contract: step axis and score source (spec, "Geometry export contract")
// ========================================================================

/// Why steps that are not time cannot carry a timing or oscillation reading.
pub const NOT_TIME: &str = "steps are checkpoint × item, not time";
/// Why repeating scores cannot carry a failure reading.
pub const SCORES_REPEAT: &str = "its failures are score dips, and the scores repeat";

/// Withhold the deltas a run's own layout invalidates, with the reason, as the export contract
/// says. A run that states neither field is untouched, so 2.0's results stand for 2.0's files.
pub fn withhold_by_layout(deltas: &mut [GeometryDelta], left: &crate::geometry::RunMetadata, right: &crate::geometry::RunMetadata) {
    let not_time = !left.steps_are_time() || !right.steps_are_time();
    let repeated: Vec<&str> = [left.repeated_scores(), right.repeated_scores()].into_iter().flatten().collect();
    for delta in deltas.iter_mut() {
        let reason = match delta.id.as_str() {
            delta_ids::CONVERGENCE_TIMING | delta_ids::STABILITY_OSCILLATION if not_time => Some(NOT_TIME.to_string()),
            delta_ids::FAILURE_PRESENCE if not_time => Some(NOT_TIME.to_string()),
            delta_ids::FAILURE_PRESENCE if !repeated.is_empty() => {
                let how = if repeated.contains(&"replayed") { "replayed after the first epoch" } else { "fixed per item" };
                Some(format!("{SCORES_REPEAT} ({how})"))
            }
            _ => None,
        };
        if let Some(reason) = reason {
            delta.status = DeltaStatus::Indeterminate;
            delta.notes.insert(0, format!("2.0's reading was: {}", delta.explanation));
            delta.explanation = format!("Withheld: {reason}");
            delta.summary_sentence = None;
            delta.visual_anchor_time = None;
        }
    }
}
