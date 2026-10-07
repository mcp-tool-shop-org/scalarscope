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
        view: super::View::Series,
        threshold: None,
        elapsed_axis: false,
        why: None,
        highlight: None,
        built: None,
        saved_hash: None,
        page: super::Page::Compare,
        guide_query: String::new(),
        pending_png: None,
        scrub: None,
        settings: prefs::ReviewPrefs::default(),
        history_dir: history,
        recent: Vec::new(),
        files: Vec::new(),
        views: Vec::new(),
        paint: Paint::from_palette(prefs::series_palette(0, false)),
        text_scale: 1.0,
        sitting_key: String::new(),
        bench: Default::default(),
        history: Default::default(),
        capture: None,
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
        fingerprints: None,
        measures: None,
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
    for (view, _) in super::View::ALL {
        app.view = view;
        show(&mut app);
    }
}

fn long_csv(level: f64, seed: u64) -> String {
    let mut rng = crate::stats::Rng::new(seed);
    let mut lines = vec!["step,latency_ms".to_string()];
    for step in 0..400 {
        let jitter = ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * 0.1;
        let spike = if step % 97 == 50 { 3.0 } else { 1.0 };
        lines.push(format!("{step},{}", level * (1.0 + jitter) * spike + 20.0 * (-(step as f64) / 15.0).exp()));
    }
    lines.join("
")
}

#[test]
fn every_view_draws_a_long_pair_and_the_threshold_reads_both_runs() {
    let mut app = blank(None);
    app.left = Some(loaded("baseline.csv", &long_csv(12.0, 1)));
    app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
    for (view, _) in super::View::ALL {
        app.view = view;
        show(&mut app);
        assert!(app.note.is_empty(), "{view:?}: {}", app.note);
    }
    app.threshold = Some(10.0);
    app.view = super::View::Distribution;
    show(&mut app);
}

#[test]
fn the_series_draws_on_elapsed_seconds_with_utilization() {
    let timed = |level: f64| -> String {
        let mut lines = vec!["step,latency_ms,time_s,cpu_percent,gpu_percent".to_string()];
        for step in 0..120 {
            lines.push(format!("{step},{},{},{},{}", level + (step % 5) as f64 * 0.1, step as f64 * 0.02, 30 + step % 7, 80));
        }
        lines.join("
")
    };
    let mut app = blank(None);
    app.left = Some(loaded("a.csv", &timed(12.0)));
    app.right = Some(loaded("b.csv", &timed(9.0)));
    show(&mut app);
    app.elapsed_axis = true;
    show(&mut app);
    assert!(app.note.is_empty(), "{}", app.note);
}

#[test]
fn tiles_open_why_and_show_me_moves_to_the_anchor() {
    let mut app = blank(None);
    app.left = Some(loaded("baseline.csv", &long_csv(12.0, 1)));
    app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
    show(&mut app);
    let Some((_, Ok(crate::review::Pair::Inference(review)))) = app.built.clone() else { panic!("built") };
    let symbols: Vec<&str> = review.explanations.iter().map(|tile| tile.symbol.as_str()).collect();
    assert_eq!(symbols, vec!["ΔF", "ΔTc", "ΔO"]);
    app.why = Some("ΔTc".to_string());
    show(&mut app);
    let anchor = review.explanations[1].anchor.clone().expect("ΔTc anchors on the series");
    app.highlight = Some((anchor.from, anchor.to));
    show(&mut app);
}

#[test]
fn the_review_is_built_once_per_pair_of_inputs() {
    let mut app = blank(None);
    app.left = Some(loaded("baseline.csv", &long_csv(12.0, 1)));
    app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
    show(&mut app);
    let first = app.built.clone().unwrap().0;
    show(&mut app);
    assert_eq!(app.built.clone().unwrap().0, first);
    app.right = Some(loaded("other.csv", &long_csv(9.0, 3)));
    show(&mut app);
    assert_ne!(app.built.clone().unwrap().0, first);
}

#[test]
fn the_settings_page_draws_and_light_theme_paints_dark_text() {
    let mut app = blank(None);
    app.page = super::Page::Settings;
    show(&mut app);
    for theme in [0, 1, 2] {
        app.settings.theme = theme;
        app.page = super::Page::Compare;
        app.left = Some(loaded("baseline.csv", &long_csv(12.0, 1)));
        app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
        show(&mut app);
    }
    let light = super::Paint::themed(prefs::series_palette(0, false), true);
    let dark = super::Paint::themed(prefs::series_palette(0, false), false);
    assert_ne!(light.background, dark.background);
    assert_ne!(light.text, Color32::WHITE);
    assert_eq!(dark.text, Color32::WHITE);
}

#[test]
fn the_anomaly_setting_reaches_the_review_and_rebuilds_it() {
    let mut app = blank(None);
    app.left = Some(loaded("baseline.csv", &long_csv(12.0, 1)));
    app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
    show(&mut app);
    let mad = app.built.clone().unwrap();
    app.settings.anomaly_rule = 1;
    app.settings_changed();
    show(&mut app);
    let sigma = app.built.clone().unwrap();
    assert_ne!(mad.0, sigma.0);
    let Ok(crate::review::Pair::Inference(review)) = sigma.1 else { panic!("inference") };
    assert!(review.caption.contains("3 population standard deviations"), "{}", review.caption);
}

#[test]
fn welcome_loads_the_sample_and_guide_search_filters() {
    let mut app = blank(None);
    app.page = super::Page::Welcome;
    show(&mut app);
    app.load_sample();
    assert_eq!(app.page, super::Page::Compare);
    show(&mut app);
    let Some((_, Ok(crate::review::Pair::Inference(review)))) = app.built.clone() else { panic!("the sample builds: {}", app.note) };
    assert!(review.headline.starts_with("B/A p50 0.6"), "{}", review.headline);
    assert_eq!(super::pages::search("").len(), super::pages::GUIDE.len());
    assert!(super::pages::search("ΔTc").iter().any(|section| section.title.starts_with("ΔTc")));
    assert!(super::pages::search("zzz-not-there").is_empty());
    app.page = super::Page::Guide;
    app.guide_query = "bundle".to_string();
    show(&mut app);
}

#[test]
fn number_keys_choose_views() {
    assert_eq!(super::pages::view_for_key(egui::Key::Num2), Some(super::View::Warmup));
    assert_eq!(super::pages::view_for_key(egui::Key::Num6), Some(super::View::HeatMap));
    assert_eq!(super::pages::view_for_key(egui::Key::A), None);
}

#[test]
fn each_view_exports_an_svg_and_a_png_round_trips() {
    let mut app = blank(None);
    app.left = Some(loaded("base<line>.csv", &long_csv(12.0, 1)));
    app.right = Some(loaded("optimized.csv", &long_csv(8.0, 2)));
    show(&mut app);
    let Some((_, Ok(crate::review::Pair::Inference(review)))) = app.built.clone() else { panic!("built") };
    let colors = app.svg_colors();
    for svg in [
        crate::svg::series(&review, colors),
        crate::svg::warmup(&review, colors),
        crate::svg::distribution(&review, colors, Some(10.0)),
        crate::svg::difference(&review, colors),
        crate::svg::spectrum(&review, colors),
    ] {
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
        assert!(svg.contains("base&lt;line&gt;"), "labels are escaped");
        assert_eq!(svg.matches("<svg").count(), 1);
    }
    let dir = std::env::temp_dir().join(format!("scalarscope-export-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("view.svg");
    super::queue_save(Some(path.clone()));
    app.view = super::View::Difference;
    app.export_svg(&review);
    assert!(fs::read_to_string(&path).unwrap().contains("by percentile"));

    let image = egui::ColorImage::new([3, 2], vec![Color32::from_rgb(10, 20, 30); 6]);
    let bytes = super::encode_png(&image).unwrap();
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((info.width, info.height), (3, 2));
    assert_eq!(&pixels[..4], &[10, 20, 30, 255]);
}

#[test]
fn a_geometry_pair_draws_and_the_scrub_moves() {
    let sample = |name: &str| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ScalarScope/Resources/Raw/Samples").join(name);
        fs::read_to_string(path).unwrap()
    };
    let mut app = blank(None);
    app.left = Some(loaded("orthogonal.json", &sample("orthogonal_professors.json")));
    app.right = Some(loaded("correlated.json", &sample("correlated_professors.json")));
    show(&mut app);
    assert!(app.note.is_empty(), "{}", app.note);
    let Some((_, Ok(crate::review::Pair::Geometry(review)))) = app.built.clone() else { panic!("a geometry pair") };
    assert_eq!(review.left_label, "sample_orthogonal_001");
    app.scrub = Some(0.0);
    show(&mut app);
    app.right = Some(loaded("trace.csv", &latency_csv(false)));
    show(&mut app);
    assert!(app.note.contains("same kind"), "{}", app.note);
}

#[test]
fn percentile_labels_name_the_nines() {
    assert_eq!(super::percentile_label(0.0), "p0");
    assert_eq!(super::percentile_label(1.0), "p90");
    assert_eq!(super::percentile_label(2.0), "p99");
    assert_eq!(super::percentile_label(3.0), "p99.9");
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
            headline: String::new(),
            explanations: Vec::new(),
            geometry: None,
            verdict: "ΔF Introduced 1 new runtime anomalies".to_string(),
            fired: vec!["ΔF".to_string()],
            caption: "The series was not stored.".to_string(),
            left_text: String::new(),
            right_text: String::new(),
            findings: Vec::new(),
            inference: None,
            training: None,
        },
        verified: true,
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
    app.load(true, false);
    assert!(app.left.is_none());

    queue_pick(Some(dir.join("missing.csv")));
    app.load(true, false);
    assert!(app.left.is_none());
    assert!(app.note.contains("Could not read"));

    queue_pick(Some(csv));
    app.load(true, false);
    assert!(app.left.is_some());
    assert!(app.note.is_empty());

    let bad = dir.join("notes.txt");
    fs::write(&bad, "not a run").unwrap();
    queue_pick(Some(bad));
    app.load(false, false);
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
    app.load(false, false);
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
    assert_eq!(super::mark_coords(&[Some(1.0), None], &[0, 4]), vec![[0.0, 1.0]]);
    assert_eq!(super::at_x(vec![[1.0, 5.0]], Some(&[0.0, 2.5])), vec![[2.5, 5.0]]);
    assert_eq!(super::x_of(3, None), 3.0);
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

fn workbench_folder() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workbench")
}

#[test]
fn the_workbench_page_asks_for_runs_then_weighs_a_picked_folder() {
    let dir = directory();
    let mut app = blank(Some(dir.clone()));
    app.page = super::Page::Workbench;
    show(&mut app);
    assert!(app.bench_board().is_none());

    // A folder of runs, picked on the page.
    queue_pick(Some(workbench_folder()));
    app.open_bench_runs();
    assert_eq!(app.bench.runs.len(), 9);
    let board = app.bench_board().unwrap();
    assert_eq!(board.varying, vec!["batch".to_string()]);
    show(&mut app);
    // The memory is in the Store folder, and the new set of runs counts toward a checkpoint.
    assert!(dir.join(crate::workbench::MEMORY_FILE).exists());
    assert_eq!(app.bench.book.since, 1);
    show(&mut app);
    assert_eq!(app.bench.book.since, 1, "the same runs count once");

    app.bench.formula = "p99 / p50".to_string();
    app.bench.formula_lines = match workbench::evaluate(&board, &app.bench.formula, &[]) {
        Ok(column) => column.lines(),
        Err(reason) => vec![reason],
    };
    assert_eq!(app.bench.formula_lines.len(), 9);

    // A file that is not a run is refused with its reason; a folder with no runs too.
    queue_pick(Some(dir.join("missing.json")));
    app.open_bench_runs();
    assert!(!app.bench.status.is_empty());
    assert_eq!(app.bench.runs.len(), 9);
    assert!(super::bench_page::open_runs(&[directory()]).unwrap_err().contains("No inference run"));
}

#[test]
fn a_finished_session_is_kept_shown_and_saved_without_paths() {
    let dir = directory();
    let mut app = blank(Some(dir.clone()));
    app.page = super::Page::Workbench;
    app.bench.runs = super::bench_page::open_runs(&[workbench_folder()]).unwrap();
    let board = app.bench_board().unwrap();
    let mut bench = workbench::Workbench::new(board, Vec::new(), Vec::new(), "2026-10-06");
    bench.call("measure", &serde_json::json!({"formula": "p50"}));
    bench.call("learn_tool", &serde_json::json!({"name": "tail_weight", "formula": "p99 / p90", "meaning": "how far the slowest steps sit past the ninetieth percentile"}));
    bench.call("propose_hypothesis", &serde_json::json!({"knob": "batch", "formula": "p50", "knob_change": "raise", "formula_moves": "up", "why": "A bigger batch does more work per step."}));
    bench.call("finish", &serde_json::json!({"note": "Latency rises with the batch; precision did not vary."}));
    app.bench.your_call = "Batch raises latency.".to_string();

    // The reply arrives as the model loop would send it.
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(workbench::BenchReply::Done {
        model: "qwen2.5:14b".to_string(),
        digest: Some("7cdf5a0187d5aaaa".to_string()),
        bench: Box::new(bench),
        stopped: "The model finished.".to_string(),
    })
    .unwrap();
    app.bench.ask = Some(rx);
    show(&mut app);
    assert!(app.bench.ask.is_none());
    let last = app.bench.last.as_ref().unwrap();
    assert_eq!(last.proposed.len(), 1);
    assert_eq!(last.proposed[0].0, "When batch size goes up, p50 goes higher.");
    assert_eq!(last.learned.len(), 1);
    assert_eq!(app.bench.hypotheses.len(), 1);
    assert_eq!(workbench::read_hypotheses(&dir.join(crate::workbench::MEMORY_FILE)).len(), 1);
    assert_eq!(workbench::read_tools(&dir.join(crate::workbench::MEMORY_FILE)).len(), 1);
    show(&mut app);

    let out = dir.join("session.json");
    queue_save(Some(out.clone()));
    app.save_session_record();
    let text = fs::read_to_string(&out).unwrap();
    let record: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(record["digest"], "7cdf5a0187d5aaaa");
    assert_eq!(record["your_call"], "Batch raises latency.");
    assert!(!text.contains("Fixtures") && !text.contains("Temp"), "the record holds a path");

    // No model: the page says why.
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(workbench::BenchReply::Absent("The local model is not running.".to_string())).unwrap();
    app.bench.ask = Some(rx);
    show(&mut app);
    assert_eq!(app.bench.status, "The local model is not running.");
    let (tx, rx) = std::sync::mpsc::channel::<workbench::BenchReply>();
    drop(tx);
    app.bench.ask = Some(rx);
    app.poll_workbench();
    assert!(app.bench.status.contains("stopped without an answer"));
}

#[test]
fn an_unpackaged_window_keeps_a_session_only_while_it_is_open() {
    let mut app = blank(None);
    app.bench.runs = super::bench_page::open_runs(&[workbench_folder()]).unwrap();
    let board = app.bench_board().unwrap();
    let bench = workbench::Workbench::new(board, Vec::new(), Vec::new(), "2026-10-06");
    app.keep_session("qwen2.5:14b", None, &bench, "The model finished.");
    assert!(app.bench.status.contains("nothing is kept"));
    app.page = super::Page::Workbench;
    show(&mut app);
}

#[test]
fn the_history_page_draws_a_project_its_shift_and_the_empty_cases() {
    let dir = directory();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/history/comparison-log.json"),
        history::file_path(&dir),
    )
    .unwrap();
    let mut app = blank(Some(dir));
    app.page = super::Page::History;
    show(&mut app);
    // Every measure, including ones no review recorded, draws without a panic.
    for measure in 0..crate::trends::MEASURES.len() {
        app.history.measure = measure;
        show(&mut app);
    }
    // The 2.0 entries' project: one review, too few to segment.
    app.history.project = 1;
    app.history.measure = 2;
    show(&mut app);
    // A selection past the end is clamped.
    app.history.project = 99;
    show(&mut app);
    assert_eq!(app.history.project, 1);

    let mut empty = blank(Some(directory()));
    empty.page = super::Page::History;
    show(&mut empty);
    let mut unpackaged = blank(None);
    unpackaged.page = super::Page::History;
    show(&mut unpackaged);
}

#[test]
fn the_geometry_page_draws_the_real_drift_pair_unordered() {
    let fixture = |name: &str| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aspire-si").join(name);
    let mut app = blank(None);
    app.open_pair(&fixture("real-local-teacher.drift.geometry.json"), &fixture("real-composite-teacher.drift.geometry.json"));
    assert!(app.left.is_some() && app.right.is_some(), "{}", app.note);
    show(&mut app);
    app.open_pair(&fixture("real-local-teacher.geometry.json"), &fixture("real-composite-teacher.geometry.json"));
    show(&mut app);
    // A path that does not open leaves its side empty with the reason.
    app.open_pair(&fixture("missing.json"), &fixture("real-local-teacher.geometry.json"));
    assert!(app.left.is_none() && !app.note.is_empty());
}

#[test]
fn capture_mode_sets_the_page_and_view_and_counts_down() {
    let mut app = blank(None);
    let out = directory().join("shot.png");
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workbench");
    app.capture(out.clone(), "workbench", Some(3), Some("ΔF".to_string()), Some(&folder), Some(1.3));
    assert_eq!(app.text_scale, 1.3);
    assert_eq!(app.page, super::Page::Workbench);
    assert_eq!(app.view, super::View::Distribution);
    assert_eq!(app.bench.runs.len(), 9);
    for _ in 0..40 {
        show(&mut app);
    }
    // The screenshot is asked for; with no renderer in a test it never arrives, so the path waits.
    assert_eq!(app.pending_png.as_deref(), Some(out.as_path()));
    for (page, expected) in [("welcome", super::Page::Welcome), ("history", super::Page::History), ("guide", super::Page::Guide), ("settings", super::Page::Settings), ("compare", super::Page::Compare)] {
        let mut other = blank(None);
        other.capture(directory().join("x.png"), page, None, None, None, Some(9.0));
        assert_eq!(other.text_scale, 1.0, "a scale out of range is ignored");
        assert_eq!(other.page, expected);
    }
    let mut missing = blank(None);
    missing.capture(directory().join("x.png"), "workbench", Some(9), None, Some(&directory()), None);
    assert!(missing.note.contains("No inference run"));
}
