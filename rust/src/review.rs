//! A pair is either two inference runs or two training entries.
//! The inference page reports ΔF, ΔO, and ΔTc. ΔTc is withheld when a
//! steady-state milestone is missing. ΔTd and ΔĀ stay off that page.
//! A training page does not compute those deltas.

use serde::{Deserialize, Serialize};

use crate::open::{InferenceRun, Side, TrainingEntry};
use crate::readings::{self, Band};
use crate::runtrace;
use crate::shape::{self, RunShape};
use crate::stats::{self, Interval, Support};
use crate::views::{self, DifferencePoint, Segment};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceReview {
    pub left_label: String,
    pub right_label: String,
    pub signal: String,
    pub unit: String,
    pub left: Vec<Option<f64>>,
    pub right: Vec<Option<f64>>,
    pub left_band: Vec<Option<Band>>,
    pub right_band: Vec<Option<Band>>,
    pub left_marks: Vec<usize>,
    pub right_marks: Vec<usize>,
    pub left_steady: Option<usize>,
    pub right_steady: Option<usize>,
    pub left_throughput: Vec<f64>,
    pub right_throughput: Vec<f64>,
    /// Memory in MiB at the same steps, when both sides recorded it.
    pub left_memory: Vec<f64>,
    pub right_memory: Vec<f64>,
    /// Seconds since each run's first sample, at the same steps, when both sides have them.
    pub left_elapsed: Vec<f64>,
    pub right_elapsed: Vec<f64>,
    /// CPU and GPU utilization in percent, when both sides recorded it.
    pub left_cpu: Vec<f64>,
    pub right_cpu: Vec<f64>,
    pub left_gpu: Vec<f64>,
    pub right_gpu: Vec<f64>,
    pub left_cdf: Vec<(f64, f64)>,
    pub right_cdf: Vec<(f64, f64)>,
    pub left_p50: Option<f64>,
    pub left_p95: Option<f64>,
    pub left_p99: Option<f64>,
    pub right_p50: Option<f64>,
    pub right_p95: Option<f64>,
    pub right_p99: Option<f64>,
    pub fired: Vec<String>,
    pub findings: Vec<Finding>,
    pub verdict: String,
    pub caption: String,
    pub left_text: String,
    pub right_text: String,
    /// Fingerprint, validation and guardrail notes, one per line. Empty for a plain file.
    pub notices: Vec<String>,
    /// B against A at p50, p90 and p99 over the steady samples, each with its interval.
    pub headline: String,
    /// B − A by percentile over the steady samples, where both runs support the percentile.
    pub difference: Vec<DifferencePoint>,
    /// Each run's levels in this window's sample indices.
    pub left_segments: Vec<Segment>,
    pub right_segments: Vec<Segment>,
    /// ΔF, ΔTc and ΔO, each with its status and why.
    pub explanations: Vec<Explanation>,
    /// The samples before the window (the warmup the alignment cut), so the warmup view can show
    /// each run from its first sample.
    pub left_lead: Vec<f64>,
    pub right_lead: Vec<f64>,
    /// Each run's settle range in sample indices from its first sample, as ΔTc read it.
    pub left_settle: Option<(usize, usize)>,
    pub right_settle: Option<(usize, usize)>,
}

/// One delta the inference page actually fired. The numbers are the same
/// ones the verdict sentence used. A bundle stores this record and shows it
/// again. It does not measure a new one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub symbol: String,
    pub id: String,
    pub name: String,
    pub kind: String,
    pub sentence: String,
    pub left: f64,
    pub right: f64,
    pub delta: f64,
    pub units: String,
}

/// Where "Show me" takes the reader: a view, and a stretch of the review window in sample indices.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    /// `series` or `distribution`.
    pub view: String,
    pub from: usize,
    pub to: usize,
}

/// One delta's tile and its "Why" panel, whether it fired, stayed quiet, or was withheld.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    pub symbol: String,
    /// `fired`, `quiet`, or `withheld`.
    pub status: String,
    pub headline: String,
    /// The rule and this pair's numbers, in plain words.
    pub why: String,
    pub parameters: Vec<[String; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrainingReview {
    pub left: TrainingEntry,
    pub right: TrainingEntry,
    pub caption: String,
    pub left_text: String,
    pub right_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Pair {
    Inference(InferenceReview),
    Training(TrainingReview),
    Geometry(GeometryReview),
}

/// Two ASPIRE training-dynamics runs side by side.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryReview {
    pub left_label: String,
    pub right_label: String,
    pub left: crate::geometry::GeometryRun,
    pub right: crate::geometry::GeometryRun,
    /// Warnings from reading either file, each naming its run.
    pub warnings: Vec<String>,
}

/// Choices that change how a review reads its samples. Recorded in the caption, so a stored
/// review says how it was made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub anomaly: stats::AnomalyRule,
}

pub fn pair(left: &Side, right: &Side) -> Result<Pair, String> {
    pair_with(left, right, Options::default())
}

pub fn pair_with(left: &Side, right: &Side, options: Options) -> Result<Pair, String> {
    match (left, right) {
        (Side::Inference(left), Side::Inference(right)) => {
            blocked(left, right)?;
            Ok(Pair::Inference(inference(left, right, options.anomaly)))
        }
        (Side::Training(left), Side::Training(right)) => Ok(Pair::Training(training(left, right))),
        (Side::Geometry(left), Side::Geometry(right)) => Ok(Pair::Geometry(geometry_review(left, right))),
        _ => Err(
            "The two sides are different kinds (an inference trace, a training history, or a geometry export). Load two of the same kind."
                .to_string(),
        ),
    }
}

fn inference(left: &InferenceRun, right: &InferenceRun, rule: stats::AnomalyRule) -> InferenceReview {
    let (skip_left, skip_right, count, summary) = align(left, right);
    let left_values = window(&left.latency_ms, skip_left, count);
    let right_values = window(&right.latency_ms, skip_right, count);
    let left_finite = readings::finite_sorted(&left_values);
    let right_finite = readings::finite_sorted(&right_values);
    let (left_throughput, right_throughput) = beside(&left.throughput, &right.throughput, left, right, skip_left, skip_right, count);
    let (left_memory, right_memory) = beside(&left.memory_mb, &right.memory_mb, left, right, skip_left, skip_right, count);
    let (left_elapsed, right_elapsed) = beside(&left.elapsed_s, &right.elapsed_s, left, right, skip_left, skip_right, count);
    let (left_cpu, right_cpu) = beside(&left.cpu_percent, &right.cpu_percent, left, right, skip_left, skip_right, count);
    let (left_gpu, right_gpu) = beside(&left.gpu_percent, &right.gpu_percent, left, right, skip_left, skip_right, count);
    let shapes = (shape::run_shape(&left.latency_ms), shape::run_shape(&right.latency_ms));
    let (findings, verdict) = inference_verdict(left, right, &shapes, rule);
    let explanations = explain(left, right, &shapes, &findings, (skip_left, skip_right, count), rule);
    let fired = findings.iter().map(|row| row.symbol.clone()).collect();
    let mut caption = format!(
        "Preset tensorflowrt-runtime-v1 (inference runtime). latency_ms (ms). {summary} The band is the p10–p90 of an 11-sample centred window, the spread of the samples, not a confidence interval. Marks are samples {marks_rule} in this window. Ratios and their 95% intervals come from {replicates} moving-block bootstrap resamples (block length the cube root of the sample count, seed fixed); they cover the variation within each run, not between runs. A percentile is printed only when enough samples bound it. p50, p95, and p99 are nearest-rank. The distribution is the empirical CDF of these same samples. ΔTd and ΔĀ stay off this page.",
        marks_rule = rule.describe(),
        replicates = stats::BOOTSTRAP_REPLICATES,
    );
    if left.steady_step.is_some() && right.steady_step.is_some() {
        caption.push_str(" The vertical line is the steady-state milestone.");
    }
    InferenceReview {
        left_text: describe(left, &left_values, &left_finite, &shapes.0, rule),
        right_text: describe(right, &right_values, &right_finite, &shapes.1, rule),
        headline: headline(left, right),
        difference: {
            let (a, b) = (steady_tails(left), steady_tails(right));
            views::difference(&slices(&a), &slices(&b))
        },
        left_segments: views::window_segments(&left.latency_ms, &shapes.0.segments, skip_left, count),
        right_segments: views::window_segments(&right.latency_ms, &shapes.1.segments, skip_right, count),
        left_label: left.label.clone(),
        right_label: right.label.clone(),
        signal: "latency_ms".to_string(),
        unit: "ms".to_string(),
        left_band: stats::quantile_band(&left_values),
        right_band: stats::quantile_band(&right_values),
        left_marks: stats::anomalies(rule, &left_values),
        right_marks: stats::anomalies(rule, &right_values),
        left_steady: readings::steady_index(&left.steps, skip_left, count, left.steady_step.filter(|_| right.steady_step.is_some())),
        right_steady: readings::steady_index(&right.steps, skip_right, count, right.steady_step.filter(|_| left.steady_step.is_some())),
        left_throughput,
        right_throughput,
        left_memory,
        right_memory,
        left_elapsed,
        right_elapsed,
        left_cpu,
        right_cpu,
        left_gpu,
        right_gpu,
        left_cdf: readings::empirical_cdf(&left_finite),
        right_cdf: readings::empirical_cdf(&right_finite),
        left_p50: readings::percentile(&left_finite, 0.50),
        left_p95: readings::percentile(&left_finite, 0.95),
        left_p99: readings::percentile(&left_finite, 0.99),
        right_p50: readings::percentile(&right_finite, 0.50),
        right_p95: readings::percentile(&right_finite, 0.95),
        right_p99: readings::percentile(&right_finite, 0.99),
        fired,
        findings,
        verdict,
        left: left_values,
        right: right_values,
        caption,
        notices: notices(left, right, &shapes),
        explanations,
        left_lead: left.latency_ms[..skip_left.min(left.latency_ms.len())].to_vec(),
        right_lead: right.latency_ms[..skip_right.min(right.latency_ms.len())].to_vec(),
        left_settle: settle_range(left, &shapes.0),
        right_settle: settle_range(right, &shapes.1),
    }
}

/// Where a run settles, as ΔTc reads it: the step stated in its file, or its detected range.
fn settle_range(run: &InferenceRun, shape: &RunShape) -> Option<(usize, usize)> {
    if stored_milestones(run) {
        let index = index_of_step(run, run.steady_step);
        return (index < run.latency_ms.len()).then_some((index, index));
    }
    shape.steady_range
}

fn param(name: &str, value: impl Into<String>) -> [String; 2] {
    [name.to_string(), value.into()]
}

/// The tiles. The numbers are recomputed from the same samples with the same seed as the verdict,
/// so a tile cannot disagree with it.
fn explain(
    left: &InferenceRun,
    right: &InferenceRun,
    shapes: &(RunShape, RunShape),
    findings: &[Finding],
    (_skip_left, skip_right, count): (usize, usize, usize),
    rule: stats::AnomalyRule,
) -> Vec<Explanation> {
    let fired = |symbol: &str| findings.iter().find(|row| row.symbol == symbol);
    let (tail_left, tail_right) = (steady_tail(left), steady_tail(right));
    let enough = tail_left.len() >= shape::MIN_SAMPLES && tail_right.len() >= shape::MIN_SAMPLES;
    let mut tiles = Vec::new();

    // ΔF
    let anomalies = |tail: &[f64]| stats::anomalies(rule, &tail.iter().map(|value| Some(*value)).collect::<Vec<_>>()).len();
    let (a, b) = (anomalies(&tail_left), anomalies(&tail_right));
    let (na, nb) = (tail_left.len(), tail_right.len());
    let p_value = stats::more_anomalies(a, na, b, nb);
    let (status, why) = match (fired("ΔF"), enough, p_value) {
        (Some(_), _, Some(p)) => (
            "fired",
            format!("B has {b} anomalies in {nb} steady samples and A has {a} in {na}. If both ran at the same anomaly rate, a split at least this lopsided would happen with probability {p:.3} (one-sided exact test), which is below {ANOMALY_LEVEL}."),
        ),
        (_, false, _) => ("quiet", format!("ΔF needs {} steady samples per side; A has {na} and B has {nb}.", shape::MIN_SAMPLES)),
        (_, true, None) => ("quiet", format!("B has {b} anomalies in {nb} steady samples and A has {a} in {na}: B does not have more for its sample count.")),
        (_, true, Some(p)) => (
            "quiet",
            format!("B has {b} anomalies in {nb} steady samples and A has {a} in {na}. With equal rates, a split at least this lopsided has probability {p:.3}, which is not below {ANOMALY_LEVEL}, so it is within chance."),
        ),
    };
    let right_marks = stats::anomalies(rule, &window(&right.latency_ms, skip_right, count));
    tiles.push(Explanation {
        symbol: "ΔF".to_string(),
        status: status.to_string(),
        headline: fired("ΔF").map_or_else(|| "No new runtime anomalies beyond chance".to_string(), |row| row.sentence.clone()),
        why: format!("{why} An anomaly is a sample {}. Warmup samples are not counted.", rule.describe()),
        parameters: vec![
            param("Rule", rule.describe()),
            param("Samples", "steady samples only"),
            param("Level", format!("one-sided p < {ANOMALY_LEVEL}")),
            param("A", format!("{a} in {na}")),
            param("B", format!("{b} in {nb}")),
        ],
        anchor: match (right_marks.first(), right_marks.last()) {
            (Some(from), Some(to)) => Some(Anchor { view: "series".to_string(), from: *from, to: *to + 1 }),
            _ => None,
        },
    });

    // ΔTc
    let tc = delta_tc(left, right, shapes);
    let range_text = |run: &InferenceRun, shape: &RunShape| match shape.steady_range {
        Some((low, high)) => format!("{}, steps {}–{}", shape.shape.label(), step_at(run, low), step_at(run, high)),
        None => shape.shape.label().to_string(),
    };
    let stated = stored_milestones(left) && stored_milestones(right);
    let (status, headline, why) = match &tc {
        Tc::Fired { text, .. } if stated => ("fired", text.clone(), "Both files state their steady-state step, and those steps are compared as written, as the 2.0 comparer did.".to_string()),
        Tc::Fired { text, .. } => (
            "fired",
            text.clone(),
            "Each run's steady start is a range: from where an 11-sample rolling median first reaches the steady level, to the latest PELT change point across penalties 3, 4 and 6 · ln n. The two ranges do not overlap.".to_string(),
        ),
        Tc::Withheld(text) => ("withheld", "No stabilization time to compare".to_string(), text.clone()),
        Tc::Quiet if stated => ("quiet", "Same steady-state step".to_string(), "Both files state the same steady-state step.".to_string()),
        Tc::Quiet => (
            "quiet",
            "Settles at about the same point".to_string(),
            "The two steady-start ranges overlap, so a difference between them could be where a segment was cut rather than a change in warmup.".to_string(),
        ),
    };
    let starts: Vec<usize> = [settle_range(left, &shapes.0), settle_range(right, &shapes.1)]
        .into_iter()
        .flatten()
        .flat_map(|(low, high)| [low, high])
        .collect();
    tiles.push(Explanation {
        symbol: "ΔTc".to_string(),
        status: status.to_string(),
        headline,
        why,
        parameters: vec![
            param("A", range_text(left, &shapes.0)),
            param("B", range_text(right, &shapes.1)),
            param("Detector", "PELT on log-latency, normal mean and variance cost"),
            param("Penalties", "3, 4 and 6 × ln n"),
        ],
        anchor: match (starts.iter().min(), starts.iter().max()) {
            (Some(from), Some(to)) => Some(Anchor { view: "warmup".to_string(), from: *from, to: *to + 1 }),
            _ => None,
        },
    });

    // ΔO
    let spread = |tail: &[f64]| stats::relative_spread(&mut tail.to_vec());
    let interval = stats::spread_ratio_interval(&tail_left, &tail_right, stats::BOOTSTRAP_SEED).filter(|_| enough);
    let spreads = format!(
        "Relative spread, (p90 − p10) / p50 of the steady samples: A {}, B {}.",
        spread(&tail_left).map_or("none".to_string(), |value| format!("{value:.3}")),
        spread(&tail_right).map_or("none".to_string(), |value| format!("{value:.3}"))
    );
    let (status, why) = match (fired("ΔO"), interval) {
        (Some(_), Some(interval)) => ("fired", format!("{spreads} B/A is {} and its 95% interval from {} block-bootstrap resamples excludes 1.", interval_text(&interval), stats::BOOTSTRAP_REPLICATES)),
        (_, Some(interval)) => ("quiet", format!("{spreads} B/A is {} and its 95% interval includes 1, so the change is within the variation.", interval_text(&interval))),
        _ if !enough => ("quiet", format!("{spreads} ΔO needs {} steady samples per side.", shape::MIN_SAMPLES)),
        _ => ("quiet", format!("{spreads} A's spread is 0, so there is no ratio to take.")),
    };
    tiles.push(Explanation {
        symbol: "ΔO".to_string(),
        status: status.to_string(),
        headline: fired("ΔO").map_or_else(|| "Runtime variability unchanged".to_string(), |row| row.sentence.clone()),
        why,
        parameters: vec![
            param("Measure", "(p90 − p10) / p50, steady samples"),
            param("Interval", format!("95%, {} moving-block resamples, seed fixed", stats::BOOTSTRAP_REPLICATES)),
            param("Fires when", "the interval excludes 1"),
        ],
        anchor: Some(Anchor { view: "distribution".to_string(), from: 0, to: count }),
    });
    tiles
}

/// The samples from the steady-state step on, or the whole run when it has none.
fn steady_tail(run: &InferenceRun) -> Vec<f64> {
    let start = index_of_step(run, run.steady_step);
    run.latency_ms[start.min(run.latency_ms.len())..].iter().copied().filter(|value| value.is_finite()).collect()
}

/// The steady samples of a run and of each of its replicates.
fn steady_tails(run: &InferenceRun) -> Vec<Vec<f64>> {
    run.runs().into_iter().map(steady_tail).collect()
}

fn slices(runs: &[Vec<f64>]) -> Vec<&[f64]> {
    runs.iter().map(Vec::as_slice).collect()
}

/// The wall-clock time of a settle range, " (2.1–3.4 s)", when the run has elapsed seconds.
fn seconds_at(run: &InferenceRun, low: usize, high: usize) -> String {
    match (run.elapsed_s.get(low), run.elapsed_s.get(high)) {
        (Some(from), Some(_)) if low == high => format!(" ({from:.2} s)"),
        (Some(from), Some(to)) => format!(" ({from:.2}–{to:.2} s)"),
        _ => String::new(),
    }
}

fn interval_text(interval: &Interval) -> String {
    format!("{:.2} ({:.2}–{:.2})", interval.estimate, interval.low, interval.high)
}

/// B/A at p50, p90 and p99 with intervals, and which percentiles have too few samples.
fn headline(left: &InferenceRun, right: &InferenceRun) -> String {
    let (tails_a, tails_b) = (steady_tails(left), steady_tails(right));
    let (a, b) = (slices(&tails_a), slices(&tails_b));
    if a.iter().chain(&b).any(|run| run.len() < shape::MIN_SAMPLES) {
        return format!("B/A: a ratio needs {} steady samples in every run.", shape::MIN_SAMPLES);
    }
    let (runs_a, runs_b) = (a.len(), b.len());
    let pooled = |runs: &[&[f64]]| runs.iter().map(|run| run.len()).sum::<usize>();
    let (count_a, count_b) = (pooled(&a), pooled(&b));
    let mut parts = Vec::new();
    let mut short = Vec::new();
    for (name, probability) in [("p50", 0.5), ("p90", 0.9), ("p99", 0.99)] {
        let needed = [count_a, count_b]
            .into_iter()
            .filter_map(|count| match stats::quantile_support(count, probability) {
                Support::TooFew { needed } => Some(needed),
                Support::Ranks { .. } => None,
            })
            .max();
        if let Some(needed) = needed {
            short.push(format!("{name} needs {needed} steady samples per side"));
            continue;
        }
        if let Some(interval) = stats::runs_interval(&a, &b, probability, stats::Contrast::Ratio, stats::BOOTSTRAP_SEED) {
            parts.push(format!("{name} {}", interval_text(&interval)));
        }
    }
    // Run-to-run variation is often larger than the variation within a run (C1, C6), so the
    // headline says which one its interval covers.
    let scope = if runs_a == 1 && runs_b == 1 {
        "within one run per side; indicative, since run-to-run variation is not measured".to_string()
    } else if runs_a.min(runs_b) < stats::MIN_RUNS {
        format!(
            "across {runs_a} and {runs_b} runs; indicative below {} runs per side",
            stats::MIN_RUNS
        )
    } else {
        format!("across {runs_a} and {runs_b} runs")
    };
    let mut text = if parts.is_empty() {
        "B/A: too few steady samples for a ratio.".to_string()
    } else {
        format!("B/A {}, {scope}.", parts.join(" · "))
    };
    if !short.is_empty() {
        text.push_str(&format!(" Not shown: {}.", short.join(", ")));
    }
    text
}

/// A broken timeline on either side stops the comparison before any delta is computed,
/// with the message the .NET app showed.
fn blocked(left: &InferenceRun, right: &InferenceRun) -> Result<(), String> {
    let (Some(left_trace), Some(right_trace)) = (&left.trace, &right.trace) else {
        let broken = [left, right]
            .into_iter()
            .find(|run| run.trace.as_ref().is_some_and(|trace| trace.validation.timeline_rejects()));
        return match broken {
            Some(run) => Err(runtrace::blocked_side_message(&run.label, run.trace.as_ref().map(|trace| &trace.validation))),
            None => Ok(()),
        };
    };
    if !left_trace.validation.timeline_rejects() && !right_trace.validation.timeline_rejects() {
        return Ok(());
    }
    Err(runtrace::blocked_message(
        (&left.label, &left_trace.run_id, &left_trace.validation),
        (&right.label, &right_trace.run_id, &right_trace.validation),
    ))
}

fn notices(left: &InferenceRun, right: &InferenceRun, shapes: &(RunShape, RunShape)) -> Vec<String> {
    let mut lines = Vec::new();
    // ΔTc counts steps. When step times differ, settling can compare differently in seconds, so
    // when both runs have elapsed time and the seconds tell a different story from the steps, the
    // page says so. Steps stated in a file are used as stated.
    let settle = |run: &InferenceRun, shape: &RunShape| -> Option<(usize, usize)> {
        if stored_milestones(run) {
            let index = index_of_step(run, run.steady_step);
            return (index < run.latency_ms.len()).then_some((index, index));
        }
        shape.steady_range
    };
    if let (Some((left_low, left_high)), Some((right_low, right_high))) = (settle(left, &shapes.0), settle(right, &shapes.1)) {
        let seconds = |run: &InferenceRun, low: usize, high: usize| Some((*run.elapsed_s.get(low)?, *run.elapsed_s.get(high)?));
        let steps_apart = step_at(left, left_high) < step_at(right, right_low) || step_at(right, right_high) < step_at(left, left_low);
        if let (Some((a_low, a_high)), Some((b_low, b_high))) = (seconds(left, left_low, left_high), seconds(right, right_low, right_high)) {
            let apart = a_high < b_low || b_high < a_low;
            if apart == steps_apart {
                return notices_rest(left, right, shapes, lines);
            }
            lines.push(format!(
                "In elapsed time, {} settles at {a_low:.2}–{a_high:.2} s and {} at {b_low:.2}–{b_high:.2} s; the ranges {}.",
                left.label,
                right.label,
                if apart { "do not overlap" } else { "overlap" }
            ));
        }
    }
    notices_rest(left, right, shapes, lines)
}

fn notices_rest(left: &InferenceRun, right: &InferenceRun, shapes: &(RunShape, RunShape), mut lines: Vec<String>) -> Vec<String> {
    for (run, shape) in [(left, &shapes.0), (right, &shapes.1)] {
        if run.trace.as_ref().is_some_and(|trace| trace.stored_milestones) {
            continue;
        }
        let Some((low, high)) = shape.steady_range else {
            continue;
        };
        let heuristic = crate::milestones::detect_warmup_end(&run.latency_ms)
            .and_then(|warmup| crate::milestones::detect_steady_start(&run.latency_ms, warmup));
        if let Some(index) = heuristic.filter(|index| *index < low || *index > high) {
            lines.push(format!(
                "{}: the 2.0 window heuristic puts steady state at step {}, outside the detected range {}–{}. Read the stabilization time with care.",
                run.label,
                step_at(run, index),
                step_at(run, low),
                step_at(run, high)
            ));
        }
    }
    if let (Some(left_trace), Some(right_trace)) = (&left.trace, &right.trace) {
        for note in runtrace::compare_fingerprints(&left_trace.fingerprints, &right_trace.fingerprints) {
            lines.push(match note.code {
                Some(code) => format!("{} ({code})", note.message),
                None => note.message,
            });
        }
    }
    for run in [left, right] {
        let Some(trace) = &run.trace else {
            continue;
        };
        for issue in trace.validation.errors.iter().chain(&trace.validation.warnings) {
            lines.push(format!("{}: {} ({})", run.label, issue.message, issue.code));
        }
        for note in &trace.guardrails {
            lines.push(format!("{}: {note}", run.label));
        }
    }
    lines
}

/// The one-sided level at which ΔF calls an excess of anomalies more than chance.
pub const ANOMALY_LEVEL: f64 = 0.05;

fn inference_verdict(left: &InferenceRun, right: &InferenceRun, shapes: &(RunShape, RunShape), rule: stats::AnomalyRule) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    let mut withheld = Vec::new();
    let (tail_left, tail_right) = (steady_tail(left), steady_tail(right));
    // A resample of a handful of samples is the same handful, so an interval from it is a point.
    let enough = tail_left.len() >= shape::MIN_SAMPLES && tail_right.len() >= shape::MIN_SAMPLES;

    // ΔF counts anomalies in the steady samples only: a warmup sample is startup cost, not a
    // runtime anomaly (C3). It fires when B's excess is beyond chance given both sample counts.
    let anomalies = |tail: &[f64]| stats::anomalies(rule, &tail.iter().map(|value| Some(*value)).collect::<Vec<_>>()).len();
    let (outliers_left, outliers_right) = (anomalies(&tail_left), anomalies(&tail_right));
    if let Some(p_value) = stats::more_anomalies(outliers_left, tail_left.len(), outliers_right, tail_right.len()).filter(|_| enough) {
        if p_value < ANOMALY_LEVEL {
            let introduced = outliers_right.saturating_sub(outliers_left);
            let sentence = if introduced == 1 {
                "Introduced 1 new runtime anomaly".to_string()
            } else {
                format!("Introduced {introduced} new runtime anomalies")
            };
            findings.push(finding(
                "ΔF",
                format!(
                    "{sentence} ({outliers_right} in {} steady samples against {outliers_left} in {}; one-sided p {p_value:.3})",
                    tail_right.len(),
                    tail_left.len()
                ),
                outliers_left as f64,
                outliers_right as f64,
                "count",
            ));
        }
    }

    match delta_tc(left, right, shapes) {
        Tc::Fired { text, left_step, right_step } => findings.push(finding(
            "ΔTc",
            text,
            left_step as f64,
            right_step as f64,
            "steps",
        )),
        Tc::Withheld(text) => withheld.push(text),
        Tc::Quiet => {}
    }

    // ΔO compares the relative spread of the steady samples, (p90 − p10) / p50, and fires only
    // when the interval on B's over A's excludes 1. Relative, so a faster run with the same
    // proportional jitter is not called steadier.
    if let Some(interval) = stats::spread_ratio_interval(&tail_left, &tail_right, stats::BOOTSTRAP_SEED).filter(|_| enough) {
        if interval.excludes(1.0) {
            let text = if interval.estimate < 1.0 { "Reduced runtime variability" } else { "Increased runtime variability" };
            let width = |tail: &[f64]| stats::relative_spread(&mut tail.to_vec()).unwrap_or(0.0);
            findings.push(finding(
                "ΔO",
                format!("{text}: relative spread (p10–p90 / p50) × {}", interval_text(&interval)),
                width(&tail_left),
                width(&tail_right),
                "ratio",
            ));
        }
    }

    let order = ["ΔF", "ΔTc", "ΔO"];
    findings.sort_by_key(|row| order.iter().position(|item| *item == row.symbol).unwrap_or(order.len()));
    let mut parts: Vec<String> = findings.iter().map(|row| format!("{} {}", row.symbol, row.sentence)).collect();
    if parts.is_empty() {
        parts.push("No delta fired on latency_ms.".to_string());
    }
    parts.extend(withheld);
    (findings, parts.join(" "))
}

fn finding(symbol: &str, sentence: String, left: f64, right: f64, units: &str) -> Finding {
    let (id, name, kind) = match symbol {
        "ΔF" => ("FailurePresence", "Failure Events", "Event"),
        "ΔTc" => ("ConvergenceTiming", "Convergence", "Timing"),
        "ΔO" => ("StabilityOscillation", "Stability", "Behavior"),
        _ => ("Unknown", "Delta", "Behavior"),
    };
    let left = stored_number(left);
    let right = stored_number(right);
    Finding {
        symbol: symbol.to_string(),
        id: id.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
        sentence,
        left,
        right,
        delta: stored_number(right - left),
        units: units.to_string(),
    }
}

/// A stored number is finite, and a zero is positive zero. Negative zero
/// compares equal and still hashes as different text.
fn stored_number(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        0.0
    } else {
        value
    }
}

enum Tc {
    Fired { text: String, left_step: i64, right_step: i64 },
    Withheld(String),
    Quiet,
}

fn steps_text(count: i64) -> String {
    if count == 1 {
        "1 step".to_string()
    } else {
        format!("{count} steps")
    }
}

fn stored_milestones(run: &InferenceRun) -> bool {
    run.trace.as_ref().is_some_and(|trace| trace.stored_milestones) && run.steady_step.is_some()
}

/// The step at a sample index.
fn step_at(run: &InferenceRun, index: usize) -> i64 {
    run.steps.get(index).copied().unwrap_or(index as i64)
}

/// ΔTc. When both files state their steady-state step, those steps are compared as the 2.0
/// comparer did. Otherwise each run's shape decides: only a flat run or one that warms up has a
/// stabilization time (W1), and ΔTc fires only when the two steady-start ranges do not overlap
/// (W3), so the difference is not an artifact of where a segment was cut.
fn delta_tc(left: &InferenceRun, right: &InferenceRun, shapes: &(RunShape, RunShape)) -> Tc {
    if stored_milestones(left) && stored_milestones(right) {
        let (left_step, right_step) = (left.steady_step.unwrap_or(0), right.steady_step.unwrap_or(0));
        let difference = right_step - left_step;
        if difference == 0 {
            return Tc::Quiet;
        }
        let text = if difference < 0 {
            format!("Stabilizes {} earlier (steps stated in the files)", steps_text(difference.abs()))
        } else {
            format!("Stabilizes {} later (steps stated in the files)", steps_text(difference))
        };
        return Tc::Fired { text, left_step, right_step };
    }
    let (left_shape, right_shape) = shapes;
    let (Some((left_low, left_high)), Some((right_low, right_high))) = (left_shape.steady_range, right_shape.steady_range) else {
        return Tc::Withheld(format!(
            "ΔTc is withheld. {} is {} and {} is {}; only a flat run or one that warms up has a stabilization time.",
            left.label,
            left_shape.shape.label(),
            right.label,
            right_shape.shape.label()
        ));
    };
    let (left_low, left_high) = (step_at(left, left_low), step_at(left, left_high));
    let (right_low, right_high) = (step_at(right, right_low), step_at(right, right_high));
    if left_low <= right_high && right_low <= left_high {
        return Tc::Quiet;
    }
    let left_step = step_at(left, left_shape.steady.unwrap_or(0));
    let right_step = step_at(right, right_shape.steady.unwrap_or(0));
    let (nearest, farthest) = if right_high < left_low {
        (left_low - right_high, left_high - right_low)
    } else {
        (right_low - left_high, right_high - left_low)
    };
    let direction = if right_high < left_low { "earlier" } else { "later" };
    let text = format!(
        "Stabilizes {}–{} {direction} ({} settles at step {left_low}–{left_high}, {} at {right_low}–{right_high})",
        nearest,
        steps_text(farthest),
        left.label,
        right.label
    );
    Tc::Fired { text, left_step, right_step }
}

/// The first sample at or after `step`, or the start when there is no milestone.
fn index_of_step(run: &InferenceRun, step: Option<i64>) -> usize {
    step.map_or(0, |step| run.steps.iter().position(|value| *value >= step).unwrap_or(run.steps.len()))
}


fn geometry_review(left: &crate::open::GeometrySide, right: &crate::open::GeometrySide) -> GeometryReview {
    let (left_label, right_label) = (left.run.name(), right.run.name());
    let warnings = left
        .warnings
        .iter()
        .map(|warning| format!("{left_label}: {warning}"))
        .chain(right.warnings.iter().map(|warning| format!("{right_label}: {warning}")))
        .collect();
    GeometryReview {
        left_label,
        right_label,
        left: left.run.clone(),
        right: right.run.clone(),
        warnings,
    }
}

fn training(left: &TrainingEntry, right: &TrainingEntry) -> TrainingReview {
    let caption = "Training loss, stored samples. Backpropagate keeps at most 100 points by uniform index sampling before it writes the file. final_loss is a separate number and is not appended to this curve. This is not an inference review, so ΔTc, ΔO, ΔF, ΔĀ, and ΔTd are not computed.".to_string();
    TrainingReview {
        left_text: training_text(left),
        right_text: training_text(right),
        left: left.clone(),
        right: right.clone(),
        caption,
    }
}

fn align(left: &InferenceRun, right: &InferenceRun) -> (usize, usize, usize, String) {
    let left_steps = steps_of(left);
    let right_steps = steps_of(right);
    let both_steady = left.steady_step.is_some() && right.steady_step.is_some();
    let mut left_anchor = left.steady_step;
    let mut right_anchor = right.steady_step;
    if left_anchor.is_none() || right_anchor.is_none() {
        if left_anchor.is_none() {
            left_anchor = left.warmup_end;
        }
        if right_anchor.is_none() {
            right_anchor = right.warmup_end;
        }
    }
    if let (Some(left_at), Some(right_at)) = (left_anchor, right_anchor) {
        let after_left = left_steps.iter().filter(|step| **step >= left_at).count();
        let after_right = right_steps.iter().filter(|step| **step >= right_at).count();
        let count = after_left.min(after_right);
        let kind = if both_steady { "steady-state milestone" } else { "warmup milestone" };
        let summary = format!("Aligned {count} steps from the {kind}.");
        return (left_steps.len() - after_left, right_steps.len() - after_right, count, summary);
    }
    let count = left_steps.len().min(right_steps.len());
    (0, 0, count, format!("Aligned {count} steps from the first step."))
}

fn steps_of(run: &InferenceRun) -> Vec<i64> {
    if run.steps.len() == run.latency_ms.len() {
        run.steps.clone()
    } else {
        (0..run.latency_ms.len() as i64).collect()
    }
}

fn window(values: &[f64], skip: usize, count: usize) -> Vec<Option<f64>> {
    values.iter().skip(skip).take(count).copied().map(Some).collect()
}

/// A series recorded beside latency (throughput, memory), cut to the same aligned window.
/// Both sides need it at every latency step, or neither is shown.
fn beside(
    left_values: &[f64],
    right_values: &[f64],
    left: &InferenceRun,
    right: &InferenceRun,
    skip_left: usize,
    skip_right: usize,
    count: usize,
) -> (Vec<f64>, Vec<f64>) {
    if left_values.len() != left.latency_ms.len() || right_values.len() != right.latency_ms.len() {
        return (Vec::new(), Vec::new());
    }
    if left_values.is_empty() || right_values.is_empty() {
        return (Vec::new(), Vec::new());
    }
    (
        left_values.iter().skip(skip_left).take(count).copied().collect(),
        right_values.iter().skip(skip_right).take(count).copied().collect(),
    )
}

fn describe(run: &InferenceRun, values: &[Option<f64>], sorted: &[f64], shape: &RunShape, rule: stats::AnomalyRule) -> String {
    let marks = stats::anomalies(rule, values).len();
    let percentile = |probability: f64| match stats::quantile_support(sorted.len(), probability) {
        Support::Ranks { .. } => format!("{} ms", fmt(readings::percentile(sorted, probability))),
        Support::TooFew { needed } => format!("needs {needed} samples"),
    };
    let settles = if run.trace.as_ref().is_some_and(|trace| trace.stored_milestones) {
        match run.steady_step {
            Some(step) => format!("steady from step {step} (stated in the file)"),
            None => "no steady step stated in the file".to_string(),
        }
    } else {
        match shape.steady_range {
            Some((low, high)) if low == high => format!("{}, settles at step {}{}", shape.shape.label(), step_at(run, low), seconds_at(run, low, low)),
            Some((low, high)) => format!(
                "{}, settles between steps {} and {}{}",
                shape.shape.label(),
                step_at(run, low),
                step_at(run, high),
                seconds_at(run, low, high)
            ),
            None => shape.shape.label().to_string(),
        }
    };
    let label = if run.replicates.is_empty() {
        run.label.clone()
    } else {
        format!("{} (first of {} runs; the deltas and this plot use it)", run.label, run.replicates.len() + 1)
    };
    format!(
        "{} · latency_ms · {} samples · p50 {} · p95 {} · p99 {} · {marks} anomaly marks · {settles}",
        label,
        values.len(),
        percentile(0.50),
        percentile(0.95),
        percentile(0.99),
    )
}

fn training_text(entry: &TrainingEntry) -> String {
    let model = if entry.model_name.is_empty() {
        String::new()
    } else {
        format!(" · {}", entry.model_name)
    };
    let metrics = if entry.task_metrics.is_empty() {
        "no task metrics".to_string()
    } else {
        entry
            .task_metrics
            .iter()
            .map(|(name, score)| {
                let ci = entry
                    .metric_ci
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, half)| format!(" ± {}", fmt(Some(*half))))
                    .unwrap_or_default();
                format!("{name} {}{ci}", fmt(Some(*score)))
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    format!(
        "{}{model} · {} · {} stored samples · final loss {} · held-out loss {} · perplexity {} · eval n {} · {metrics}",
        entry.run_id,
        entry.selection,
        entry.loss.len(),
        fmt(entry.final_loss),
        fmt(entry.held_out_loss),
        fmt(entry.perplexity),
        entry.eval_n.map(|count| count.to_string()).unwrap_or_else(|| "none".to_string()),
    )
}

fn fmt(value: Option<f64>) -> String {
    match value {
        Some(number) if number.is_finite() => {
            let text = format!("{number:.3}");
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        }
        _ => "none".to_string(),
    }
}
