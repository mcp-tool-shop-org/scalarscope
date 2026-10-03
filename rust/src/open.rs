//! What a file is allowed to become.
//!
//! An inference file contributes a latency series. A backpropagate
//! `run_history.json` contributes the stored training-loss samples of one
//! entry. Geometry stays out of this shell.

use std::fs;
use std::path::Path;

use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceRun {
    pub label: String,
    pub steps: Vec<i64>,
    pub latency_ms: Vec<f64>,
    pub throughput: Vec<f64>,
    pub steady_step: Option<i64>,
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
    let mut latency = Vec::new();
    for event in events {
        let Some(name) = event.get("name").and_then(Value::as_str) else {
            continue;
        };
        let lowered = name.to_ascii_lowercase();
        if !lowered.contains("tensorrt") && !lowered.contains("inference") {
            continue;
        }
        if let Some(duration) = event.get("dur").and_then(Value::as_f64).filter(|value| value.is_finite()) {
            latency.push(duration / 1000.0);
        }
    }
    if latency.is_empty() {
        return Err(
            "This profiler trace has no numeric latency. Event names have to contain TensorRT or inference."
                .to_string(),
        );
    }
    Ok(Side::Inference(series(label, latency, Vec::new())))
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
    Ok(Side::Inference(InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput,
        steady_step: None,
    }))
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
    InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput,
        steady_step: None,
    }
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
