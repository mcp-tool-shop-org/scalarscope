//! The window, painted without a native file dialog.
//! A queued path is a chosen file. A queued cancel leaves the review alone.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use eframe::egui::{self, Color32};
use eframe::App;

use crate::bundle::{self, OpenedBundle, StoredReview};
use crate::history::{self, LogEntry};
use crate::open::{open_text, Loaded, Side};
use crate::prefs::{self, RecentFile, SavedView};
use crate::readings::Band;
use crate::review;

use super::{queue_pick, queue_save, Paint, ScalarScopeApp};

fn directory() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
    let path = std::env::temp_dir().join(format!("scalarscope-ui-{}-{nanos}-{id}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn blank(history: Option<PathBuf>) -> ScalarScopeApp {
    ScalarScopeApp {
        left: None,
        right: None,
        opened: None,
        note: String::new(),
        distribution: false,
        history_dir: history,
        recent: Vec::new(),
        files: Vec::new(),
        views: Vec::new(),
        paint: Paint::from_palette(prefs::series_palette(0, false)),
        text_scale: 1.0,
        sitting_key: String::new(),
    }
}

fn show(app: &mut ScalarScopeApp) {
    let ctx = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1200.0, 900.0))),
        time: Some(0.0),
        ..Default::default()
    };
    let mut frame = eframe::Frame::_new_kittest();
    ctx.run_ui(raw, |ui| app.ui(ui, &mut frame)).drop_without_applying_deltas();
}

fn latency_csv(spike: bool) -> String {
    let mut lines = vec!["step,latency_ms,throughput".to_string()];
    for step in 0..20 {
        let sample = if spike && step == 19 { 100.0 } else { 10.0 };
        lines.push(format!("{step},{sample},{}", 100 + step));
    }
    lines.join("\n")
}

fn loaded(path: &str, text: &str) -> Loaded {
    Loaded {
        path: path.to_string(),
        side: open_text(text, Path::new(path).file_stem().and_then(|stem| stem.to_str()).unwrap_or("run")).unwrap(),
    }
}

fn training_json(run_id: &str) -> String {
    format!(
        r#"[{{"run_id":"{run_id}","status":"completed","model_name":"qwen","steps":40,"loss_history":[1.0,0.5],"final_loss":0.4,"eval":{{"held_out_loss":0.6,"perplexity":2.0,"eval_n":8,"task_metrics":{{"acc":0.9}},"metric_ci":{{"acc":0.1}}}}}}]"#
    )
}

fn entry(kind: &str, bundle: Option<&str>) -> LogEntry {
    LogEntry {
        id: "id".to_string(),
        finished_at: String::new(),
        left_name: "left".to_string(),
        right_name: if kind == "bundle" { String::new() } else { "right".to_string() },
        left_path: None,
        right_path: None,
        left_run_id: None,
        right_run_id: None,
        bundle_path: bundle.map(str::to_string),
        bundle_hash: Some("abcdef0123456789".to_string()),
        alignment: String::new(),
        deltas_fired: vec!["ΔO".to_string()],
        kind: kind.to_string(),
    }
}

#[test]
fn an_unpackaged_window_asks_for_two_runs_and_leaves_the_store_folder_alone() {
    let mut app = ScalarScopeApp::default();
    assert!(app.history_dir.is_none());
    show(&mut app);
    assert!(app.note.is_empty());
    assert!(app.left.is_none());
}

#[test]
fn install_style_paints_the_dark_review() {
    let ctx = egui::Context::default();
    let creation = eframe::CreationContext::_new_kittest(ctx);
    super::install_style(&creation);
}

#[test]
fn the_inference_page_draws_the_series_the_spread_and_the_distribution() {
    let mut app = blank(None);
    app.left = Some(loaded("baseline.csv", &latency_csv(false)));
    app.right = Some(loaded("optimized.csv", &latency_csv(true)));
    app.text_scale = 1.25;
    show(&mut app);
    assert!(app.note.is_empty());
    app.distribution = true;
    show(&mut app);
}

#[test]
fn a_training_pair_draws_loss_and_a_mixed_pair_refuses() {
    let mut app = blank(None);
    app.left = Some(loaded("left.json", &training_json("left-run")));
    app.right = Some(loaded("right.json", &training_json("right-run")));
    show(&mut app);
    assert!(app.note.is_empty());

    app.right = Some(loaded("trace.csv", &latency_csv(false)));
    show(&mut app);
    assert!(app.note.contains("same kind"));
}

#[test]
fn a_saved_palette_changes_the_page_and_a_note_stays_visible() {
    let mut app = blank(None);
    app.note = "The last file did not open.".to_string();
    app.paint = Paint::from_palette(prefs::series_palette(1, false));
    show(&mut app);
    app.paint = Paint::from_palette(prefs::series_palette(5, false));
    show(&mut app);
    app.paint = Paint::from_palette(prefs::series_palette(4, false));
    show(&mut app);
    app.paint = Paint::from_palette(prefs::series_palette(2, false));
    show(&mut app);
    app.paint = Paint::from_palette(prefs::series_palette(3, false));
    show(&mut app);
}

#[test]
fn a_stored_review_draws_inference_training_and_findings_text() {
    let mut app = blank(None);
    let inference = review::pair(
        &open_text(&latency_csv(false), "baseline").unwrap(),
        &open_text(&latency_csv(true), "optimized").unwrap(),
    )
    .unwrap();
    let sealed = bundle::seal(&bundle::document_from_pair(&inference), "2026-10-03T12:00:00Z").unwrap();
    app.opened = Some(bundle::open_bytes(&sealed.bytes).unwrap());
    show(&mut app);

    let training = review::pair(
        &open_text(&training_json("left-run"), "left").unwrap(),
        &open_text(&training_json("right-run"), "right").unwrap(),
    )
    .unwrap();
    let sealed = bundle::seal(&bundle::document_from_pair(&training), "2026-10-03T12:00:00Z").unwrap();
    app.opened = Some(bundle::open_bytes(&sealed.bytes).unwrap());
    show(&mut app);

    app.opened = Some(OpenedBundle {
        bundle_hash: "0123456789abcdef0123456789abcdef".to_string(),
        review: StoredReview {
            kind: "findings".to_string(),
            notices: Vec::new(),
            verdict: "ΔF Introduced 1 new runtime anomalies".to_string(),
            fired: vec!["ΔF".to_string()],
            caption: "The series was not stored.".to_string(),
            left_text: String::new(),
            right_text: String::new(),
            findings: Vec::new(),
            inference: None,
            training: None,
        },
    });
    show(&mut app);
    app.opened.as_mut().unwrap().review.verdict.clear();
    show(&mut app);
}

#[test]
fn recent_files_and_views_are_listed_when_a_package_folder_is_set() {
    let dir = directory();
    let present = dir.join("trace.csv");
    fs::write(&present, latency_csv(false)).unwrap();
    let mut app = blank(Some(dir));
    app.recent = vec![entry("compare", None), entry("bundle", Some("missing.scbundle"))];
    app.files = vec![
        RecentFile {
            path: present.display().to_string(),
            name: "trace".to_string(),
            last_opened: String::new(),
        },
        RecentFile {
            path: r"D:\no\such\trace.csv".to_string(),
            name: "gone".to_string(),
            last_opened: String::new(),
        },
    ];
    app.views = vec![
        SavedView {
            title: "Has a file".to_string(),
            source: Some(present.display().to_string()),
        },
        SavedView {
            title: "No file".to_string(),
            source: None,
        },
    ];
    show(&mut app);

    app.recent.clear();
    app.files.clear();
    app.views.clear();
    show(&mut app);
}

#[test]
fn loading_a_path_fills_one_side_and_a_cancel_or_a_missing_file_does_not() {
    let dir = directory();
    let csv = dir.join("left.csv");
    fs::write(&csv, latency_csv(false)).unwrap();
    let mut app = blank(Some(dir.clone()));

    queue_pick(None);
    app.load(true);
    assert!(app.left.is_none());

    queue_pick(Some(dir.join("missing.csv")));
    app.load(true);
    assert!(app.left.is_none());
    assert!(app.note.contains("Could not read"));

    queue_pick(Some(csv));
    app.load(true);
    assert!(app.left.is_some());
    assert!(app.note.is_empty());

    let bad = dir.join("notes.txt");
    fs::write(&bad, "not a run").unwrap();
    queue_pick(Some(bad));
    app.load(false);
    assert!(app.right.is_none());
    assert!(!app.note.is_empty());
}

#[test]
fn a_preferences_folder_that_is_a_file_keeps_the_opened_run_and_the_error() {
    let dir = directory();
    let csv = dir.join("left.csv");
    fs::write(&csv, latency_csv(false)).unwrap();
    let blocked = dir.join("not-a-folder");
    fs::write(&blocked, "x").unwrap();
    let mut app = blank(Some(blocked));
    queue_pick(Some(csv));
    app.load(false);
    assert!(app.right.is_some());
    assert!(app.note.contains("preferences"));
}

#[test]
fn opening_a_bundle_stores_the_review_and_a_cancel_or_a_bad_file_does_not() {
    let dir = directory();
    let built = review::pair(
        &open_text(&latency_csv(false), "baseline").unwrap(),
        &open_text(&latency_csv(true), "optimized").unwrap(),
    )
    .unwrap();
    let sealed = bundle::seal(&bundle::document_from_pair(&built), "2026-10-03T12:00:00Z").unwrap();
    let path = dir.join("review.scbundle");
    bundle::write_file(&path, &sealed).unwrap();
    let mut app = blank(Some(dir.clone()));

    queue_pick(None);
    app.open_bundle();
    assert!(app.opened.is_none());

    queue_pick(Some(dir.join("missing.scbundle")));
    app.open_bundle();
    assert!(app.opened.is_none());
    assert!(app.note.contains("Could not read"));

    queue_pick(Some(path.clone()));
    app.open_bundle();
    assert!(app.opened.is_some());
    assert!(app.note.is_empty());
    assert_eq!(history::read(&dir).len(), 1);

    let blocked = dir.join("blocked");
    fs::write(&blocked, "x").unwrap();
    app.history_dir = Some(blocked);
    queue_pick(Some(path));
    app.open_bundle();
    assert!(app.opened.is_some());
}

#[test]
fn saving_needs_two_runs_of_the_same_kind_and_a_chosen_file() {
    let dir = directory();
    let mut app = blank(Some(dir.clone()));
    app.save_bundle();
    assert!(app.note.contains("Open both sides"));

    app.left = Some(loaded("left.csv", &latency_csv(false)));
    app.save_bundle();
    assert!(app.note.contains("Open both sides"));

    app.right = Some(loaded("train.json", &training_json("train")));
    app.save_bundle();
    assert!(app.note.contains("same kind"));

    app.right = Some(loaded("right.csv", &latency_csv(true)));
    queue_save(None);
    app.save_bundle();
    assert!(app.note.contains("same kind"));

    let destination = dir.join("review.scbundle");
    queue_save(Some(destination.clone()));
    app.save_bundle();
    assert!(destination.is_file());
    assert!(app.note.contains("Saved the stored review"));
    assert_eq!(history::read(&dir).len(), 1);

    app.history_dir = None;
    queue_save(Some(dir.join("again.scbundle")));
    app.save_bundle();
    assert!(app.note.contains("Saved the stored review"));
    assert!(!app.note.contains("not stamped"));

    queue_save(Some(dir.clone()));
    app.save_bundle();
    assert!(app.note.contains("Could not write"));
}

#[test]
fn a_history_folder_that_is_a_file_reports_that_the_save_was_not_stamped() {
    let dir = directory();
    let blocked = dir.join("blocked");
    fs::write(&blocked, "x").unwrap();
    let mut app = blank(Some(blocked));
    app.left = Some(loaded("left.csv", &latency_csv(false)));
    app.right = Some(loaded("right.csv", &latency_csv(true)));
    queue_save(Some(dir.join("review.scbundle")));
    app.save_bundle();
    assert!(app.note.contains("not stamped"));
}

#[test]
fn the_same_sitting_is_recorded_once_and_a_blocked_folder_keeps_the_error() {
    let dir = directory();
    let pair = review::pair(
        &open_text(&latency_csv(false), "baseline").unwrap(),
        &open_text(&latency_csv(true), "optimized").unwrap(),
    )
    .unwrap();
    let mut app = blank(Some(dir.clone()));
    app.remember("baseline.csv", "optimized.csv", &pair);
    app.remember("baseline.csv", "optimized.csv", &pair);
    assert_eq!(history::read(&dir).len(), 1);
    assert!(!app.sitting_key.is_empty());

    let training = review::pair(
        &open_text(&training_json("left-run"), "left").unwrap(),
        &open_text(&training_json("right-run"), "right").unwrap(),
    )
    .unwrap();
    app.remember("left.json", "right.json", &training);
    assert_eq!(history::read(&dir)[0].alignment, "loss");

    let mut unpackaged = blank(None);
    unpackaged.remember("baseline.csv", "optimized.csv", &pair);
    assert!(unpackaged.sitting_key.is_empty());
    unpackaged.refresh_recent();
    assert!(unpackaged.recent.is_empty());

    let blocked = dir.join("blocked");
    fs::write(&blocked, "x").unwrap();
    let mut failed = blank(Some(blocked));
    failed.remember("baseline.csv", "optimized.csv", &pair);
    assert!(failed.note.contains("history"));
}

#[test]
fn a_bundle_path_without_a_file_name_still_records() {
    let dir = directory();
    let mut app = blank(Some(dir.clone()));
    app.record_opened_bundle(r"C:\", "abcdef0123456789");
    let entries = history::read(&dir);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].left_name, "bundle");

    let mut unpackaged = blank(None);
    unpackaged.record_opened_bundle("review.scbundle", "abcdef0123456789");
    assert!(unpackaged.recent.is_empty());
}

#[test]
fn reopening_uses_the_bundle_or_the_two_paths() {
    let dir = directory();
    let left = dir.join("left.csv");
    let right = dir.join("right.csv");
    fs::write(&left, latency_csv(false)).unwrap();
    fs::write(&right, latency_csv(true)).unwrap();
    let built = review::pair(&open_text(&latency_csv(false), "left").unwrap(), &open_text(&latency_csv(true), "right").unwrap()).unwrap();
    let sealed = bundle::seal(&bundle::document_from_pair(&built), "2026-10-03T12:00:00Z").unwrap();
    let bundle_path = dir.join("review.scbundle");
    bundle::write_file(&bundle_path, &sealed).unwrap();

    let mut app = blank(Some(dir.clone()));
    let mut saved = entry("bundle", Some(&bundle_path.display().to_string()));
    app.reopen(&saved);
    assert!(app.opened.is_some());
    assert!(app.note.is_empty());

    saved.bundle_path = Some(dir.join("missing.scbundle").display().to_string());
    saved.left_path = None;
    saved.right_path = None;
    app.reopen(&saved);
    assert!(app.note.contains("Could not read"));

    let mut runs = entry("compare", None);
    runs.left_path = Some(left.display().to_string());
    runs.right_path = Some(right.display().to_string());
    app.opened = Some(bundle::open_bytes(&sealed.bytes).unwrap());
    app.reopen(&runs);
    assert!(app.opened.is_none());
    assert!(app.left.is_some());
    assert!(app.right.is_some());
    assert!(app.note.is_empty());

    runs.right_path = Some(dir.join("missing.csv").display().to_string());
    app.reopen(&runs);
    assert!(app.note.contains("Could not read"));

    let empty = entry("compare", Some("   "));
    app.note.clear();
    app.reopen(&empty);
    assert!(app.note.contains("no file to reopen"));
}

#[test]
fn a_saved_path_fills_the_empty_side_and_then_the_other() {
    let dir = directory();
    let csv = dir.join("trace.csv");
    fs::write(&csv, latency_csv(false)).unwrap();
    let train = dir.join("history.json");
    fs::write(&train, training_json("alpha")).unwrap();
    let mut app = blank(Some(dir));
    app.open_saved_path(&csv.display().to_string());
    assert!(app.note.contains("path A"));
    assert!(matches!(app.left.as_ref().map(|item| &item.side), Some(Side::Inference(_))));
    app.open_saved_path(&train.display().to_string());
    assert!(app.note.contains("path B"));
    assert!(matches!(app.right.as_ref().map(|item| &item.side), Some(Side::Training(_))));
    app.open_saved_path(r"D:\no\such.csv");
    assert!(app.note.contains("Could not read"));
}

#[test]
fn a_gap_in_the_spread_splits_the_band_and_a_bad_hex_is_black() {
    let band = vec![
        Some(Band { low: 1.0, high: 3.0 }),
        Some(Band { low: 1.0, high: 3.0 }),
        None,
        Some(Band { low: 2.0, high: 4.0 }),
        Some(Band { low: 2.0, high: 4.0 }),
    ];
    assert_eq!(super::band_polygons(&band).len(), 2);
    assert!(super::band_polygons(&[Some(Band { low: 1.0, high: 2.0 })]).is_empty());
    assert!(super::band_polygons(&[None, Some(Band { low: 1.0, high: 2.0 }), None]).is_empty());
    assert_eq!(super::hex_color("ff00aa"), Color32::from_rgb(0xff, 0x00, 0xaa));
    assert_eq!(super::hex_color("nope"), Color32::from_rgb(0, 0, 0));
    let named = loaded("trace.csv", &latency_csv(false));
    assert_eq!(super::side_name(&Some(named), "Path A"), "trace");
    let trained = loaded("history.json", &training_json("alpha"));
    assert_eq!(super::side_name(&Some(trained), "Path A"), "alpha");
    assert_eq!(super::side_name(&None, "Path A"), "Path A");
}

#[test]
fn mark_points_skip_an_index_the_series_does_not_have() {
    let _points = super::mark_points("A marks", &[Some(1.0), None], &[0, 4], Color32::WHITE);
    let _line = super::series_line("A", Color32::WHITE, vec![[0.0, 1.0]]);
    assert_eq!(super::value_points(&[Some(1.0), None]), vec![[0.0, 1.0]]);
    assert_eq!(super::plain_points(&[2.0]), vec![[0.0, 2.0]]);
    assert_eq!(super::cdf_points(&[(3.0, 1.0)]), vec![[3.0, 1.0]]);
}

#[test]
fn the_band_is_one_convex_trapezoid_per_step() {
    let band: Vec<Option<Band>> = (0..6).map(|step| Some(Band { low: step as f64, high: step as f64 + 2.0 + (step % 2) as f64 })).collect();
    let polygons = super::band_polygons(&band);
    assert_eq!(polygons.len(), 5);
    for (index, polygon) in polygons.iter().enumerate() {
        let x = index as f64;
        assert_eq!(polygon.len(), 4);
        assert_eq!([polygon[0][0], polygon[1][0], polygon[2][0], polygon[3][0]], [x, x + 1.0, x + 1.0, x]);
        assert!(polygon[0][1] <= polygon[3][1] && polygon[1][1] <= polygon[2][1]);
    }
}
