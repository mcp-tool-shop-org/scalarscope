//! The workbench on inference runs: knobs, latency measures, and the shared crate's verdicts
//! on the simulated knob folder (`tests/Fixtures/Workbench`) and the 2.0 golden pair.

use std::path::PathBuf;

use serde_json::{json, Value};

use scalarscope::knobs::{self, Knobs, Source};
use scalarscope::open::{open_path, InferenceRun, Side};
use scalarscope::stats::AnomalyRule;
use scalarscope::workbench::{board, measures_not_used, run_identity, runs_of, session_record};
use workbench::{evaluate, propose, test_hypothesis, Proposal, State, Workbench};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/Fixtures")
}

fn inference(path: PathBuf) -> InferenceRun {
    match open_path(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())).side {
        Side::Inference(run) => run,
        other => panic!("not an inference run: {}", other.name()),
    }
}

fn knob_runs() -> Vec<InferenceRun> {
    let mut runs = Vec::new();
    for batch in [1, 4, 8] {
        for seed in 1..=3 {
            runs.push(inference(fixtures().join(format!("Workbench/batch{batch}_seed{seed}_runtrace.json"))));
        }
    }
    runs
}

fn golden() -> Vec<InferenceRun> {
    let folder = fixtures().join("InferenceOptimization");
    vec![
        inference(folder.join("baseline_tfrt_runtrace.json")),
        inference(folder.join("optimized_tfrt_runtrace.json")),
    ]
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scalarscope-workbench-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_runtrace_carries_its_knobs_seed_and_framework() {
    let run = inference(fixtures().join("Workbench/batch4_seed2_runtrace.json"));
    assert_eq!(run.knobs.values["batch"], json!(4));
    assert_eq!(run.knobs.values["precision"], json!("fp16"));
    assert_eq!(run.knobs.sources["batch"], Source::Trace);
    let trace = run.trace.as_ref().unwrap();
    assert_eq!(trace.seed, Some(2));
    assert_eq!(trace.framework, "simulated");
    // The 2.0 fixtures record no knobs.
    assert!(golden().iter().all(|run| run.knobs.is_empty()));
}

#[test]
fn folder_names_read_as_knobs_only_when_they_read_cleanly() {
    let read = knobs::from_name("batch=8_cuda_graphs=on,precision=fp16_threads=4");
    assert_eq!(read["batch"], json!(8));
    assert_eq!(read["cuda_graphs"], json!(true));
    assert_eq!(read["precision"], json!("fp16"));
    assert_eq!(read["threads"], json!(4));
    assert_eq!(knobs::from_name("lr=0.5")["lr"], json!(0.5));
    assert_eq!(knobs::from_name("trt=off")["trt"], json!(false));
    for name in ["baseline", "batch=", "=8", "batch=8 precision=fp16", "9lives=1", "a=1=2"] {
        assert!(knobs::from_name(name).is_empty(), "{name}");
    }
}

#[test]
fn the_trace_wins_then_the_knobs_file_then_the_folder_name() {
    let dir = scratch("sources").join("batch=2_threads=8");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(knobs::KNOBS_FILE), r#"{"batch": 16, "precision": "int8", "shape": [1, 3], "trt": true}"#).unwrap();
    let source = fixtures().join("Workbench/batch1_seed1_runtrace.json");
    std::fs::copy(&source, dir.join("run.json")).unwrap();
    let run = inference(dir.join("run.json"));
    let knobs = &run.knobs;
    assert_eq!(knobs.values["batch"], json!(1), "the trace's own value wins");
    assert_eq!(knobs.sources["batch"], Source::Trace);
    assert_eq!(knobs.values["precision"], json!("fp16"));
    assert_eq!(knobs.values["trt"], json!(true));
    assert_eq!(knobs.sources["trt"], Source::File);
    assert!(!knobs.values.contains_key("shape"), "a list is not a knob");
    assert_eq!(knobs.values["threads"], json!(8));
    assert_eq!(knobs.sources["threads"], Source::FolderName);
    assert_eq!(Source::FolderName.label(), "folder name");
    assert_eq!(Source::File.label(), "knobs.json");
    assert_eq!(Source::Trace.label(), "RunTrace metadata");

    // A run folder: its own knobs file and name.
    let folder = scratch("folder").join("precision=int8");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(&source, folder.join("benchmark.json")).unwrap();
    std::fs::write(folder.join(knobs::KNOBS_FILE), r#"{"threads": 2}"#).unwrap();
    let run = inference(folder.clone());
    assert_eq!(run.knobs.values["precision"], json!("fp16"), "the trace still wins");
    assert_eq!(run.knobs.values["threads"], json!(2));
    let mut empty = Knobs::default();
    empty.fill_from_folder(&folder);
    assert_eq!(empty.values["precision"], json!("int8"));
}

#[test]
fn the_board_splits_knobs_and_names_runs_by_their_data() {
    let runs = knob_runs();
    let board = board(runs.clone(), AnomalyRule::Mad);
    assert_eq!(board.runs.len(), 9);
    assert_eq!(board.varying, vec!["batch".to_string()]);
    assert_eq!(board.shared["precision"], json!("fp16"));
    assert_eq!(board.method, "simulated");
    assert_eq!(board.label("batch"), "batch size");
    assert_eq!(board.label("cuda_graphs"), "CUDA graphs");
    assert_eq!(board.label("odd_knob"), "odd knob");
    // A run opened twice, or as a replicate, counts once.
    let mut side = runs[0].clone();
    side.replicates = vec![runs[1].clone(), runs[0].clone()];
    let flat = runs_of(&[&side, &runs[1]]);
    assert_eq!(flat.len(), 2);
    assert_ne!(run_identity(&runs[0]), run_identity(&runs[1]));
    assert_eq!(golden().len(), 2);
    assert_eq!(scalarscope::workbench::board(golden(), AnomalyRule::Mad).method, "tensorflowrt");
}

#[test]
fn latency_measures_are_computed_by_the_program() {
    let runs = knob_runs();
    let board = board(runs, AnomalyRule::Mad);
    let column = |formula: &str| evaluate(&board, formula, &[]).unwrap();
    let values = |formula: &str| -> Vec<f64> {
        column(formula).values.iter().map(|(_, value)| *value.as_ref().unwrap()).collect()
    };
    let p50 = values("p50");
    // Base latency 4 + 1.5 × batch: 5.5, 10 and 16 ms.
    for (index, base) in [5.5, 5.5, 5.5, 10.0, 10.0, 10.0, 16.0, 16.0, 16.0].iter().enumerate() {
        assert!((p50[index] / base - 1.0).abs() < 0.03, "{index}: {}", p50[index]);
    }
    assert!(values("p99_over_p50").iter().all(|ratio| *ratio > 1.0));
    assert!(values("warmup_cost").iter().all(|cost| *cost > 0.0));
    assert!(values("memory_peak")[8] > values("memory_peak")[0]);
    assert!(values("anomaly_rate").iter().all(|rate| (0.0..0.2).contains(rate)));
    assert_eq!(values("samples")[0], 400.0);
    assert!(values("throughput_p50")[0] > values("throughput_p50")[8] / 8.0);
    let early = values("mean_between(0, 3) / p50");
    assert!(early.iter().all(|ratio| *ratio > 2.0), "{early:?}");
    let q = values("quantile_between(8, 399, 0.5)");
    assert!((q[0] / p50[0] - 1.0).abs() < 1e-9);
    assert!((values("steady_mean")[0] / p50[0] - 1.0).abs() < 0.2);
    assert_eq!(values("p90").len(), 9);
    assert_eq!(values("knob('batch')"), vec![1.0, 1.0, 1.0, 4.0, 4.0, 4.0, 8.0, 8.0, 8.0]);
}

#[test]
fn a_measure_a_run_cannot_supply_is_refused_with_its_reason() {
    let board = board(golden(), AnomalyRule::ThreeSigma);
    let reason = |formula: &str| {
        let column = evaluate(&board, formula, &[]).unwrap();
        column.values[0].1.clone().unwrap_err()
    };
    // Seven steady samples cannot place a 99th percentile (S4).
    assert!(reason("p99").contains("needs at least"), "{}", reason("p99"));
    assert!(reason("mean_between(500, 600)").contains("no latency between steps"));
    assert!(reason("quantile_between(0, 5, 2)").contains("q from 0 to 1"));
    assert!(evaluate(&board, "latency()", &[]).is_err());
    let anomaly = evaluate(&board, "anomaly_rate", &[]).unwrap();
    assert!(anomaly.values[0].1.is_ok());
}

#[test]
fn on_the_knob_folder_a_hypothesis_gets_a_program_set_state() {
    let board = board(knob_runs(), AnomalyRule::Mad);
    let proposal = Proposal { knob: "batch", formula: "p50", direction: "higher", why: "A bigger batch does more work per step." };
    let hypothesis = propose(&board, &[], &[], &proposal, "2026-10-06").unwrap();
    assert_eq!(hypothesis.statement_on(&board), "When batch size goes up, p50 goes higher.");
    let (evaluation, comparison) = test_hypothesis(&board, &hypothesis, &[], "2026-10-06");
    // Three against three at the ends, a clean separation: the exact test's smallest p is 1/20.
    assert_eq!(evaluation.state, State::Supported, "{}", evaluation.detail);
    let comparison = comparison.unwrap();
    assert_eq!(comparison.low.level, "1");
    assert_eq!(comparison.high.level, "8");
    assert_eq!(comparison.middle, 1);

    // A knob every run shares is not testable here.
    let shared = Proposal { knob: "precision", formula: "p50", direction: "lower", why: "Lower precision is cheaper." };
    let hypothesis = propose(&board, &[], &[], &shared, "2026-10-06").unwrap();
    let (evaluation, _) = test_hypothesis(&board, &hypothesis, &[], "2026-10-06");
    assert_eq!(evaluation.state, State::Untestable);
}

#[test]
fn on_the_golden_pair_every_hypothesis_is_not_testable() {
    let board = board(golden(), AnomalyRule::Mad);
    assert!(board.varying.is_empty() && board.shared.is_empty());
    let proposal = Proposal { knob: "batch", formula: "p50", direction: "higher", why: "Batching adds work." };
    // No run records a batch knob, so the program refuses the hypothesis outright.
    let refused = propose(&board, &[], &[], &proposal, "2026-10-06").unwrap_err();
    assert!(refused.contains("not a field of these recipes"), "{refused}");
}

#[test]
fn a_session_record_names_the_model_and_holds_no_paths() {
    let board = board(knob_runs(), AnomalyRule::Mad);
    let mut bench = Workbench::new(board, Vec::new(), Vec::new(), "2026-10-06");
    assert!(bench.system_prompt().contains("You are the ScalarScope workbench"));
    assert!(bench.system_prompt().contains("quantile_between(a, b, q)"));
    assert!(bench.opening().contains("batch (batch 1 seed 1 1"), "{}", bench.opening());
    bench.call("measure", &json!({"formula": "p50"}));
    bench.call("propose_hypothesis", &json!({"knob": "batch", "formula": "p99_over_p50", "knob_change": "raise", "formula_moves": "up", "why": "Bigger batches spike more often."}));
    bench.call("finish", &json!({"note": "The tail grows with the batch; precision did not vary."}));
    let record = session_record("qwen2.5:14b", Some("7cdf5a0187d5"), "The model finished.", "I expect batch to raise the tail.", &bench);
    assert_eq!(record["model"], "qwen2.5:14b");
    assert_eq!(record["digest"], "7cdf5a0187d5");
    assert_eq!(record["your_call"], "I expect batch to raise the tail.");
    assert_eq!(record["runs"].as_array().unwrap().len(), 9);
    assert_eq!(record["proposed"][0]["statement"], "When batch size goes up, p99_over_p50 goes higher.");
    assert!(record["proposed"][0]["state"].is_string());
    assert_eq!(record["model_note"], "The tail grows with the batch; precision did not vary.");
    let not_used = measures_not_used(&bench);
    assert!(!not_used.contains(&"p50") && !not_used.contains(&"p99_over_p50"));
    assert!(not_used.contains(&"memory_peak"));
    let text = serde_json::to_string(&record).unwrap();
    for marker in [":\\\\", ":/", "/Users/", "\\\\Users\\\\", "Fixtures"] {
        assert!(!text.contains(marker), "the record holds a path ({marker})");
    }
    let _: Value = record;
}
