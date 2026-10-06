//! The .NET app's golden RunTrace fixtures, read by the Rust review.
//!
//! The expectations come from `tests/Fixtures/InferenceOptimization/expected_assertions*.json`,
//! the same files the .NET FixtureTests read, so the two apps answer to one oracle.

use std::fs;
use std::path::PathBuf;

use scalarscope::open::{open_text, InferenceRun, Side};
use scalarscope::review::{pair, Pair};
use scalarscope::runtrace::{self, Severity};
use serde_json::Value;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/Fixtures/InferenceOptimization").join(name);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn json(name: &str) -> Value {
    serde_json::from_str(&fixture(name)).unwrap()
}

fn run(name: &str) -> InferenceRun {
    match open_text(&fixture(name), name).unwrap() {
        Side::Inference(run) => run,
        Side::Training(_) => panic!("{name} opened as a training history"),
    }
}

fn steady_mean(run: &InferenceRun, values: &[f64]) -> f64 {
    let start = run.steps.iter().position(|step| Some(*step) >= run.steady_step).unwrap();
    values[start..].iter().sum::<f64>() / (values.len() - start) as f64
}

fn steady_std(run: &InferenceRun) -> f64 {
    let start = run.steps.iter().position(|step| Some(*step) >= run.steady_step).unwrap();
    let tail = &run.latency_ms[start..];
    let mean = tail.iter().sum::<f64>() / tail.len() as f64;
    (tail.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / tail.len() as f64).sqrt()
}

#[test]
fn the_golden_pair_matches_the_dotnet_assertions() {
    let expected = &json("expected_assertions.json")["expected"];
    let baseline = run("baseline_tfrt_runtrace.json");
    let optimized = run("optimized_tfrt_runtrace.json");

    for (side, run, id_key) in [("baseline", &baseline, "baselineRunId"), ("optimized", &optimized, "optimizedRunId")] {
        let trace = run.trace.as_ref().expect("a stored trace");
        assert!(trace.validation.is_valid(), "{side}: {:?}", trace.validation.errors);
        assert_eq!(trace.run_id, expected["validation"][id_key].as_str().unwrap());
        assert!(trace.stored_milestones);
        let milestones = &expected["milestones"][side];
        assert_eq!(run.warmup_end, milestones["warmup_end"].as_i64(), "{side}");
        assert_eq!(run.steady_step, milestones["steady_state_start"].as_i64(), "{side}");
        let prints = &expected["fingerprints"][side];
        assert_eq!(trace.fingerprints.model, prints["model"].as_str().unwrap());
        assert_eq!(trace.fingerprints.dataset, prints["dataset"].as_str().unwrap());
        assert_eq!(trace.fingerprints.code, prints["code"].as_str().unwrap());
        assert_eq!(trace.fingerprints.environment, prints["environment"].as_str().unwrap());
        let capabilities = &expected["capabilities"][side];
        assert_eq!(!run.throughput.is_empty(), capabilities["hasThroughput"].as_bool().unwrap(), "{side}");
        assert_eq!(!run.memory_mb.is_empty(), capabilities["hasMemory"].as_bool().unwrap(), "{side}");
        let metrics = &expected["steadyStateMetrics"][side];
        assert!((steady_mean(run, &run.latency_ms) - metrics["latencyMean"].as_f64().unwrap()).abs() < 1e-9, "{side}");
        assert!((steady_std(run) - metrics["latencyStdDev"].as_f64().unwrap()).abs() < 1e-9, "{side}");
        assert!((steady_mean(run, &run.throughput) - metrics["throughputMean"].as_f64().unwrap()).abs() < 0.05, "{side}");
    }
    let intent = &expected["comparisonIntent"];
    assert_eq!(baseline.label, intent["labelA"].as_str().unwrap());
    assert_eq!(optimized.label, intent["labelB"].as_str().unwrap());

    let Pair::Inference(review) = pair(&Side::Inference(baseline.clone()), &Side::Inference(optimized.clone())).unwrap() else {
        panic!("inference");
    };
    let deltas = &expected["expectedDeltas"];
    assert_eq!(review.fired.contains(&"ΔTc".to_string()), deltas["deltaTc"]["fired"].as_bool().unwrap());
    let earlier = baseline.steady_step.unwrap() - optimized.steady_step.unwrap();
    assert!(earlier >= deltas["deltaTc"]["minDeltaSteps"].as_i64().unwrap());
    assert_eq!(deltas["deltaTc"]["direction"], "optimized_stabilizes_earlier");
    assert!(review.verdict.contains(&format!("Stabilizes {earlier} steps earlier")), "{}", review.verdict);
    assert_eq!(review.fired.contains(&"ΔO".to_string()), deltas["deltaO"]["fired"].as_bool().unwrap());
    assert_eq!(review.fired.contains(&"ΔF".to_string()), deltas["deltaF"]["fired"].as_bool().unwrap());
    assert!(deltas["deltaTd"]["shouldBeSuppressed"].as_bool().unwrap() && !review.fired.contains(&"ΔTd".to_string()));
    assert!(deltas["deltaA"]["shouldBeSuppressed"].as_bool().unwrap() && !review.fired.contains(&"ΔĀ".to_string()));

    // The model may differ; dataset, code and environment match. The baseline's warmup ends at
    // step 12 of 20, past half the run, which the .NET guardrail also notes.
    assert_eq!(
        review.notices,
        vec![
            "Model changed (expected for optimization)".to_string(),
            format!("Baseline: {}", runtrace::HIGH_WARMUP),
        ]
    );
}

#[test]
fn the_nearly_identical_pair_fires_nothing() {
    let expected = json("expected_assertions_nearly_identical.json");
    let left = run("nearly_identical_baseline_tfrt_runtrace.json");
    let right = run("nearly_identical_optimized_tfrt_runtrace.json");
    let Pair::Inference(review) = pair(&Side::Inference(left), &Side::Inference(right)).unwrap() else {
        panic!("inference");
    };
    assert!(review.fired.is_empty(), "{:?} / {}", review.fired, review.verdict);
    let text = expected.to_string();
    for symbol in ["deltaTc", "deltaO", "deltaF"] {
        assert!(text.contains(symbol), "the oracle names {symbol}");
    }
}

#[test]
fn the_broken_fixture_is_rejected_and_blocks_the_comparison() {
    let expected = json("expected_assertions_broken.json");
    let broken = run("broken_baseline_tfrt_runtrace.json");
    let trace = broken.trace.as_ref().unwrap();
    let validation = &expected["expectedValidation"];
    assert!(!trace.validation.is_valid());
    assert_eq!(trace.validation.errors.len() as u64, validation["errorCount"].as_u64().unwrap());
    let error = &trace.validation.errors[0];
    let oracle = &validation["errors"][0];
    assert_eq!(error.code, oracle["code"].as_str().unwrap());
    assert_eq!(error.severity, Severity::Error);
    assert!(error.message.contains(oracle["messageContains"].as_str().unwrap()), "{}", error.message);
    assert!(error.path.contains(oracle["pathContains"].as_str().unwrap()), "{}", error.path);
    let index = oracle["context"]["indexOfViolation"].as_u64().unwrap() as usize;
    assert_eq!(error.index, Some(index));
    let raw = json("broken_baseline_tfrt_runtrace.json");
    assert_eq!(raw["timeline"]["steps"][index], oracle["context"]["repeatedValue"]);
    assert!(trace.validation.warnings.is_empty() && trace.validation.infos.is_empty(), "the validator short-circuits");

    let message = expected["expectedUserMessage"].clone();
    let blocked = pair(&Side::Inference(broken.clone()), &Side::Inference(broken.clone())).unwrap_err();
    assert!(blocked.starts_with(message["title"].as_str().unwrap()), "{blocked}");
    assert!(blocked.contains(message["bullets"][0].as_str().unwrap()), "{blocked}");
    assert!(blocked.contains(message["actionableAdvice"][0].as_str().unwrap()), "{blocked}");

    // Against a valid run, the broken side is named.
    let optimized = run("optimized_tfrt_runtrace.json");
    let named = pair(&Side::Inference(broken.clone()), &Side::Inference(optimized)).unwrap_err();
    assert!(named.contains(&format!("{}: Timeline steps repeat at index {index}", broken.label)), "{named}");
}

fn trace(edit: impl FnOnce(&mut Value)) -> runtrace::RunTrace {
    let mut value = json("baseline_tfrt_runtrace.json");
    edit(&mut value);
    runtrace::parse(value.as_object().unwrap()).unwrap()
}

fn codes(trace: &runtrace::RunTrace) -> Vec<String> {
    let result = runtrace::validate(trace);
    result.errors.iter().chain(&result.warnings).chain(&result.infos).map(|issue| issue.code.clone()).collect()
}

#[test]
fn each_validator_rule_has_its_code() {
    let cases: Vec<(&str, Box<dyn FnOnce(&mut Value)>)> = vec![
        ("RT_TIMELINE_EMPTY", Box::new(|v: &mut Value| v["timeline"]["steps"] = Value::Array(vec![]))),
        ("RT_TIMELINE_NEGATIVE_STEP", Box::new(|v: &mut Value| v["timeline"]["steps"][0] = (-1).into())),
        ("RT_TIMELINE_LENGTH_MISMATCH", Box::new(|v: &mut Value| v["timeline"]["wallTimeSeconds"] = serde_json::json!([0, 1]))),
        ("RT_SCALAR_LENGTH_MISMATCH", Box::new(|v: &mut Value| v["scalars"]["series"][2]["values"] = serde_json::json!([1, 2]))),
        ("RT_SCALAR_ALL_NULL", Box::new(|v: &mut Value| v["scalars"]["series"][2]["values"] = Value::Array(vec![Value::Null; 20]))),
        ("RT_SCALAR_INVALID_VALUE", Box::new(|v: &mut Value| v["scalars"]["series"][2]["values"][3] = "NaN".into())),
        ("RT_SCALAR_AGG_NO_MILESTONE", Box::new(|v: &mut Value| {
            v["scalars"]["series"][2]["aggregation"] = "mean".into();
            v["milestones"]["list"] = Value::Array(vec![]);
        })),
        ("RT_SCALAR_HIGH_NULL_DENSITY", Box::new(|v: &mut Value| {
            for index in 0..7 {
                v["scalars"]["series"][2]["values"][index] = Value::Null;
            }
        })),
        ("RT_SCALAR_LEADING_NULLS", Box::new(|v: &mut Value| v["scalars"]["series"][2]["values"][0] = Value::Null)),
        ("RT_SCALAR_TRAILING_NULLS", Box::new(|v: &mut Value| {
            for index in 9..20 {
                v["scalars"]["series"][2]["values"][index] = Value::Null;
            }
        })),
        ("RT_MILESTONE_OUT_OF_RANGE", Box::new(|v: &mut Value| v["milestones"]["list"][2]["step"] = 40.into())),
        ("RT_MILESTONE_MISSING_STEADY_STATE", Box::new(|v: &mut Value| {
            v["milestones"]["list"] = serde_json::json!([{ "type": "steady_state_end", "step": 19 }]);
        })),
        ("RT_ENVIRONMENT_MISSING", Box::new(|v: &mut Value| v["metadata"]["environmentFingerprint"] = "".into())),
        ("RT_FINGERPRINT_INVALID", Box::new(|v: &mut Value| v["metadata"]["codeFingerprint"] = "not-a-hash".into())),
        ("RT_CAPABILITY_MISMATCH", Box::new(|v: &mut Value| v["capabilities"]["hasLoss"] = true.into())),
    ];
    for (code, edit) in cases {
        let found = codes(&trace(edit));
        assert!(found.contains(&code.to_string()), "{code}: {found:?}");
    }
    assert!(codes(&trace(|_| {})).is_empty());
}

#[test]
fn fingerprints_follow_the_comparer_rules() {
    let base = trace(|_| {}).fingerprints;
    let messages = |left: &runtrace::Fingerprints, right: &runtrace::Fingerprints| {
        runtrace::compare_fingerprints(left, right)
            .into_iter()
            .map(|note| (note.code, note.severity))
            .collect::<Vec<_>>()
    };
    assert!(messages(&base, &base).is_empty());
    let mut other = base.clone();
    other.dataset = "f".repeat(64);
    assert!(messages(&base, &other).contains(&(Some("CMP_FINGERPRINT_DATASET_MISMATCH".to_string()), Severity::Error)));
    let mut absent = base.clone();
    absent.code = "unknown:2026-01-01".to_string();
    assert!(messages(&absent, &absent).contains(&(Some("CMP_FINGERPRINT_CODE_ABSENT".to_string()), Severity::Warning)));
    let mut changed = base.clone();
    changed.code = "e".repeat(64);
    assert!(messages(&base, &changed).contains(&(Some("CMP_FINGERPRINT_CODE_MISMATCH".to_string()), Severity::Warning)));
}

#[test]
fn guardrails_note_a_long_warmup_and_a_missing_milestone() {
    let long = trace(|v| v["milestones"]["list"][0]["step"] = 15.into());
    assert!(runtrace::guardrails(&long).contains(&runtrace::HIGH_WARMUP.to_string()));
    let none = trace(|v| v["milestones"]["list"] = Value::Array(vec![]));
    assert!(runtrace::guardrails(&none).contains(&runtrace::NO_STEADY_STATE.to_string()));
    assert_eq!(runtrace::guardrails(&trace(|_| {})), vec![runtrace::HIGH_WARMUP.to_string()]);
    let optimized = json("optimized_tfrt_runtrace.json");
    assert!(runtrace::guardrails(&runtrace::parse(optimized.as_object().unwrap()).unwrap()).is_empty());
}

#[test]
fn a_trace_without_milestones_falls_back_to_detection() {
    let mut value = json("baseline_tfrt_runtrace.json");
    value["milestones"]["list"] = Value::Array(vec![]);
    let Side::Inference(run) = open_text(&value.to_string(), "plain").unwrap() else { panic!("inference") };
    assert!(!run.trace.as_ref().unwrap().stored_milestones);
}

#[test]
fn memory_in_bytes_is_shown_in_mib() {
    let mut value = json("baseline_tfrt_runtrace.json");
    value["scalars"]["series"][2]["name"] = "memory_bytes".into();
    value["scalars"]["series"][2]["values"] = Value::Array(vec![Value::from(1_048_576 * 512_u64); 20]);
    let Side::Inference(run) = open_text(&value.to_string(), "bytes").unwrap() else { panic!("inference") };
    assert!(run.memory_mb.iter().all(|value| (*value - 512.0).abs() < 1e-9));
}

#[test]
fn a_csv_milestone_is_a_step_number_not_an_index() {
    let rows: String = (0..40).map(|index| format!("{},{}\n", 100 + index, if index < 8 { 30.0 - index as f64 } else { 20.0 })).collect();
    let Side::Inference(run) = open_text(&format!("step,latency_ms\n{rows}"), "offset").unwrap() else { panic!("inference") };
    let steady = run.steady_step.expect("a steady state");
    assert!(steady >= 100, "{steady} should be a step from the file");
}
