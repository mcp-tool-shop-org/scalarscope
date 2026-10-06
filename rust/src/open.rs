//! What a file is allowed to become.
//!
//! An inference file contributes a latency series. A stored RunTrace also brings its
//! milestones, fingerprints and validation. A backpropagate `run_history.json`
//! contributes the stored training-loss samples of one entry. Geometry stays out of
//! this shell.
//!
//! A milestone (`warmup_end`, `steady_step`) is a step number from `steps`, never a
//! sample index.

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::milestones::{detect_steady_start, detect_warmup_end};
use crate::runtrace::{self, TraceInfo};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InferenceRun {
    pub label: String,
    pub steps: Vec<i64>,
    pub latency_ms: Vec<f64>,
    pub throughput: Vec<f64>,
    pub warmup_end: Option<i64>,
    pub steady_step: Option<i64>,
    /// Memory in MiB at the same steps, or empty.
    pub memory_mb: Vec<f64>,
    /// Present when the run came from a stored RunTrace.
    pub trace: Option<TraceInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrainingEntry {
    pub run_id: String,
    pub model_name: String,
    pub loss: Vec<f64>,
    pub final_loss: Option<f64>,
    pub train_steps: Option<u64>,
    pub held_out_loss: Option<f64>,
    pub perplexity: Option<f64>,
    pub eval_n: Option<u64>,
    pub task_metrics: Vec<(String, f64)>,
    pub metric_ci: Vec<(String, f64)>,
    pub entry_count: usize,
    pub selection: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Inference(InferenceRun),
    Training(TrainingEntry),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub path: String,
    pub side: Side,
}

pub fn open_path(path: &Path) -> Result<Loaded, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("Could not read the file. {error}"))?;
    let label = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("run")
        .to_string();
    let side = open_text(&text, &label)?;
    Ok(Loaded {
        path: path.display().to_string(),
        side,
    })
}

pub fn open_text(text: &str, label: &str) -> Result<Side, String> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let value: Value = serde_json::from_str(trimmed).map_err(|error| format!("The JSON did not parse. {error}"))?;
        return open_json(value, label);
    }
    open_csv(trimmed, label)
}

fn open_json(value: Value, label: &str) -> Result<Side, String> {
    match &value {
        Value::Object(map) if map.contains_key("traceEvents") => open_trace(map, label),
        Value::Object(map) if runtrace::looks_like_runtrace(map) => open_runtrace(map, label),
        Value::Object(map) if map.contains_key("trajectory") => Err(
            "This is a geometry run. This Rust review opens an inference trace or a backpropagate training history."
                .to_string(),
        ),
        Value::Object(map) if bench_array(map).is_some() => open_benchmark(&value, label),
        _ if is_training(&value) => open_training(&value, label),
        Value::Object(_) => Err(
            "This JSON is not a profiler trace, a benchmark, or a backpropagate run history.".to_string(),
        ),
        _ => Err("This JSON is not a profiler trace, a benchmark, or a backpropagate run history.".to_string()),
    }
}

fn open_trace(map: &serde_json::Map<String, Value>, label: &str) -> Result<Side, String> {
    let events = map
        .get("traceEvents")
        .and_then(Value::as_array)
        .ok_or_else(|| "The profiler file has no traceEvents array.".to_string())?;
    // A complete ProfilerStep is one inference. Nested ops, including a
    // TensorRT event inside that step, are not more samples. A trace with
    // no such step still uses the TensorRT or inference name filter.
    let steps = step_durations(events);
    let latency = if steps.is_empty() {
        named_inference_durations(events)
    } else {
        steps
    };
    if latency.is_empty() {
        return Err(
            "This profiler trace has no numeric latency. A complete ProfilerStep is one inference. Without one, event names have to contain TensorRT or inference."
                .to_string(),
        );
    }
    Ok(Side::Inference(series(label, latency, Vec::new())))
}

fn step_durations(events: &[Value]) -> Vec<f64> {
    events
        .iter()
        .filter(|event| is_complete(event) && event_name(event).is_some_and(is_profiler_step))
        .filter_map(duration_ms)
        .collect()
}

fn named_inference_durations(events: &[Value]) -> Vec<f64> {
    events
        .iter()
        .filter(|event| {
            event_name(event).is_some_and(|name| {
                let lowered = name.to_ascii_lowercase();
                lowered.contains("tensorrt") || lowered.contains("inference")
            })
        })
        .filter_map(duration_ms)
        .collect()
}

fn is_complete(event: &Value) -> bool {
    match event.get("ph") {
        None => true,
        Some(Value::String(phase)) => phase == "X",
        Some(_) => false,
    }
}

fn is_profiler_step(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered == "profilerstep" || lowered.starts_with("profilerstep#")
}

fn event_name(event: &Value) -> Option<&str> {
    event.get("name").and_then(Value::as_str)
}

fn duration_ms(event: &Value) -> Option<f64> {
    event
        .get("dur")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .map(|microseconds| microseconds / 1000.0)
}

fn open_benchmark(value: &Value, label: &str) -> Result<Side, String> {
    let results = value
        .as_object()
        .and_then(bench_array)
        .ok_or_else(|| "The benchmark file has no results.".to_string())?;
    let mut latency = Vec::new();
    let mut throughput = Vec::new();
    let mut saw_throughput = false;
    for item in results {
        let Some(sample) = item
            .get("latency_ms")
            .or_else(|| item.get("latency"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
        else {
            continue;
        };
        latency.push(sample);
        if let Some(rate) = item
            .get("throughput")
            .or_else(|| item.get("items_per_sec"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
        {
            throughput.push(rate);
            saw_throughput = true;
        }
    }
    if latency.is_empty() {
        return Err("This benchmark has no numeric latency.".to_string());
    }
    if !saw_throughput || throughput.len() != latency.len() {
        throughput.clear();
    }
    Ok(Side::Inference(series(label, latency, throughput)))
}

fn open_csv(text: &str, label: &str) -> Result<Side, String> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| "The CSV has no header.".to_string())?
        .split(',')
        .map(|cell| cell.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let latency_at = column(&header, &["latency_ms", "latency", "time_ms"])
        .ok_or_else(|| "The CSV has no latency column.".to_string())?;
    let step_at = column(&header, &["step", "iteration", "batch"]);
    let throughput_at = column(&header, &["throughput", "items_per_sec", "samples_per_sec"]);
    let mut steps = Vec::new();
    let mut latency = Vec::new();
    let mut throughput = Vec::new();
    let mut next_step = 0_i64;
    for line in lines {
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        let Some(cell) = cells.get(latency_at) else {
            continue;
        };
        let Ok(sample) = cell.parse::<f64>() else {
            return Err("This CSV has no numeric latency.".to_string());
        };
        if !sample.is_finite() {
            return Err("This CSV has no numeric latency.".to_string());
        }
        let step = step_at
            .and_then(|index| cells.get(index))
            .and_then(|cell| cell.parse::<i64>().ok())
            .unwrap_or(next_step);
        next_step = step + 1;
        steps.push(step);
        latency.push(sample);
        if let Some(index) = throughput_at {
            if let Some(rate) = cells.get(index).and_then(|cell| cell.parse::<f64>().ok()).filter(|rate| rate.is_finite()) {
                throughput.push(rate);
            }
        }
    }
    if latency.is_empty() {
        return Err("This CSV has no numeric latency.".to_string());
    }
    if throughput.len() != latency.len() {
        throughput.clear();
    }
    Ok(Side::Inference(finish(InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput,
        warmup_end: None,
        steady_step: None,
        memory_mb: Vec::new(),
        trace: None,
    })))
}

fn open_training(value: &Value, label: &str) -> Result<Side, String> {
    let entries: Vec<&Value> = match value {
        Value::Array(items) => items.iter().filter(|item| item.as_object().is_some_and(is_entry)).collect(),
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    };
    if entries.is_empty() {
        return Err("This run history has no training entry.".to_string());
    }
    let count = entries.len();
    let chosen = entries
        .iter()
        .rposition(|entry| entry.get("status").and_then(Value::as_str) == Some("completed"))
        .map(|index| (index, "the last completed row in this file"))
        .or_else(|| {
            entries.iter().rposition(has_loss).map(|index| (index, "the last row in this file with a loss"))
        })
        .ok_or_else(|| "This run history has no loss to draw.".to_string())?;
    let entry = entries[chosen.0];
    let loss = entry
        .get("loss_history")
        .and_then(Value::as_array)
        .map(|samples| {
            samples
                .iter()
                .filter_map(Value::as_f64)
                .filter(|sample| sample.is_finite())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let eval = entry.get("eval");
    let task_metrics = metric_pairs(eval, "task_metrics");
    let metric_ci = metric_pairs(eval, "metric_ci");
    Ok(Side::Training(TrainingEntry {
        run_id: entry
            .get("run_id")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .unwrap_or(label)
            .to_string(),
        model_name: entry.get("model_name").and_then(Value::as_str).unwrap_or("").to_string(),
        loss,
        final_loss: entry.get("final_loss").and_then(Value::as_f64).filter(|value| value.is_finite()),
        train_steps: entry.get("steps").and_then(Value::as_u64),
        held_out_loss: eval
            .and_then(|item| item.get("held_out_loss"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite()),
        perplexity: eval
            .and_then(|item| item.get("perplexity"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite()),
        eval_n: eval.and_then(|item| item.get("eval_n")).and_then(Value::as_u64),
        task_metrics,
        metric_ci,
        entry_count: count,
        selection: chosen.1.to_string(),
    }))
}

fn series(label: &str, latency: Vec<f64>, throughput: Vec<f64>) -> InferenceRun {
    let steps = (0..latency.len() as i64).collect();
    finish(InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput,
        warmup_end: None,
        steady_step: None,
        memory_mb: Vec::new(),
        trace: None,
    })
}

fn finish(mut run: InferenceRun) -> InferenceRun {
    let warmup = detect_warmup_end(&run.latency_ms);
    let step_at = |index: usize| run.steps.get(index).copied().unwrap_or(index as i64);
    run.warmup_end = warmup.map(step_at);
    run.steady_step = warmup
        .and_then(|index| detect_steady_start(&run.latency_ms, index))
        .map(step_at);
    run
}

/// A stored RunTrace. Its own milestones are used as written; only a trace without
/// them falls back to detection from the latency values.
fn open_runtrace(map: &serde_json::Map<String, Value>, label: &str) -> Result<Side, String> {
    let trace = runtrace::parse(map)?;
    let validation = runtrace::validate(&trace);
    let latency = runtrace::series(&trace, &runtrace::LATENCY_NAMES)
        .ok_or_else(|| "This RunTrace has no latency series.".to_string())?;
    let throughput = runtrace::series(&trace, &runtrace::THROUGHPUT_NAMES);
    let memory = runtrace::series(&trace, &runtrace::MEMORY_NAMES);
    let memory_scale = memory.map_or(1.0, |series| if series.name.ends_with("_bytes") { 1.0 / 1_048_576.0 } else { 1.0 });
    let mut steps = Vec::new();
    let mut latency_ms = Vec::new();
    let mut rates = Vec::new();
    let mut memory_mb = Vec::new();
    for (index, sample) in latency.values.iter().enumerate() {
        let Some(sample) = sample.filter(|value| value.is_finite()) else {
            continue;
        };
        steps.push(trace.steps.get(index).copied().unwrap_or(index as i64));
        latency_ms.push(sample);
        let at = |series: Option<&runtrace::Scalar>| {
            series.and_then(|series| series.values.get(index).copied().flatten()).filter(|value| value.is_finite())
        };
        if let Some(rate) = at(throughput) {
            rates.push(rate);
        }
        if let Some(memory) = at(memory) {
            memory_mb.push(memory * memory_scale);
        }
    }
    if latency_ms.is_empty() {
        return Err("This RunTrace has no numeric latency.".to_string());
    }
    if rates.len() != latency_ms.len() {
        rates.clear();
    }
    if memory_mb.len() != latency_ms.len() {
        memory_mb.clear();
    }
    let stored_warmup = runtrace::milestone(&trace, "warmup_end");
    let stored_steady = runtrace::milestone(&trace, "steady_state_start");
    let stored = stored_warmup.is_some() || stored_steady.is_some();
    let run = InferenceRun {
        label: if trace.label.trim().is_empty() { label.to_string() } else { trace.label.clone() },
        steps,
        latency_ms,
        throughput: rates,
        memory_mb,
        trace: Some(TraceInfo {
            run_id: trace.run_id.clone(),
            fingerprints: trace.fingerprints.clone(),
            guardrails: runtrace::guardrails(&trace),
            validation,
            stored_milestones: stored,
        }),
        ..InferenceRun::default()
    };
    Ok(Side::Inference(if stored {
        InferenceRun {
            warmup_end: stored_warmup,
            steady_step: stored_steady,
            ..run
        }
    } else {
        finish(run)
    }))
}

fn bench_array(map: &serde_json::Map<String, Value>) -> Option<&Vec<Value>> {
    ["results", "iterations", "benchmarks"]
        .iter()
        .find_map(|name| map.get(*name).and_then(Value::as_array))
}

fn is_training(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().any(|item| item.as_object().is_some_and(is_entry)),
        Value::Object(map) => is_entry(map),
        _ => false,
    }
}

fn is_entry(map: &serde_json::Map<String, Value>) -> bool {
    map.get("loss_history").and_then(Value::as_array).is_some()
        || (map.contains_key("run_id") && (map.contains_key("final_loss") || map.contains_key("loss_history")))
}

fn has_loss(entry: &&Value) -> bool {
    entry.get("loss_history").and_then(Value::as_array).is_some_and(|samples| !samples.is_empty())
        || entry.get("final_loss").and_then(Value::as_f64).is_some()
}

fn column(header: &[String], names: &[&str]) -> Option<usize> {
    header.iter().position(|cell| names.iter().any(|name| cell == name))
}

fn metric_pairs(eval: Option<&Value>, name: &str) -> Vec<(String, f64)> {
    let Some(object) = eval.and_then(|item| item.get(name)).and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut pairs: Vec<(String, f64)> = object
        .iter()
        .filter_map(|(key, value)| value.as_f64().filter(|number| number.is_finite()).map(|number| (key.clone(), number)))
        .collect();
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    pairs
}
