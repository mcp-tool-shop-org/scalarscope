//! A pair is either two inference runs or two training entries.
//! The inference drawing reads the aligned latency window. The training
//! drawing reads the stored loss samples. Canonical inference deltas are
//! not computed on either page in this shell.

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
    pub caption: String,
    pub left_text: String,
    pub right_text: String,
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
    let caption = format!(
        "latency_ms (ms). {summary} The band is a centered 5-sample rolling mean ± population standard deviation, not a confidence interval. Marks are 3-sigma on this window. p50, p95, and p99 are nearest-rank. The distribution is the empirical CDF of these same samples. Canonical deltas are not computed in this shell."
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
        left: left_values,
        right: right_values,
        caption,
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
    if let (Some(left_anchor), Some(right_anchor)) = (left.steady_step, right.steady_step) {
        let after_left = left_steps.iter().filter(|step| **step >= left_anchor).count();
        let after_right = right_steps.iter().filter(|step| **step >= right_anchor).count();
        let count = after_left.min(after_right);
        let summary = format!("Aligned {count} steps from the steady-state milestone.");
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
