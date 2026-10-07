//! ScalarScope's side of the shared workbench (the `workbench` crate, which RunForge also uses).
//!
//! This module hands the workbench the open inference runs: their knobs, an identity from their
//! own samples, and the latency measures a formula can read. The workbench parses formulas,
//! tests hypotheses about knobs, gathers e-value evidence and runs the session with a local
//! model. ScalarScope computes every measure here, and the model never computes one.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{json, Value};
use workbench::{Board, Host, Measure, Run};

use crate::open::InferenceRun;
use crate::stats::{self, AnomalyRule, Support};

/// The workbench's memory: learned tools, hypotheses, the checkpoint book. It holds no paths.
pub const MEMORY_FILE: &str = "workbench.json";

/// The latency measures. The workbench adds `knob` and arithmetic.
///
/// No `means` carries a digit: the model reads them in the tool schema and echoes them, and
/// the wording fence refuses a digit it writes (live runs 1 and 2, docs/receipts).
pub const MEASURES: &[Measure] = &[
    Measure { name: "p50", args: "", means: "the median latency of the steady samples, in ms" },
    Measure { name: "p90", args: "", means: "the upper tail of the steady samples, in ms" },
    Measure { name: "p99", args: "", means: "the far tail of the steady samples, in ms" },
    Measure { name: "p99_over_p50", args: "", means: "the far tail divided by the median: how heavy the tail is" },
    Measure { name: "steady_mean", args: "", means: "the mean latency of the steady samples, in ms" },
    Measure {
        name: "warmup_cost",
        args: "",
        means: "the latency above the steady median spent before the steady state, in ms × steps",
    },
    Measure {
        name: "anomaly_rate",
        args: "",
        means: "the share of steady samples the anomaly rule marks, from none to all",
    },
    Measure { name: "throughput_p50", args: "", means: "the median throughput of the steady samples" },
    Measure { name: "memory_peak", args: "", means: "the highest memory sample, in MiB" },
    Measure { name: "samples", args: "", means: "how many samples the run has" },
    Measure {
        name: "quantile_between",
        args: "a, b, q",
        means: "the latency at quantile q (a fraction, such as a half) at steps a to b",
    },
    Measure { name: "mean_between", args: "a, b", means: "the mean latency at steps a to b" },
];

/// Open inference runs, as the workbench measures them.
pub struct LatencyHost {
    runs: Vec<InferenceRun>,
    rule: AnomalyRule,
}

impl Host for LatencyHost {
    fn program(&self) -> &str {
        "ScalarScope"
    }

    fn subject(&self) -> &str {
        "inference runs (latency per step)"
    }

    fn build_example(&self) -> &str {
        "For example, how much of the tail sits above the 90th percentile (p99 / p90), or how slow the first ten steps are against the steady median (mean_between(0, 9) / p50)."
    }

    fn measures(&self) -> &[Measure] {
        MEASURES
    }

    fn measure(&self, run: usize, name: &str, args: &[f64]) -> Result<f64, String> {
        let run = &self.runs[run];
        match (name, args) {
            ("p50", []) => steady_quantile(run, 0.5),
            ("p90", []) => steady_quantile(run, 0.9),
            ("p99", []) => steady_quantile(run, 0.99),
            ("p99_over_p50", []) => Ok(steady_quantile(run, 0.99)? / steady_quantile(run, 0.5)?),
            ("steady_mean", []) => mean(&steady(run)).ok_or_else(|| missing(run, "steady samples")),
            ("warmup_cost", []) => warmup_cost(run),
            ("anomaly_rate", []) => {
                let values: Vec<Option<f64>> = steady(run).into_iter().map(Some).collect();
                if values.is_empty() {
                    return Err(missing(run, "steady samples"));
                }
                Ok(stats::anomalies(self.rule, &values).len() as f64 / values.len() as f64)
            }
            ("throughput_p50", []) => {
                if run.throughput.len() != run.latency_ms.len() || run.throughput.is_empty() {
                    return Err(missing(run, "throughput series"));
                }
                let mut values = run.throughput[steady_start(run)..].to_vec();
                values.retain(|value| value.is_finite());
                values.sort_by(f64::total_cmp);
                workbench::quantile(&values, 0.5).ok_or_else(|| missing(run, "steady throughput"))
            }
            ("memory_peak", []) => run
                .memory_mb
                .iter()
                .copied()
                .filter(|value| value.is_finite())
                .reduce(f64::max)
                .ok_or_else(|| missing(run, "memory series")),
            ("samples", []) => Ok(run.latency_ms.len() as f64),
            ("quantile_between", [a, b, q]) => {
                if !(0.0..=1.0).contains(q) {
                    return Err("quantile_between takes q from 0 to 1, such as 0.99.".to_string());
                }
                let mut values = between(run, *a, *b);
                values.sort_by(f64::total_cmp);
                workbench::quantile(&values, *q).ok_or_else(|| empty_span(run, *a, *b))
            }
            ("mean_between", [a, b]) => mean(&between(run, *a, *b)).ok_or_else(|| empty_span(run, *a, *b)),
            (other, _) => Err(format!("{other} is not a measure this program knows.")),
        }
    }

    fn reason_hint(&self) -> &str {
        "The mechanism you suspect, in words; name a measure by its name, such as p99, and write no other numbers."
    }

    fn note_hint(&self) -> &str {
        "What you looked at and what is still open, in words; name a measure by its name, such as p99, and write no other numbers."
    }

    fn knob_label(&self, key: &str) -> String {
        match key {
            "batch" | "batch_size" => "batch size".to_string(),
            "precision" | "dtype" => "precision".to_string(),
            "tensorrt" | "trt" => "TensorRT".to_string(),
            "threads" | "num_threads" => "threads".to_string(),
            "cuda_graphs" => "CUDA graphs".to_string(),
            "input_shape" | "shape" => "input shape".to_string(),
            other => other.replace('_', " "),
        }
    }
}

fn missing(run: &InferenceRun, what: &str) -> String {
    format!("{} has no {what}.", run.label)
}

fn empty_span(run: &InferenceRun, a: f64, b: f64) -> String {
    format!("{} has no latency between steps {a} and {b}.", run.label)
}

/// The index of the first steady sample: the stored or detected steady step, else 0.
fn steady_start(run: &InferenceRun) -> usize {
    match run.steady_step {
        Some(step) => run.steps.iter().position(|at| *at >= step).unwrap_or(run.latency_ms.len()),
        None => 0,
    }
}

fn steady(run: &InferenceRun) -> Vec<f64> {
    run.latency_ms[steady_start(run).min(run.latency_ms.len())..]
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect()
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

/// A steady percentile, refused when the run has too few samples to place it (S4).
fn steady_quantile(run: &InferenceRun, probability: f64) -> Result<f64, String> {
    let mut values = steady(run);
    if values.is_empty() {
        return Err(missing(run, "steady samples"));
    }
    if probability != 0.5 {
        if let Support::TooFew { needed } = stats::quantile_support(values.len(), probability) {
            return Err(format!(
                "{} has {} steady samples; its p{} needs at least {needed}.",
                run.label,
                values.len(),
                (probability * 100.0).round()
            ));
        }
    }
    values.sort_by(f64::total_cmp);
    workbench::quantile(&values, probability).ok_or_else(|| missing(run, "steady samples"))
}

/// The latency above the steady median before the steady state, summed: ms × steps.
fn warmup_cost(run: &InferenceRun) -> Result<f64, String> {
    let median = steady_quantile(run, 0.5)?;
    let start = steady_start(run).min(run.latency_ms.len());
    Ok(run.latency_ms[..start]
        .iter()
        .filter(|value| value.is_finite())
        .map(|value| (value - median).max(0.0))
        .sum())
}

fn between(run: &InferenceRun, a: f64, b: f64) -> Vec<f64> {
    let (from, to) = if a <= b { (a, b) } else { (b, a) };
    run.steps
        .iter()
        .zip(&run.latency_ms)
        .filter(|(step, value)| (**step as f64) >= from && (**step as f64) <= to && value.is_finite())
        .map(|(_, value)| *value)
        .collect()
}

/// Samples hashed into a run's identity: enough to tell runs apart, few enough that a longer
/// recording of the same run keeps its identity.
const IDENTITY_SAMPLES: usize = 32;

/// A run's identity: an FNV-1a hash of its seed and its first steps and latencies. The same run
/// opened from two places is counted once.
pub fn run_identity(run: &InferenceRun) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    let seed = run.trace.as_ref().and_then(|trace| trace.seed);
    feed(&seed.unwrap_or(i64::MIN).to_le_bytes());
    for (step, value) in run.steps.iter().zip(&run.latency_ms).take(IDENTITY_SAMPLES) {
        feed(&step.to_le_bytes());
        feed(&value.to_bits().to_le_bytes());
    }
    format!("{hash:016x}")
}

/// Every run open on either side, each repeat as its own run, in the order they were opened.
pub fn runs_of(sides: &[&InferenceRun]) -> Vec<InferenceRun> {
    let mut out: Vec<InferenceRun> = Vec::new();
    for side in sides {
        for run in side.runs() {
            let mut run = run.clone();
            run.replicates.clear();
            if !out.iter().any(|seen| run_identity(seen) == run_identity(&run)) {
                out.push(run);
            }
        }
    }
    out
}

/// The workbench's view of these runs.
pub fn board(runs: Vec<InferenceRun>, rule: AnomalyRule) -> Board {
    let built: Vec<Run> = runs
        .iter()
        .map(|run| Run {
            name: run.label.clone(),
            seed: run.trace.as_ref().and_then(|trace| trace.seed),
            knobs: run.knobs.values.clone(),
            identity: run_identity(run),
        })
        .collect();
    let (shared, varying) = workbench::split_knobs(&built);
    let mut ids: Vec<String> = built.iter().map(|run| run.identity.clone()).collect();
    ids.sort_unstable();
    let method = runs
        .iter()
        .find_map(|run| run.trace.as_ref().map(|trace| trace.framework.trim().to_string()))
        .filter(|framework| !framework.is_empty())
        .unwrap_or_else(|| "inference".to_string());
    Board {
        runs: built,
        shared,
        varying,
        key: ids.join("\n"),
        method,
        host: Arc::new(LatencyHost { runs, rule }),
    }
}

/// The memory file in the preferences directory.
pub fn memory_file(directory: &Path) -> PathBuf {
    directory.join(MEMORY_FILE)
}

/// What a finished session leaves as its record: the pinned model and digest, the person's own
/// call, every step with the program's answer, what was learned and proposed with the state the
/// program set, and the note labelled as the model's words. No paths.
pub fn session_record(
    model: &str,
    digest: Option<&str>,
    stopped: &str,
    your_call: &str,
    bench: &workbench::Workbench,
) -> Value {
    let board = bench.board();
    json!({
        "program": "ScalarScope",
        "model": model,
        "digest": digest,
        "stopped": stopped,
        "your_call": your_call.trim(),
        "runs": board.runs.iter().map(|run| json!({
            "name": run.name,
            "identity": run.identity,
            "knobs": run.knobs,
        })).collect::<Vec<_>>(),
        "steps": bench.steps.iter().map(|step| json!({
            "round": step.round,
            "tool": step.tool,
            "args": step.args,
            "ok": step.ok,
            "answer": step.result,
        })).collect::<Vec<_>>(),
        "learned": bench.learned.iter().map(|tool| json!({
            "name": tool.name,
            "formula": tool.formula,
            "meaning": tool.meaning,
        })).collect::<Vec<_>>(),
        "proposed": bench.proposed.iter().map(|hypothesis| json!({
            "id": hypothesis.id,
            "statement": hypothesis.statement_on(board),
            "state": hypothesis.state().map(|state| state.word()),
        })).collect::<Vec<_>>(),
        "model_note": bench.note,
        "model_note_dropped": bench.note_dropped,
        "measures_not_used": measures_not_used(bench),
    })
}

/// The built-in measures no call in the session used (M4).
pub fn measures_not_used(bench: &workbench::Workbench) -> Vec<&'static str> {
    MEASURES
        .iter()
        .map(|measure| measure.name)
        .filter(|name| {
            !bench.steps.iter().any(|step| {
                step.args
                    .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    .any(|word| word == *name)
            })
        })
        .collect()
}

const BOARDS_KEY: &str = "boards";
const RUNS_KEY: &str = "runs";
/// Boards and runs remembered, newest last. Older ones fall off.
const MAX_REMEMBERED: usize = 400;

/// Every run identity the workbench has seen: a hypothesis registered now cannot count them.
pub fn known_runs(file: &Path) -> Vec<String> {
    strings(&workbench::read_memory(file), RUNS_KEY)
}

/// Remember a board. True when its key is new, so it counts toward the next checkpoint.
pub fn note_board(file: &Path, board: &Board) -> Result<bool, std::io::Error> {
    let mut memory = workbench::read_memory(file);
    let mut boards = strings(&memory, BOARDS_KEY);
    let new = !boards.contains(&board.key);
    let mut runs = strings(&memory, RUNS_KEY);
    for run in &board.runs {
        if !runs.contains(&run.identity) {
            runs.push(run.identity.clone());
        }
    }
    if new {
        boards.push(board.key.clone());
    }
    for list in [&mut boards, &mut runs] {
        let excess = list.len().saturating_sub(MAX_REMEMBERED);
        list.drain(..excess);
    }
    memory.insert(BOARDS_KEY.to_string(), json!(boards));
    memory.insert(RUNS_KEY.to_string(), json!(runs));
    workbench::write_memory(file, &memory)?;
    Ok(new)
}

fn strings(memory: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    memory
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}
