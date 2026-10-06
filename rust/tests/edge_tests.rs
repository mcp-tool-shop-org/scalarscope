//! Edges the review still has to refuse, withhold, or store exactly as written.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use scalarscope::bundle::{
    bundle_hash, document_from_pair, open_bytes, open_file, pack, seal, sha256_hex, stored_pair, unpack, utc_now, verify, write_file,
    BundleDocument, StoredBand, StoredReview, StoredSeries, StoredTraining,
};
use scalarscope::history::{self, LogEntry};
use scalarscope::milestones::{detect_steady_start, detect_warmup_end};
use scalarscope::open::{open_path, open_text, InferenceRun, Side, TrainingEntry};
use scalarscope::prefs::{self, series_palette};
use scalarscope::readings::{deviation_band, percentile, steady_index};
use scalarscope::review::{pair, Finding, Pair};

const STAMP: &str = "2026-10-03T12:00:00Z";

fn directory() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
    let path = std::env::temp_dir().join(format!("scalarscope-edge-{}-{nanos}-{id}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn inference(label: &str, latency: Vec<f64>, steady: Option<i64>, warmup: Option<i64>, throughput: Vec<f64>) -> Side {
    Side::Inference(InferenceRun {
        label: label.to_string(),
        steps: (0..latency.len() as i64).collect(),
        latency_ms: latency,
        throughput,
        warmup_end: warmup,
        steady_step: steady,
        memory_mb: Vec::new(),
        trace: None,
    })
}

fn training(run_id: &str, loss: Vec<f64>) -> Side {
    Side::Training(TrainingEntry {
        run_id: run_id.to_string(),
        model_name: String::new(),
        loss,
        final_loss: None,
        train_steps: None,
        held_out_loss: None,
        perplexity: None,
        eval_n: None,
        task_metrics: Vec::new(),
        metric_ci: Vec::new(),
        entry_count: 1,
        selection: "the last completed row in this file".to_string(),
    })
}

fn findings_archive(deltas: &str) -> Vec<u8> {
    let bytes = deltas.as_bytes().to_vec();
    let mut hashes = BTreeMap::new();
    hashes.insert("findings/deltas.json".to_string(), sha256_hex(&bytes));
    let digest = bundle_hash(&hashes);
    let integrity = format!(
        r#"{{"fileHashes":{{"findings/deltas.json":"{}"}},"bundleHash":"{digest}","computedAt":"{STAMP}"}}"#,
        hashes["findings/deltas.json"]
    );
    pack(&[
        ("findings/deltas.json".to_string(), bytes),
        ("integrity.json".to_string(), integrity.into_bytes()),
    ])
    .unwrap()
}

#[test]
fn open_path_uses_the_file_stem_and_refuses_a_missing_file() {
    let dir = directory();
    let path = dir.join("baseline.csv");
    fs::write(&path, "step,latency_ms\n0,10\n1,11\n").unwrap();
    let loaded = open_path(&path).unwrap();
    assert_eq!(loaded.path, path.display().to_string());
    let Side::Inference(run) = loaded.side else { panic!("csv") };
    assert_eq!(run.label, "baseline");
    assert!(open_path(Path::new(r"D:\no\such\scalarscope.csv")).unwrap_err().contains("Could not read"));
    // A folder is a source too: it opens its best file under the folder's name.
    let folder = open_path(&dir).unwrap();
    let Side::Inference(run) = folder.side else { panic!("csv") };
    assert_eq!(run.latency_ms, vec![10.0, 11.0]);
    assert_eq!(run.label, dir.file_name().unwrap().to_str().unwrap());
}

#[test]
fn a_geometry_file_and_a_json_that_is_not_a_run_are_refused() {
    let geometry = r#"{"trajectory":{"timesteps":[0]}}"#;
    assert!(open_text(geometry, "path").unwrap_err().contains("geometry"));
    assert!(open_text(r#"{"hello":1}"#, "notes").unwrap_err().contains("not a profiler trace"));
    assert!(open_text("[42]", "number").unwrap_err().contains("not a profiler trace"));
    assert!(open_text("42", "number").unwrap_err().contains("no latency column"));
    assert!(open_text("{}", "empty").unwrap_err().contains("not a profiler trace"));
}

#[test]
fn a_phase_that_is_not_text_does_not_count_as_a_complete_step() {
    let trace = r#"{"traceEvents":[
        {"name":"ProfilerStep#0","ph":1,"dur":5000},
        {"name":"inference","ph":"X","dur":4000}
    ]}"#;
    let Side::Inference(run) = open_text(trace, "capture").unwrap() else { panic!("named event") };
    assert_eq!(run.latency_ms, vec![4.0]);
}

#[test]
fn a_benchmark_keeps_items_per_sec_and_skips_a_row_without_latency() {
    let text = r#"{"results":[
        {"name":"warmup"},
        {"latency_ms":1.5,"items_per_sec":3.0},
        {"latency":2.5,"throughput":4.0}
    ]}"#;
    let Side::Inference(run) = open_text(text, "bench").unwrap() else { panic!("benchmark") };
    assert_eq!(run.latency_ms, vec![1.5, 2.5]);
    assert_eq!(run.throughput, vec![3.0, 4.0]);
    assert!(open_text(r#"{"iterations":[{"name":"x"}]}"#, "bench").unwrap_err().contains("no numeric latency"));
    assert!(open_text(r#"{"benchmarks":[]}"#, "bench").unwrap_err().contains("no numeric latency"));
}

#[test]
fn a_csv_skips_a_short_row_and_refuses_a_non_finite_or_empty_series() {
    let skipped = "step,latency_ms\n0\n1,2.5\n";
    let Side::Inference(run) = open_text(skipped, "rows").unwrap() else { panic!("csv") };
    assert_eq!(run.latency_ms, vec![2.5]);
    assert!(open_text("latency_ms\nNaN\n", "bad").unwrap_err().contains("no numeric latency"));
    assert!(open_text("latency_ms\ninf\n", "bad").unwrap_err().contains("no numeric latency"));
    assert!(open_text("latency_ms\n", "empty").unwrap_err().contains("no numeric latency"));
    assert!(open_text("", "empty").unwrap_err().contains("no header"));
}

#[test]
fn a_run_history_uses_the_last_row_with_a_loss_when_none_completed() {
    let text = r#"[
        {"run_id":"running","status":"running","loss_history":[3.0]},
        {"run_id":"failed","status":"failed","final_loss":1.5}
    ]"#;
    let Side::Training(entry) = open_text(text, "history").unwrap() else { panic!("training") };
    assert_eq!(entry.run_id, "failed");
    assert!(entry.loss.is_empty());
    assert_eq!(entry.final_loss, Some(1.5));
    assert!(entry.selection.contains("last row"));

    let object = r#"{"run_id":"","loss_history":[1.0,0.5],"eval":{"task_metrics":{"acc":"no","ok":0.5},"metric_ci":{"ok":1.0,"bad":null}}}"#;
    let Side::Training(entry) = open_text(object, "label").unwrap() else { panic!("object") };
    assert_eq!(entry.run_id, "label");
    assert_eq!(entry.task_metrics, vec![("ok".to_string(), 0.5)]);
    assert_eq!(entry.metric_ci, vec![("ok".to_string(), 1.0)]);

    assert!(open_text(r#"[{"run_id":"x"}]"#, "none").unwrap_err().contains("not a profiler trace"));
    assert!(open_text(r#"[{"loss_history":[]}]"#, "empty").unwrap_err().contains("no loss"));
    assert!(open_text("[1, 2]", "numbers").unwrap_err().contains("not a profiler trace"));
}

#[test]
fn percentiles_and_bands_leave_a_gap_when_the_window_is_too_small() {
    assert_eq!(percentile(&[], 0.5), None);
    assert_eq!(percentile(&[4.0, 9.0], 0.0), Some(4.0));
    assert_eq!(percentile(&[4.0, 9.0], 2.0), Some(9.0));
    assert!(deviation_band(&[Some(1.0)]).iter().all(Option::is_none));
    assert!(deviation_band(&[None, None]).iter().all(Option::is_none));
    assert_eq!(steady_index(&[0, 1], 0, 0, Some(1)), None);
    assert_eq!(steady_index(&[0, 1], 0, 2, None), None);
}

#[test]
fn warmup_falls_back_when_the_series_keeps_falling_and_steady_state_stays_quiet() {
    assert_eq!(detect_warmup_end(&[1.0; 9]), None);
    let mut sparse = vec![1.0; 12];
    sparse[0] = f64::NAN;
    sparse[1] = f64::NAN;
    sparse[2] = f64::NAN;
    assert_eq!(detect_warmup_end(&sparse), None);

    let mut falling = Vec::new();
    let mut value = 1000.0;
    for _ in 0..20 {
        falling.push(value);
        value *= 0.9;
    }
    assert_eq!(detect_warmup_end(&falling), Some(5));
    assert_eq!(detect_steady_start(&[1.0; 4], 4), None);
    assert_eq!(detect_steady_start(&[1.0; 12], 5), None);
    let mut noisy = vec![10.0; 20];
    noisy[10] = 100.0;
    assert_eq!(detect_steady_start(&noisy, 0), None);
    assert_eq!(detect_steady_start(&[10.0; 20], 0), Some(0));
}

#[test]
fn a_step_list_that_does_not_match_the_samples_is_not_a_stabilization_time() {
    let mut left = InferenceRun {
        label: "left".to_string(),
        steps: vec![1],
        latency_ms: vec![10.0, 11.0, 12.0],
        throughput: vec![1.0],
        warmup_end: None,
        steady_step: None,
        memory_mb: Vec::new(),
        trace: None,
    };
    let right = InferenceRun {
        label: "right".to_string(),
        steps: vec![],
        latency_ms: vec![8.0, 9.0],
        throughput: Vec::new(),
        warmup_end: None,
        steady_step: None,
        memory_mb: Vec::new(),
        trace: None,
    };
    let Pair::Inference(review) = pair(&Side::Inference(left.clone()), &Side::Inference(right)).unwrap() else { panic!("inference") };
    // Two and three samples are too short to tell where a run settles.
    assert!(review.verdict.contains("ΔTc is withheld") && review.verdict.contains("too short to tell"), "{}", review.verdict);
    assert!(!review.verdict.contains("Stabilizes"));
    assert!(review.left_throughput.is_empty());

    left.steady_step = Some(2);
    left.steps = vec![0, 1, 2];
    left.throughput.clear();
    let right = InferenceRun {
        label: "right".to_string(),
        steps: vec![0, 1, 2],
        latency_ms: vec![8.0, 9.0, 10.0],
        throughput: Vec::new(),
        warmup_end: None,
        steady_step: Some(2),
        memory_mb: Vec::new(),
        trace: None,
    };
    let Pair::Inference(review) = pair(&Side::Inference(left), &Side::Inference(right)).unwrap() else { panic!("inference") };
    assert!(!review.fired.iter().any(|symbol| symbol == "ΔO"));
}

#[test]
fn alignment_uses_warmup_when_steady_state_is_missing_and_the_first_step_otherwise() {
    let left = inference("left", vec![10.0; 8], None, Some(2), vec![1.0; 8]);
    let right = inference("right", vec![12.0; 8], None, Some(4), vec![2.0; 8]);
    let Pair::Inference(review) = pair(&left, &right).unwrap() else { panic!("inference") };
    assert!(review.caption.contains("warmup milestone"));
    assert!(!review.caption.contains("vertical line"));
    assert_eq!(review.left_throughput.len(), review.right_throughput.len());

    let left = inference("left", vec![10.0, 11.0], None, None, Vec::new());
    let right = inference("right", vec![12.0, 13.0], None, None, Vec::new());
    let Pair::Inference(review) = pair(&left, &right).unwrap() else { panic!("inference") };
    assert!(review.caption.contains("first step"));

    let left = inference("left", vec![10.0; 8], Some(1), Some(0), Vec::new());
    let right = inference("right", vec![12.0; 8], Some(4), Some(0), Vec::new());
    let Pair::Inference(review) = pair(&left, &right).unwrap() else { panic!("inference") };
    // Steps set by hand are not stated in a file, and eight samples are too short to tell.
    assert!(review.verdict.contains("ΔTc is withheld"), "{}", review.verdict);
    assert!(review.caption.contains("vertical line"));

    let left = inference("left", vec![10.0; 8], Some(6), None, Vec::new());
    let right = inference("right", vec![12.0; 8], Some(1), None, Vec::new());
    let Pair::Inference(review) = pair(&left, &right).unwrap() else { panic!("inference") };
    assert!(review.verdict.contains("ΔTc is withheld"), "{}", review.verdict);
}

#[test]
fn an_empty_pair_does_not_invent_throughput() {
    let left = inference("left", Vec::new(), None, None, Vec::new());
    let right = inference("right", Vec::new(), None, None, Vec::new());
    let Pair::Inference(review) = pair(&left, &right).unwrap() else { panic!("inference") };
    assert!(review.left_throughput.is_empty());
    assert!(review.right_throughput.is_empty());
}

#[test]
fn the_seal_refuses_a_bad_stamp_a_non_finite_delta_and_keeps_a_negative_magnitude_positive() {
    let built = pair(
        &inference("left", vec![10.0; 8], None, None, Vec::new()),
        &inference("right", vec![12.0; 8], None, None, Vec::new()),
    )
    .unwrap();
    let document = document_from_pair(&built);
    assert!(seal(&document, "").unwrap_err().contains("timestamp"));
    assert!(seal(&document, "2026-10-03T12:00:00Z\n").unwrap_err().contains("timestamp"));
    assert!(seal(&document, "2026-10-03T12:00:00Zé").unwrap_err().contains("timestamp"));

    let mut nan = document.clone();
    nan.review.findings.push(Finding {
        symbol: "ΔF".to_string(),
        id: "FailurePresence".to_string(),
        name: "Failure Events".to_string(),
        kind: "Event".to_string(),
        sentence: "Introduced 1 new runtime anomalies".to_string(),
        left: 0.0,
        right: 1.0,
        delta: f64::NAN,
        units: "count".to_string(),
    });
    let sealed = seal(&nan, STAMP).unwrap();
    let nan_text = String::from_utf8(unpack(&sealed.bytes).unwrap().into_iter().find(|(path, _)| path == "findings/deltas.json").unwrap().1).unwrap();
    assert!(nan_text.contains("\"magnitude\": 0.0") || nan_text.contains("\"magnitude\": 0"), "{nan_text}");
    assert!(!nan_text.contains("NaN"), "{nan_text}");

    let mut negative = document.clone();
    negative.review.findings.push(Finding {
        symbol: "ΔO".to_string(),
        id: "StabilityOscillation".to_string(),
        name: "Stability".to_string(),
        kind: "Behavior".to_string(),
        sentence: "Reduced runtime variability".to_string(),
        left: 4.0,
        right: 1.0,
        delta: -3.0,
        units: "ms".to_string(),
    });
    let sealed = seal(&negative, STAMP).unwrap();
    let entries = unpack(&sealed.bytes).unwrap();
    let deltas = entries.iter().find(|(path, _)| path == "findings/deltas.json").unwrap().1.clone();
    let text = String::from_utf8(deltas).unwrap();
    assert!(text.contains("\"magnitude\": 3.0") || text.contains("\"magnitude\": 3"), "{text}");
    assert!(text.contains("\"delta\": -3.0") || text.contains("\"delta\": -3"), "{text}");
}

#[test]
fn stored_drawings_come_back_and_an_unknown_kind_does_not_become_a_chart() {
    let inference_pair = pair(
        &inference("desk/baseline", vec![10.0, 11.0, 12.0, 13.0], Some(1), None, vec![1.0, 1.0, 1.0, 1.0]),
        &inference("other/candidate", vec![8.0, 9.0, 10.0, 11.0], Some(1), None, vec![2.0, 2.0, 2.0, 2.0]),
    )
    .unwrap();
    let sealed = seal(&document_from_pair(&inference_pair), STAMP).unwrap();
    let opened = open_bytes(&sealed.bytes).unwrap();
    let Pair::Inference(restored) = stored_pair(&opened.review).unwrap() else { panic!("inference") };
    assert!(!restored.left.is_empty());
    assert_eq!(restored.left_cdf.len(), restored.left.iter().flatten().count());

    let training_pair = pair(&training(r"folder\alpha", vec![1.0, 0.5]), &training("/", vec![0.4])).unwrap();
    let sealed = seal(&document_from_pair(&training_pair), STAMP).unwrap();
    let opened = open_bytes(&sealed.bytes).unwrap();
    let Pair::Training(restored) = stored_pair(&opened.review).unwrap() else { panic!("training") };
    assert_eq!(restored.left.run_id, "alpha");
    assert_eq!(restored.right.run_id, "run");

    let mut bare = opened.review.clone();
    bare.kind = "findings".to_string();
    assert!(stored_pair(&bare).is_none());
    bare.kind = "inference".to_string();
    bare.inference = None;
    assert!(stored_pair(&bare).is_none());
    bare.kind = "training".to_string();
    bare.training = None;
    assert!(stored_pair(&bare).is_none());

    let hand = StoredReview {
        kind: "inference".to_string(),
        notices: Vec::new(),
        headline: String::new(),
        verdict: String::new(),
        fired: Vec::new(),
        caption: String::new(),
        left_text: String::new(),
        right_text: String::new(),
        findings: Vec::new(),
        inference: Some(StoredSeries {
            left_label: "a".to_string(),
            right_label: "b".to_string(),
            signal: "latency_ms".to_string(),
            unit: "ms".to_string(),
            left: vec![Some(1.0), None],
            right: vec![Some(2.0)],
            left_band: vec![Some(StoredBand { low: 0.0, high: 2.0 }), None],
            right_band: vec![None],
            left_marks: vec![0],
            right_marks: Vec::new(),
            left_steady: None,
            right_steady: Some(0),
            left_throughput: Vec::new(),
            right_throughput: vec![1.0],
            left_memory: Vec::new(),
            right_memory: Vec::new(),
            left_cdf: vec![[1.0, 1.0]],
            right_cdf: Vec::new(),
            left_p50: Some(1.0),
            left_p95: None,
            left_p99: None,
            right_p50: None,
            right_p95: None,
            right_p99: None,
        }),
        training: Some(StoredTraining {
            left_run_id: "unused".to_string(),
            right_run_id: "unused".to_string(),
            left_loss: vec![1.0],
            right_loss: Vec::new(),
        }),
    };
    let Pair::Inference(restored) = stored_pair(&hand).unwrap() else { panic!("inference") };
    assert!(restored.left_band[1].is_none());
    assert_eq!(restored.left_cdf, vec![(1.0, 1.0)]);
}

#[test]
fn a_findings_bundle_keeps_only_the_sentences_that_were_stored() {
    let mixed = r#"[
        {"id":"FailurePresence","status":"Absent","summarySentence":"skip me"},
        {"id":"ConvergenceTiming","status":"Present","summarySentence":""},
        {"id":"StabilityOscillation","status":"Present","explanation":"spread changed"},
        {"id":"NotADelta","status":"Present","summarySentence":"plain sentence"},
        {"id":"StructuralEmergence","status":"Present","summarySentence":"a stored emergence"},
        {"id":"EvaluatorAlignment","status":"Present","summarySentence":"a stored alignment"}
    ]"#;
    let opened = open_bytes(&findings_archive(mixed)).unwrap();
    assert!(stored_pair(&opened.review).is_none());
    assert!(opened.review.verdict.contains("ΔO spread changed"));
    assert!(opened.review.verdict.contains("plain sentence"));
    assert!(!opened.review.verdict.contains("ΔF"));
    assert!(opened.review.verdict.contains("ΔTd"));
    assert!(opened.review.verdict.contains("ΔĀ"));
    assert_eq!(
        opened.review.fired,
        vec!["ΔO".to_string(), "ΔTd".to_string(), "ΔĀ".to_string()]
    );

    let empty = r#"[{"id":"FailurePresence","status":"Absent","summarySentence":"no"}]"#;
    let opened = open_bytes(&findings_archive(empty)).unwrap();
    assert_eq!(opened.review.verdict, "No stored finding was present.");

    let readme = pack(&[("README.md".to_string(), b"hi".to_vec())]).unwrap();
    let verification = verify(&readme).unwrap();
    assert!(!verification.valid);
    assert!(verification.codes.iter().any(|code| code == "MissingIntegrity"));
    assert!(open_bytes(&readme).unwrap_err().contains("MissingIntegrity"));

    let broken = pack(&[
        ("README.md".to_string(), b"hi".to_vec()),
        ("integrity.json".to_string(), b"not json".to_vec()),
    ])
    .unwrap();
    let verification = verify(&broken).unwrap();
    assert!(verification.codes.iter().any(|code| code == "InvalidIntegrity"));

    let bad_path = format!(
        r#"{{"fileHashes":{{"../secret":"abc","integrity.json":"def","./README.md":"{}"}},"bundleHash":"nope","computedAt":"{STAMP}"}}"#,
        sha256_hex(b"hi")
    );
    let bytes = pack(&[
        ("README.md".to_string(), b"hi".to_vec()),
        ("integrity.json".to_string(), bad_path.into_bytes()),
    ])
    .unwrap();
    let verification = verify(&bytes).unwrap();
    assert!(verification.codes.iter().any(|code| code == "BadPath"));
    assert!(verification.codes.iter().any(|code| code == "SealedFileListed"));
}

#[test]
fn pack_strips_a_dot_slash_and_refuses_a_path_that_leaves_the_archive() {
    let bytes = pack(&[("././notes.txt".to_string(), b"hi".to_vec())]).unwrap();
    let entries = unpack(&bytes).unwrap();
    assert_eq!(entries[0].0, "notes.txt");
    assert!(pack(&[("../secret.txt".to_string(), b"hi".to_vec())]).unwrap_err().contains("cannot leave"));
    assert!(pack(&[("/secret.txt".to_string(), b"hi".to_vec())]).unwrap_err().contains("cannot leave"));
    assert!(pack(&[("folder/".to_string(), b"hi".to_vec())]).unwrap_err().contains("name a file"));
    assert!(pack(&[("".to_string(), b"hi".to_vec())]).unwrap_err().contains("name a file"));
    assert!(pack(&[
        ("notes.txt".to_string(), b"a".to_vec()),
        ("notes.txt".to_string(), b"b".to_vec()),
    ])
    .unwrap_err()
    .contains("Duplicate"));
    assert!(unpack(b"not a zip").unwrap_err().contains("not a bundle"));
}

#[test]
fn write_file_and_open_file_round_trip_and_report_a_missing_path() {
    let dir = directory();
    let built = pair(
        &inference("left", vec![10.0; 4], None, None, Vec::new()),
        &inference("right", vec![12.0; 4], None, None, Vec::new()),
    )
    .unwrap();
    let sealed = seal(&document_from_pair(&built), STAMP).unwrap();
    let path = dir.join("review.scbundle");
    write_file(&path, &sealed).unwrap();
    let opened = open_file(&path).unwrap();
    assert_eq!(opened.bundle_hash, sealed.bundle_hash);
    assert!(write_file(&dir, &sealed).unwrap_err().contains("Could not write"));
    assert!(open_file(&dir.join("missing.scbundle")).unwrap_err().contains("Could not read"));
    let now = utc_now();
    assert!(now.ends_with('Z'));
    assert_eq!(now.len(), 20);

    let empty = BundleDocument {
        review: StoredReview {
            kind: "findings".to_string(),
            notices: Vec::new(),
            headline: String::new(),
            verdict: String::new(),
            fired: Vec::new(),
            caption: String::new(),
            left_text: String::new(),
            right_text: String::new(),
            findings: Vec::new(),
            inference: None,
            training: None,
        },
    };
    let sealed = seal(&empty, STAMP).unwrap();
    let opened = open_bytes(&sealed.bytes).unwrap();
    assert_eq!(opened.review.kind, "findings");
    assert!(opened.review.verdict.is_empty());
    assert!(stored_pair(&opened.review).is_none());
    let summary = unpack(&sealed.bytes).unwrap().into_iter().find(|(path, _)| path == "findings/summary.md").unwrap().1;
    let summary = String::from_utf8(summary).unwrap();
    assert!(summary.contains("No finding was stored on this page."));
}

#[test]
fn history_fills_an_empty_kind_updates_the_same_bundle_and_matches_a_name_only_sitting() {
    let dir = directory();
    assert!(history::record(named("Alpha", "Beta"), Path::new("")).unwrap_err().contains("missing"));
    assert!(history::attach_bundle("  ", None, &dir).unwrap_err().contains("missing"));
    history::clear(&dir).unwrap();
    assert!(!history::file_path(&dir).exists());

    let mut entry = named("Alpha", "Beta");
    entry.kind.clear();
    let saved = history::record(entry, &dir).unwrap();
    assert_eq!(saved.kind, "compare");

    let again = history::record(named("Alpha", "Beta"), &dir).unwrap();
    assert_eq!(again.id, saved.id);
    assert_eq!(history::read(&dir).len(), 1);

    history::clear(&dir).unwrap();
    let path = r"D:\reviews\one.scbundle";
    history::attach_bundle(path, Some("aaaaaaaaaaaaaaaa"), &dir).unwrap();
    let updated = history::attach_bundle(path, Some("bbbbbbbbbbbbbbbb"), &dir).unwrap();
    assert_eq!(updated.bundle_hash.as_deref(), Some("bbbbbbbbbbbbbbbb"));
    assert_eq!(history::read(&dir).len(), 1);
    assert_eq!(updated.kind, "bundle");

    let json = r#"[{"leftName":"solo","rightName":"","deltasFired":[]}]"#;
    fs::write(history::file_path(&dir), json).unwrap();
    let entries = history::read(&dir);
    assert_eq!(entries[0].kind, "compare");
    assert_eq!(entries[0].title(), "solo");
    assert_eq!(entries[0].subtitle(), "No deltas fired");

    let blocked = dir.join("blocked");
    fs::write(&blocked, "x").unwrap();
    assert!(history::record(named("Alpha", "Beta"), &blocked).unwrap_err().contains("Could not"));
}

#[test]
fn preferences_skip_empty_paths_unreadable_gallery_files_and_non_string_fields() {
    assert!(prefs::remember_file(Path::new(""), "trace.csv", "trace").unwrap_err().contains("missing"));
    let dir = directory();
    assert!(prefs::remember_file(&dir, "  ", "trace").unwrap_err().contains("missing"));
    prefs::remember_file(&dir, r"D:\reviews\trace.csv", "  ").unwrap();
    let saved = prefs::read(&dir);
    assert_eq!(saved.recent[0].name, "trace");

    fs::write(dir.join(prefs::FILE_NAME), "[1, 2]").unwrap();
    assert!(prefs::remember_file(&dir, "trace.csv", "trace").unwrap_err().contains("left unchanged"));
    assert_eq!(fs::read_to_string(dir.join(prefs::FILE_NAME)).unwrap(), "[1, 2]");

    fs::write(
        dir.join(prefs::FILE_NAME),
        r#"{"TextScale":"big","ColorVisionMode":"no","HighContrastMode":"no","RecentFilesLimit":80,"RecentFiles":[{"Path":"  ","Name":"skip"},{"Path":"D:\\reviews\\plain.csv"},{"Path":"D:\\reviews\\named.csv","Name":1,"LastOpened":1}]}"#,
    )
    .unwrap();
    let read = prefs::read(&dir);
    assert!((read.text_scale - 1.0).abs() < 1e-6);
    assert_eq!(read.color_vision, 0);
    assert!(!read.high_contrast);
    assert_eq!(read.recent_limit, 10);
    assert_eq!(read.recent.len(), 2);
    assert_eq!(read.recent[0].name, "plain");

    assert!(prefs::saved_views(&dir).is_empty());
    assert!(prefs::saved_views(&dir.join(prefs::FILE_NAME)).is_empty());
    let gallery = dir.join("gallery");
    fs::create_dir_all(&gallery).unwrap();
    fs::write(gallery.join("notes.txt"), "skip").unwrap();
    fs::create_dir_all(gallery.join("locked.json")).unwrap();
    fs::write(gallery.join("array.json"), "[1]").unwrap();
    fs::write(
        gallery.join("view.json"),
        r#"{"title":1,"createdAt":1,"state":{"dataSource":"  "}}"#,
    )
    .unwrap();
    let views = prefs::saved_views(&dir);
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Saved view");
    assert!(views[0].source.is_none());
    assert!(series_palette(0, false).label.is_none());
}

fn named(left: &str, right: &str) -> LogEntry {
    LogEntry {
        id: String::new(),
        finished_at: String::new(),
        left_name: left.to_string(),
        right_name: right.to_string(),
        left_path: None,
        right_path: None,
        left_run_id: None,
        right_run_id: None,
        bundle_path: None,
        bundle_hash: None,
        alignment: "latency".to_string(),
        deltas_fired: Vec::new(),
        kind: "compare".to_string(),
    }
}
