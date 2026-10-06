//! ScalarScope's stored RunTrace JSON (`schemaVersion`, `timeline`, `scalars.series`,
//! `milestones.list`, fingerprints), as the .NET app reads and validates it.
//!
//! The validator follows `RunTraceValidator.cs` stage by stage, with the same codes and the
//! same short-circuit: after a timeline error no later stage runs, because they all index
//! the timeline. The fingerprint check follows `RunTraceComparer.CompareFingerprints`, and
//! the guardrails follow `TfrtGuardrails.ValidateForAnalysis`.

use serde_json::{Map, Value};

/// Null share above which a series draws a warning (`RuntimeValidationOptions.MaxNullDensity`).
pub const MAX_NULL_DENSITY: f64 = 0.30;
/// Warmup past this share of the run draws a guardrail note (`TfrtGuardrails.MaxWarmupRatio`).
pub const MAX_WARMUP_RATIO: f64 = 0.5;

pub const NO_STEADY_STATE: &str = "⚠ Steady state not established";
pub const HIGH_WARMUP: &str = "⚠ Warmup exceeds 50% of run";
pub const AGGREGATED_ONLY: &str = "⚠ Only aggregated stats available - time-based deltas disabled";

/// Latency names in the order the TFRT preset maps them (`TfrtRuntimePreset.CreateMappings`).
pub const LATENCY_NAMES: [&str; 5] = ["latency_ms", "latency", "inference_latency", "avg_latency_ms", "p50_latency_ms"];
pub const THROUGHPUT_NAMES: [&str; 5] = ["throughput_items_per_sec", "throughput", "items_per_sec", "samples_per_sec", "qps"];
/// Memory names; `memory_bytes` and `gpu_memory_bytes` are converted to MiB.
pub const MEMORY_NAMES: [&str; 4] = ["memory_mb", "memory_bytes", "peak_memory", "gpu_memory_bytes"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub path: String,
    /// For a non-monotonic timeline, the index where the steps stop increasing.
    pub index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Validation {
    pub errors: Vec<Issue>,
    pub warnings: Vec<Issue>,
    pub infos: Vec<Issue>,
}

impl Validation {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// A broken timeline stops the comparison; other errors are reported but do not.
    pub fn timeline_rejects(&self) -> bool {
        self.errors.iter().any(|issue| issue.code.starts_with("RT_TIMELINE"))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scalar {
    pub name: String,
    pub unit: String,
    pub aggregation: String,
    pub values: Vec<Option<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub kind: String,
    pub step: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fingerprints {
    pub model: String,
    pub dataset: String,
    pub code: String,
    pub environment: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunTrace {
    pub run_id: String,
    pub label: String,
    pub run_type: String,
    pub framework: String,
    pub steps: Vec<i64>,
    pub wall_time_seconds: Option<usize>,
    pub epoch: Option<usize>,
    pub scalars: Vec<Scalar>,
    pub milestones: Vec<Milestone>,
    pub fingerprints: Fingerprints,
    pub capabilities: Map<String, Value>,
}

/// What a stored trace adds to an inference run: where it came from, and what the validator found.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceInfo {
    pub run_id: String,
    pub fingerprints: Fingerprints,
    pub validation: Validation,
    pub guardrails: Vec<String>,
    /// True when the milestones came from the file, not from the latency values.
    pub stored_milestones: bool,
}

/// A JSON object is a stored RunTrace when it has a string `schemaVersion` and a `scalars.series` array.
pub fn looks_like_runtrace(map: &Map<String, Value>) -> bool {
    map.get("schemaVersion").is_some_and(Value::is_string)
        && map
            .get("scalars")
            .and_then(|scalars| scalars.get("series"))
            .is_some_and(Value::is_array)
        && !map.contains_key("traceEvents")
        && !map.contains_key("trajectory")
}

pub fn parse(map: &Map<String, Value>) -> Result<RunTrace, String> {
    let text = |value: Option<&Value>| value.and_then(Value::as_str).unwrap_or("").to_string();
    let timeline = map.get("timeline").and_then(Value::as_object);
    let steps = match timeline.and_then(|timeline| timeline.get("steps")) {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_i64().ok_or_else(|| "timeline.steps holds a value that is not a whole number.".to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err("timeline.steps is not a list.".to_string()),
    };
    let length_of = |key: &str| {
        timeline
            .and_then(|timeline| timeline.get(key))
            .and_then(Value::as_array)
            .map(Vec::len)
    };
    let mut scalars = Vec::new();
    for (index, item) in map["scalars"]["series"].as_array().into_iter().flatten().enumerate() {
        let name = text(item.get("name"));
        if name.is_empty() {
            return Err(format!("scalars.series[{index}] has no name."));
        }
        let values = item
            .get("values")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("Scalar '{name}' has no values list."))?
            .iter()
            .map(|value| match value {
                Value::Null => Ok(None),
                Value::Number(number) => Ok(number.as_f64()),
                // JSON has no NaN or Infinity, so a writer that kept them used strings.
                Value::String(word) => match word.as_str() {
                    "NaN" => Ok(Some(f64::NAN)),
                    "Infinity" | "+Infinity" => Ok(Some(f64::INFINITY)),
                    "-Infinity" => Ok(Some(f64::NEG_INFINITY)),
                    _ => Err(format!("Scalar '{name}' holds a value that is not a number.")),
                },
                _ => Err(format!("Scalar '{name}' holds a value that is not a number.")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        scalars.push(Scalar {
            name,
            unit: text(item.get("unit")),
            aggregation: text(item.get("aggregation")),
            values,
        });
    }
    let milestones = map
        .get("milestones")
        .and_then(|milestones| milestones.get("list"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(Milestone {
                kind: item.get("type")?.as_str()?.to_string(),
                step: item.get("step")?.as_i64()?,
            })
        })
        .collect();
    let metadata = map.get("metadata");
    let fingerprint = |key: &str| text(metadata.and_then(|metadata| metadata.get(key)));
    Ok(RunTrace {
        run_id: text(map.get("runId")),
        label: text(map.get("label")),
        run_type: text(map.get("runType")),
        framework: text(map.get("framework")),
        steps,
        wall_time_seconds: length_of("wallTimeSeconds"),
        epoch: length_of("epoch"),
        scalars,
        milestones,
        fingerprints: Fingerprints {
            model: fingerprint("modelFingerprint"),
            dataset: fingerprint("datasetFingerprint"),
            code: fingerprint("codeFingerprint"),
            environment: fingerprint("environmentFingerprint"),
        },
        capabilities: map.get("capabilities").and_then(Value::as_object).cloned().unwrap_or_default(),
    })
}

fn issue(code: &str, severity: Severity, message: String, path: String) -> Issue {
    Issue {
        code: code.to_string(),
        severity,
        message,
        path,
        index: None,
    }
}

pub fn validate(trace: &RunTrace) -> Validation {
    let mut result = Validation::default();
    timeline(trace, &mut result);
    if result.timeline_rejects() {
        return result;
    }
    scalars(trace, &mut result);
    nulls(trace, &mut result);
    milestones(trace, &mut result);
    metadata(trace, &mut result);
    capabilities(trace, &mut result);
    result
}

fn timeline(trace: &RunTrace, result: &mut Validation) {
    let steps = &trace.steps;
    if steps.is_empty() {
        result.errors.push(issue(
            "RT_TIMELINE_EMPTY",
            Severity::Error,
            "No steps provided in timeline".to_string(),
            "timeline.steps".to_string(),
        ));
        return;
    }
    for (index, step) in steps.iter().enumerate() {
        if *step < 0 {
            result.errors.push(Issue {
                index: Some(index),
                ..issue(
                    "RT_TIMELINE_NEGATIVE_STEP",
                    Severity::Error,
                    format!("Step at index {index} is negative: {step}"),
                    format!("timeline.steps[{index}]"),
                )
            });
        }
    }
    if let Some(index) = (1..steps.len()).find(|index| steps[*index] <= steps[index - 1]) {
        result.errors.push(Issue {
            index: Some(index),
            ..issue(
                "RT_TIMELINE_NON_MONOTONIC",
                Severity::Error,
                format!("Steps not strictly increasing at index {index}: {} → {}", steps[index - 1], steps[index]),
                format!("timeline.steps[{index}]"),
            )
        });
    }
    for (key, length) in [("wallTimeSeconds", trace.wall_time_seconds), ("epoch", trace.epoch)] {
        if let Some(length) = length.filter(|length| *length != steps.len()) {
            result.errors.push(issue(
                "RT_TIMELINE_LENGTH_MISMATCH",
                Severity::Error,
                format!("{key} length ({length}) != steps length ({})", steps.len()),
                format!("timeline.{key}"),
            ));
        }
    }
}

fn scalars(trace: &RunTrace, result: &mut Validation) {
    let count = trace.steps.len();
    for series in &trace.scalars {
        let path = format!("scalars.{}", series.name);
        if series.values.len() != count {
            result.errors.push(issue(
                "RT_SCALAR_LENGTH_MISMATCH",
                Severity::Error,
                format!("Scalar '{}' length ({}) != steps length ({count})", series.name, series.values.len()),
                path,
            ));
            continue;
        }
        if series.values.iter().all(Option::is_none) {
            result.errors.push(issue(
                "RT_SCALAR_ALL_NULL",
                Severity::Error,
                format!("Scalar '{}' contains only null values", series.name),
                path,
            ));
            continue;
        }
        if let Some((index, value)) = series
            .values
            .iter()
            .enumerate()
            .find_map(|(index, value)| value.filter(|value| !value.is_finite()).map(|value| (index, value)))
        {
            result.errors.push(issue(
                "RT_SCALAR_INVALID_VALUE",
                Severity::Error,
                format!("Scalar '{}' has invalid value at index {index}: {value}", series.name),
                format!("{path}.values[{index}]"),
            ));
        }
        let aggregated = !series.aggregation.is_empty() && !series.aggregation.eq_ignore_ascii_case("none");
        if aggregated && trace.milestones.is_empty() {
            result.errors.push(issue(
                "RT_SCALAR_AGG_NO_MILESTONE",
                Severity::Error,
                format!("Aggregated scalar '{}' requires at least one milestone for context", series.name),
                path,
            ));
        }
    }
}

fn nulls(trace: &RunTrace, result: &mut Validation) {
    for series in &trace.scalars {
        let values = &series.values;
        if values.is_empty() {
            continue;
        }
        let path = format!("scalars.{}", series.name);
        let density = values.iter().filter(|value| value.is_none()).count() as f64 / values.len() as f64;
        if density > MAX_NULL_DENSITY {
            result.warnings.push(issue(
                "RT_SCALAR_HIGH_NULL_DENSITY",
                Severity::Warning,
                format!(
                    "Scalar '{}' has {:.0}% null values (threshold: {:.0}%)",
                    series.name,
                    density * 100.0,
                    MAX_NULL_DENSITY * 100.0
                ),
                path.clone(),
            ));
        }
        let leading = values.iter().take_while(|value| value.is_none()).count();
        if leading > 0 && leading < values.len() {
            result.infos.push(issue(
                "RT_SCALAR_LEADING_NULLS",
                Severity::Info,
                format!("Scalar '{}' has {leading} leading nulls (warmup assumed)", series.name),
                path.clone(),
            ));
        }
        let trailing = values.iter().rev().take_while(|value| value.is_none()).count();
        let ratio = trailing as f64 / values.len() as f64;
        if trailing > 0 && ratio > 0.5 {
            result.errors.push(issue(
                "RT_SCALAR_TRAILING_NULLS",
                Severity::Error,
                format!(
                    "Scalar '{}' has {trailing} trailing nulls ({:.0}%) - signal never stabilizes",
                    series.name,
                    ratio * 100.0
                ),
                path,
            ));
        }
    }
}

fn milestones(trace: &RunTrace, result: &mut Validation) {
    let (Some(min), Some(max)) = (trace.steps.iter().min(), trace.steps.iter().max()) else {
        return;
    };
    for milestone in &trace.milestones {
        if milestone.step < *min || milestone.step > *max {
            result.errors.push(issue(
                "RT_MILESTONE_OUT_OF_RANGE",
                Severity::Error,
                format!(
                    "Milestone '{}' at step {} is outside timeline range [{min}, {max}]",
                    milestone.kind, milestone.step
                ),
                format!("milestones[step={}]", milestone.step),
            ));
        }
    }
    if trace.run_type.eq_ignore_ascii_case("inference")
        && !trace
            .milestones
            .iter()
            .any(|milestone| milestone.kind == "steady_state_start" || milestone.kind == "warmup_end")
    {
        result.warnings.push(issue(
            "RT_MILESTONE_MISSING_STEADY_STATE",
            Severity::Warning,
            "Inference run lacks steady-state or warmup-end marker".to_string(),
            "milestones".to_string(),
        ));
    }
    if trace.run_type.eq_ignore_ascii_case("training")
        && !trace
            .milestones
            .iter()
            .any(|milestone| milestone.kind == "epoch_start" || milestone.kind == "epoch_end")
    {
        result.infos.push(issue(
            "RT_MILESTONE_MISSING_EPOCH",
            Severity::Info,
            "Training run has no epoch markers".to_string(),
            "milestones".to_string(),
        ));
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn short(value: &str) -> &str {
    value.char_indices().nth(16).map_or(value, |(end, _)| &value[..end])
}

fn metadata(trace: &RunTrace, result: &mut Validation) {
    let prints = &trace.fingerprints;
    if prints.environment.trim().is_empty() {
        result.errors.push(issue(
            "RT_ENVIRONMENT_MISSING",
            Severity::Error,
            "Environment fingerprint is required".to_string(),
            "metadata.environmentFingerprint".to_string(),
        ));
    } else if !is_sha256(&prints.environment) {
        result.errors.push(issue(
            "RT_FINGERPRINT_INVALID",
            Severity::Error,
            format!("Environment fingerprint is not valid SHA-256: {}...", short(&prints.environment)),
            "metadata.environmentFingerprint".to_string(),
        ));
    }
    for (value, path, name) in [
        (&prints.model, "metadata.modelFingerprint", "Model"),
        (&prints.dataset, "metadata.datasetFingerprint", "Dataset"),
        (&prints.code, "metadata.codeFingerprint", "Code"),
    ] {
        if !value.trim().is_empty() && !is_sha256(value) {
            result.errors.push(issue(
                "RT_FINGERPRINT_INVALID",
                Severity::Error,
                format!("{name} fingerprint is not valid SHA-256: {}...", short(value)),
                path.to_string(),
            ));
        }
    }
}

fn capabilities(trace: &RunTrace, result: &mut Validation) {
    let has = |names: &[&str]| trace.scalars.iter().any(|series| names.contains(&series.name.as_str()));
    let checks: [(&str, bool, &str); 5] = [
        ("hasLatency", has(&["latency_ms", "latency"]), "latency"),
        ("hasThroughput", has(&["throughput_items_per_sec", "throughput"]), "throughput"),
        ("hasMemory", has(&["memory_bytes", "memory_mb", "memory"]), "memory"),
        ("hasLoss", has(&["loss", "train_loss"]), "loss"),
        ("hasAccuracy", has(&["accuracy", "train_accuracy"]), "accuracy"),
    ];
    for (flag, exists, data) in checks {
        let claimed = trace.capabilities.get(flag).and_then(Value::as_bool).unwrap_or(false);
        if claimed != exists {
            let message = if claimed {
                format!("Capability '{flag}' is true but no '{data}' series found")
            } else {
                format!("Capability '{flag}' is false but '{data}' series exists")
            };
            result.errors.push(issue("RT_CAPABILITY_MISMATCH", Severity::Error, message, format!("capabilities.{flag}")));
        }
    }
}

/// `absent`, an empty value, or a dated `unknown:*` value is not a fingerprint.
pub fn is_absent(value: &str) -> bool {
    let value = value.trim();
    value.is_empty() || value.eq_ignore_ascii_case("absent") || value.to_ascii_lowercase().starts_with("unknown:")
}

#[derive(Clone, Debug, PartialEq)]
pub struct FingerprintNote {
    pub code: Option<String>,
    pub severity: Severity,
    pub message: String,
}

/// The fingerprint check between two runs. The model may differ (that is often the
/// optimization); dataset, code and environment are expected to match. Two absent values
/// are not a match.
pub fn compare_fingerprints(left: &Fingerprints, right: &Fingerprints) -> Vec<FingerprintNote> {
    let mut notes = Vec::new();
    let note = |code: Option<&str>, severity, message: &str| FingerprintNote {
        code: code.map(str::to_string),
        severity,
        message: message.to_string(),
    };
    if left.model != right.model {
        notes.push(note(None, Severity::Info, "Model changed (expected for optimization)"));
    }
    if is_absent(&left.environment) || is_absent(&right.environment) {
        notes.push(note(None, Severity::Info, "Environment fingerprint is unknown"));
    } else if left.environment != right.environment {
        notes.push(note(None, Severity::Warning, "Environment facts in the traces differ"));
    }
    if is_absent(&left.dataset) || is_absent(&right.dataset) {
        notes.push(note(
            Some("CMP_FINGERPRINT_DATASET_ABSENT"),
            Severity::Warning,
            "Dataset hash is absent; dataset identity was not checked",
        ));
    } else if left.dataset != right.dataset {
        notes.push(note(
            Some("CMP_FINGERPRINT_DATASET_MISMATCH"),
            Severity::Error,
            "Dataset fingerprints differ - comparison invalid unless intentional",
        ));
    }
    if is_absent(&left.code) || is_absent(&right.code) {
        notes.push(note(
            Some("CMP_FINGERPRINT_CODE_ABSENT"),
            Severity::Warning,
            "Code hash is absent; code identity was not checked",
        ));
    } else if left.code != right.code {
        notes.push(note(
            Some("CMP_FINGERPRINT_CODE_MISMATCH"),
            Severity::Warning,
            "Code fingerprints differ - ensure changes are optimization-related",
        ));
    }
    notes
}

pub fn guardrails(trace: &RunTrace) -> Vec<String> {
    let mut notes = Vec::new();
    if !trace
        .milestones
        .iter()
        .any(|milestone| milestone.kind == "steady_state_start" || milestone.kind == "warmup_end")
    {
        notes.push(NO_STEADY_STATE.to_string());
    }
    if let Some(warmup) = milestone(trace, "warmup_end") {
        if !trace.steps.is_empty() && warmup as f64 / trace.steps.len() as f64 > MAX_WARMUP_RATIO {
            notes.push(HIGH_WARMUP.to_string());
        }
    }
    if !trace.scalars.iter().any(|series| series.values.len() > 1) {
        notes.push(AGGREGATED_ONLY.to_string());
    }
    notes
}

pub fn milestone(trace: &RunTrace, kind: &str) -> Option<i64> {
    trace.milestones.iter().find(|milestone| milestone.kind == kind).map(|milestone| milestone.step)
}

/// The first series whose name is in `names`, in the order of `names`.
pub fn series<'a>(trace: &'a RunTrace, names: &[&str]) -> Option<&'a Scalar> {
    names
        .iter()
        .find_map(|name| trace.scalars.iter().find(|series| series.name == *name))
}

/// The user message for a comparison a broken timeline stops, as the .NET app wrote it.
pub fn blocked_message(left: (&str, &str, &Validation), right: (&str, &str, &Validation)) -> String {
    let mut lines = vec!["We couldn't analyze this run".to_string()];
    let same_run = left.1 == right.1;
    let mut repeat = append_errors(&mut lines, if same_run { None } else { Some(left.0) }, left.2);
    if !same_run {
        repeat |= append_errors(&mut lines, Some(right.0), right.2);
    }
    if repeat {
        lines.push("Ensure iteration indices are strictly increasing".to_string());
    }
    lines.join("\n")
}

/// The same message when only one side is a stored trace.
pub fn blocked_side_message(label: &str, validation: Option<&Validation>) -> String {
    let mut lines = vec!["We couldn't analyze this run".to_string()];
    if let Some(validation) = validation {
        if append_errors(&mut lines, Some(label), validation) {
            lines.push("Ensure iteration indices are strictly increasing".to_string());
        }
    }
    lines.join("\n")
}

fn append_errors(lines: &mut Vec<String>, label: Option<&str>, validation: &Validation) -> bool {
    let prefix = label.filter(|label| !label.is_empty()).map(|label| format!("{label}: ")).unwrap_or_default();
    let mut repeat = false;
    for error in &validation.errors {
        match (error.code.as_str(), error.index) {
            ("RT_TIMELINE_NON_MONOTONIC", Some(index)) => {
                lines.push(format!("{prefix}Timeline steps repeat at index {index}"));
                repeat = true;
            }
            _ => lines.push(format!("{prefix}{}", error.message)),
        }
    }
    repeat
}
