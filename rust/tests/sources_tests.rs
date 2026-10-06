//! Gzipped profiler traces, runtime logs, and run folders, the other sources the .NET
//! connector read.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use scalarscope::open::{is_runtime_log, open_log, open_path, Side};

fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "scalarscope-sources-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn trace_json() -> String {
    let events: Vec<String> = (0..12)
        .map(|index| {
            let duration = if index < 3 { 9000 - index * 2000 } else { 2000 };
            format!(r#"{{"name":"ProfilerStep#{index}","ph":"X","ts":{},"dur":{duration}}}"#, index * 10_000)
        })
        .collect();
    format!(r#"{{"traceEvents":[{}]}}"#, events.join(","))
}

fn inference(side: Side) -> scalarscope::open::InferenceRun {
    match side {
        Side::Inference(run) => run,
        Side::Training(_) => panic!("inference"),
    }
}

#[test]
fn a_gzipped_profiler_trace_opens_like_the_plain_one() {
    let dir = scratch("gz");
    let plain = dir.join("trace.json");
    fs::write(&plain, trace_json()).unwrap();
    let packed = dir.join("trace.json.gz");
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(trace_json().as_bytes()).unwrap();
    fs::write(&packed, encoder.finish().unwrap()).unwrap();
    let a = inference(open_path(&plain).unwrap().side);
    let b = inference(open_path(&packed).unwrap().side);
    assert_eq!(a.latency_ms, b.latency_ms);
    assert_eq!(b.label, "trace");
    let broken = dir.join("broken.json.gz");
    fs::write(&broken, b"not gzip").unwrap();
    assert!(open_path(&broken).unwrap_err().contains("gzip"));
}

#[test]
fn a_runtime_log_reads_step_latency_throughput_and_memory() {
    let text = "TF-TRT engine built, batch size 8\n\
                step 0 latency_ms: 12.5 throughput=640.2 memory 104857600\n\
                step 1 latency_ms: 11.0 throughput=727.3 memory 104857600\n\
                warning: something unrelated\n\
                latency = 10.5ms throughput: 761.9 memory: 209715200\n";
    assert!(is_runtime_log(text));
    let run = inference(open_log(text, "runtime").unwrap());
    assert_eq!(run.steps, vec![0, 1, 2]);
    assert_eq!(run.latency_ms, vec![12.5, 11.0, 10.5]);
    assert_eq!(run.throughput, vec![640.2, 727.3, 761.9]);
    assert_eq!(run.memory_mb, vec![100.0, 100.0, 200.0]);
    // The .NET patterns found no number after `latency_ms` or `memory_mb`; these read.
    let named = inference(open_log("TensorRT\nlatency_ms: 9.5 memory_mb: 512\n", "units").unwrap());
    assert_eq!((named.latency_ms[0], named.memory_mb[0]), (9.5, 512.0));
    assert!(!is_runtime_log("just some text\nwith nothing in it"));
    assert!(open_log("batch size 8\nno numbers here", "empty").is_err());
}

#[test]
fn a_log_line_without_latency_still_moves_the_step() {
    let run = inference(open_log("TensorRT\nstep 4 throughput 10\nlatency 3\nstep 9 latency 4\n", "gap").unwrap());
    assert_eq!(run.steps, vec![5, 9]);
    assert!(run.throughput.is_empty(), "throughput is only kept when every latency line has it");
}

#[test]
fn a_folder_opens_its_best_source_and_takes_warmup_from_config() {
    let dir = scratch("folder");
    fs::create_dir_all(dir.join("profiler")).unwrap();
    fs::write(dir.join("runtime.log"), "TensorRT\nstep 0 latency_ms: 50\n").unwrap();
    let rows: String = (0..40).map(|step| format!("{step},{}\n", if step < 6 { 40.0 - step as f64 } else { 20.0 })).collect();
    fs::write(dir.join("benchmark.csv"), format!("step,latency_ms\n{rows}")).unwrap();
    // The CSV outranks the log.
    let loaded = open_path(&dir).unwrap();
    let run = inference(loaded.side);
    assert_eq!(run.latency_ms.len(), 40);
    assert_eq!(loaded.path, dir.display().to_string());
    assert_eq!(run.label, dir.file_name().unwrap().to_str().unwrap());

    // A profiler trace outranks the CSV.
    fs::write(dir.join("profiler").join("trace.json"), trace_json()).unwrap();
    assert_eq!(inference(open_path(&dir).unwrap().side).latency_ms.len(), 12);

    // config.json sets where warmup ends.
    fs::remove_file(dir.join("profiler").join("trace.json")).unwrap();
    fs::write(dir.join("config.json"), r#"{"warmup_steps": 10}"#).unwrap();
    let run = inference(open_path(&dir).unwrap().side);
    assert_eq!(run.warmup_end, Some(10));
    assert!(run.steady_step.is_some_and(|step| step >= 10));
}

#[test]
fn an_empty_folder_says_what_it_looked_for() {
    let dir = scratch("empty");
    fs::write(dir.join("notes.txt"), "nothing").unwrap();
    assert!(open_path(&dir).unwrap_err().contains("no profiler trace"));
}

fn csv(level: f64) -> String {
    let rows: String = (0..60).map(|step| format!("{step},{}\n", level + (step % 3) as f64 * 0.1)).collect();
    format!("step,latency_ms\n{rows}")
}

#[test]
fn several_picked_files_are_repeats_of_one_side() {
    let dir = scratch("several");
    let paths: Vec<PathBuf> = (0..3)
        .map(|index| {
            let path = dir.join(format!("run-{index}.csv"));
            fs::write(&path, csv(10.0 + index as f64)).unwrap();
            path
        })
        .collect();
    let run = inference(scalarscope::open::open_paths(&paths).unwrap().side);
    assert_eq!(run.label, "run-0");
    assert_eq!(run.replicates.len(), 2);
    assert_eq!(run.runs().len(), 3);
    assert!(scalarscope::open::open_paths(&[]).is_err());
    let history = dir.join("run_history.json");
    fs::write(&history, r#"[{"run_id":"r","status":"completed","loss_history":[1.0,0.5]}]"#).unwrap();
    assert!(scalarscope::open::open_paths(&[paths[0].clone(), history]).unwrap_err().contains("inference run"));
}

#[test]
fn a_folder_of_run_folders_is_a_set_of_repeats() {
    let dir = scratch("set");
    for index in 0..3 {
        let run = dir.join(format!("seed-{index}"));
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("benchmark.csv"), csv(10.0 + index as f64)).unwrap();
    }
    let run = inference(open_path(&dir).unwrap().side);
    assert_eq!(run.replicates.len(), 2);
    assert_eq!(run.label, dir.file_name().unwrap().to_str().unwrap());
    // A file at the top keeps the 2.0 rule: the folder is one run.
    fs::write(dir.join("benchmark.csv"), csv(5.0)).unwrap();
    assert!(inference(open_path(&dir).unwrap().side).replicates.is_empty());
}
