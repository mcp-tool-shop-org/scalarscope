//! The phase-5 exit test's live session: the workbench with a real tool-calling model, on the
//! 2.0 golden pair and on the simulated knob folder. Not run in CI.
//!
//! cargo run --example live_workbench -- <ollama port> <output folder>
//!
//! The port is the local Ollama (11434) or an offrig tunnel to a rented pod (11435). Each
//! session's record (model, digest, every call and the program's answer, the states the program
//! set, the model's note) is written to the output folder as JSON, with no paths.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use scalarscope::open::{open_path, InferenceRun, Side};
use scalarscope::stats::AnomalyRule;
use scalarscope::workbench::{board, session_record};
use workbench::{start_bench_on, BenchReply, Workbench};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/Fixtures")
}

fn inference(path: &Path) -> InferenceRun {
    match open_path(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())).side {
        Side::Inference(run) => run,
        _ => panic!("not an inference run"),
    }
}

fn session(port: u16, name: &str, runs: Vec<InferenceRun>, your_call: &str, out: &Path) {
    let bench = Workbench::new(board(runs, AnomalyRule::Mad), Vec::new(), Vec::new(), "2026-10-06");
    let started = Instant::now();
    let reply = start_bench_on(port, bench).recv_timeout(Duration::from_secs(1800)).expect("the session did not answer within 30 minutes");
    match reply {
        BenchReply::Done { model, digest, bench, stopped } => {
            let mut record = session_record(&model, digest.as_deref(), &stopped, your_call, &bench);
            record["session"] = serde_json::json!(name);
            record["seconds"] = serde_json::json!(started.elapsed().as_secs());
            let file = out.join(format!("workbench-{name}.json"));
            std::fs::write(&file, serde_json::to_string_pretty(&record).unwrap() + "\n").unwrap();
            println!("{name}: {model} {digest:?}, {stopped} ({} s), {} calls", started.elapsed().as_secs(), bench.steps.len());
            for step in &bench.steps {
                println!("  > {} {}\n    {}", step.tool, step.args, step.result.replace('\n', "\n    "));
            }
            println!("  proposed: {}", record["proposed"]);
            println!("  note: {:?} (dropped: {})", bench.note, bench.note_dropped);
        }
        BenchReply::Absent(text) => panic!("{name}: no session: {text}"),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let port: u16 = args.next().and_then(|port| port.parse().ok()).expect("usage: live_workbench <port> <output folder>");
    let out = PathBuf::from(args.next().expect("usage: live_workbench <port> <output folder>"));
    std::fs::create_dir_all(&out).unwrap();

    let golden = fixtures().join("InferenceOptimization");
    session(
        port,
        "golden-pair",
        vec![inference(&golden.join("baseline_tfrt_runtrace.json")), inference(&golden.join("optimized_tfrt_runtrace.json"))],
        "These two files record no knobs, so I expect nothing about a knob can be settled here.",
        &out,
    );

    let mut knob_runs = Vec::new();
    for batch in [1, 4, 8] {
        for seed in 1..=3 {
            knob_runs.push(inference(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/workbench/batch{batch}_seed{seed}_runtrace.json"))));
        }
    }
    session(port, "knob-folder", knob_runs, "I expect a larger batch to raise p50 and to make the tail heavier.", &out);
}
