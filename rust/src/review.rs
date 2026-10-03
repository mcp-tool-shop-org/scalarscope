//! A pair is either two inference runs or two training entries.
//! The inference page reports ΔF, ΔO, and ΔTc. ΔTc is withheld when a
//! steady-state milestone is missing. ΔTd and ΔĀ stay off that page.
//! A training page does not compute those deltas.

use serde::{Deserialize, Serialize};

use crate::open::{InferenceRun, Side, TrainingEntry};
use crate::readings::{self, Band};

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
}

pub fn pair(left: &Side, right: &Side) -> Result<Pair, String> {
    match (left, right) {
        (Side::Inference(left), Side::Inference(right)) => Ok(Pair::Inference(inference(left, right))),
        (Side::Training(left), Side::Training(right)) => Ok(Pair::Training(training(left, right))),
        _ => Err(
            "One side is a training history and the other is an inference trace. Load two of the same kind."
                .to_string(),
        ),
    }
}

fn inference(left: &InferenceRun, right: &InferenceRun) -> InferenceReview {
    let (skip_left, skip_right, count, summary) = align(left, right);
    let left_values = window(&left.latency_ms, skip_left, count);
    let right_values = window(&right.latency_ms, skip_right, count);
    let left_finite = readings::finite_sorted(&left_values);
    let right_finite = readings::finite_sorted(&right_values);
    let (left_throughput, right_throughput) = throughput(left, right, skip_left, skip_right, count);
    let (findings, verdict) = inference_verdict(left, right);
    let fired = findings.iter().map(|row| row.symbol.clone()).collect();
    let caption = format!(
        "latency_ms (ms). {summary} The band is a centered 5-sample rolling mean ± population standard deviation, not a confidence interval. Marks are 3-sigma on this window. p50, p95, and p99 are nearest-rank. The distribution is the empirical CDF of these same samples. ΔTd and ΔĀ stay off this page."
    );
    InferenceReview {
        left_text: describe(&left.label, &left_values, &left_finite),
        right_text: describe(&right.label, &right_values, &right_finite),
        left_label: left.label.clone(),
        right_label: right.label.clone(),
        signal: "latency_ms".to_string(),
        unit: "ms".to_string(),
        left_band: readings::deviation_band(&left_values),
        right_band: readings::deviation_band(&right_values),
        left_marks: readings::three_sigma_indices(&left_values),
        right_marks: readings::three_sigma_indices(&right_values),
        left_steady: readings::steady_index(&left.steps, skip_left, count, left.steady_step.filter(|_| right.steady_step.is_some())),
        right_steady: readings::steady_index(&right.steps, skip_right, count, right.steady_step.filter(|_| left.steady_step.is_some())),
        left_throughput,
        right_throughput,
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
    }
}

fn inference_verdict(left: &InferenceRun, right: &InferenceRun) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    let mut withheld = Vec::new();

    let outliers_left = count_outliers(&left.latency_ms);
    let outliers_right = count_outliers(&right.latency_ms);
    if outliers_right > outliers_left {
        let introduced = outliers_right - outliers_left;
        findings.push(finding(
            "ΔF",
            format!("Introduced {introduced} new runtime anomalies"),
            outliers_left as f64,
            outliers_right as f64,
            "count",
        ));
    }

    match delta_tc(left, right) {
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

    let spread_left = population_std_from(&left.latency_ms, left.steady_step.unwrap_or(0).max(0) as usize);
    let spread_right = population_std_from(&right.latency_ms, right.steady_step.unwrap_or(0).max(0) as usize);
    let scale = spread_left.max(spread_right);
    if (spread_right - spread_left).abs() > 0.01 * scale {
        let text = if spread_right < spread_left {
            "Reduced runtime variability".to_string()
        } else {
            "Increased runtime variability".to_string()
        };
        findings.push(finding("ΔO", text, spread_left, spread_right, "ms"));
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

fn delta_tc(left: &InferenceRun, right: &InferenceRun) -> Tc {
    let both = left.steady_step.is_some() && right.steady_step.is_some();
    let left_step = left.steady_step.unwrap_or_else(|| last_step(left));
    let right_step = right.steady_step.unwrap_or_else(|| last_step(right));
    let difference = right_step - left_step;
    if difference == 0 {
        return Tc::Quiet;
    }
    if !both {
        return Tc::Withheld(
            "ΔTc is withheld. A steady-state milestone is missing, so the last step is not a stabilization time.".to_string(),
        );
    }
    let text = if difference < 0 {
        format!("Stabilizes {} steps earlier", difference.abs())
    } else {
        format!("Stabilizes {difference} steps later")
    };
    Tc::Fired {
        text,
        left_step,
        right_step,
    }
}

fn last_step(run: &InferenceRun) -> i64 {
    if run.steps.len() == run.latency_ms.len() {
        run.steps.last().copied().unwrap_or(0)
    } else {
        run.latency_ms.len().saturating_sub(1) as i64
    }
}

fn count_outliers(values: &[f64]) -> usize {
    let finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if finite.len() < 3 {
        return 0;
    }
    let mean = finite.iter().sum::<f64>() / finite.len() as f64;
    let standard_deviation = (finite.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / finite.len() as f64).sqrt();
    let threshold = 3.0 * standard_deviation;
    finite.iter().filter(|value| (*value - mean).abs() > threshold).count()
}

fn population_std_from(values: &[f64], start: usize) -> f64 {
    let tail: Vec<f64> = values.iter().skip(start).copied().filter(|value| value.is_finite()).collect();
    if tail.len() < 2 {
        return 0.0;
    }
    let mean = tail.iter().sum::<f64>() / tail.len() as f64;
    (tail.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / tail.len() as f64).sqrt()
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

fn throughput(
    left: &InferenceRun,
    right: &InferenceRun,
    skip_left: usize,
    skip_right: usize,
    count: usize,
) -> (Vec<f64>, Vec<f64>) {
    if left.throughput.len() != left.latency_ms.len() || right.throughput.len() != right.latency_ms.len() {
        return (Vec::new(), Vec::new());
    }
    if left.throughput.is_empty() || right.throughput.is_empty() {
        return (Vec::new(), Vec::new());
    }
    (
        left.throughput.iter().skip(skip_left).take(count).copied().collect(),
        right.throughput.iter().skip(skip_right).take(count).copied().collect(),
    )
}

fn describe(label: &str, values: &[Option<f64>], sorted: &[f64]) -> String {
    let marks = readings::three_sigma_indices(values).len();
    format!(
        "{label} · latency_ms · {} samples · p50 {} ms · p95 {} ms · p99 {} ms · {marks} anomaly marks",
        values.len(),
        fmt(readings::percentile(sorted, 0.50)),
        fmt(readings::percentile(sorted, 0.95)),
        fmt(readings::percentile(sorted, 0.99)),
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
