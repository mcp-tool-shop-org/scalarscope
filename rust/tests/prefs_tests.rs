use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use scalarscope::prefs::{self, series_palette};

#[test]
fn remember_keeps_fields_this_review_does_not_use() {
    let directory = new_directory();
    fs::write(
        directory.join(prefs::FILE_NAME),
        r#"{
            "HasSeenDemo": true,
            "DismissedHints": ["welcome"],
            "ColorVisionMode": 1,
            "HighContrastMode": false,
            "TextScale": 1.25,
            "RecentFilesLimit": 2,
            "FutureFlag": 1,
            "RecentFiles": [
                {"Path": "D:\\reviews\\old.json", "Name": "old", "LastOpened": "2026-10-01T00:00:00Z"}
            ]
        }"#,
    )
    .unwrap();

    prefs::remember_file(&directory, r"D:\reviews\new.json", "new").unwrap();
    let text = fs::read_to_string(directory.join(prefs::FILE_NAME)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["HasSeenDemo"], true);
    assert_eq!(value["DismissedHints"][0], "welcome");
    assert_eq!(value["ColorVisionMode"], 1);
    assert_eq!(value["FutureFlag"], 1);
    assert_eq!(value["RecentFiles"][0]["Path"], r"D:\reviews\new.json");
    assert_eq!(value["RecentFiles"].as_array().unwrap().len(), 2);
    assert!(text.contains("old.json"));

    let read = prefs::read(&directory);
    assert_eq!(read.color_vision, 1);
    assert!((read.text_scale - 1.25).abs() < 1e-6);
    assert_eq!(read.recent_limit, 2);
    assert_eq!(read.recent[0].name, "new");
    assert_eq!(read.recent[1].path, r"D:\reviews\old.json");

    prefs::remember_file(&directory, r"D:\reviews\third.json", "third").unwrap();
    let trimmed = prefs::read(&directory);
    assert_eq!(trimmed.recent.len(), 2);
    assert_eq!(trimmed.recent[0].name, "third");
    assert!(trimmed.recent.iter().all(|file| file.name != "old"));
    let kept = fs::read_to_string(directory.join(prefs::FILE_NAME)).unwrap();
    assert!(kept.contains("HasSeenDemo"));
    assert!(kept.contains("FutureFlag"));
}

#[test]
fn a_missing_file_stays_in_the_list() {
    let directory = new_directory();
    fs::write(
        directory.join(prefs::FILE_NAME),
        r#"{"RecentFiles":[{"Path":"D:\\reviews\\missing.json","Name":"missing","LastOpened":"2026-10-03T12:00:00Z"}]}"#,
    )
    .unwrap();
    let read = prefs::read(&directory);
    assert_eq!(read.recent.len(), 1);
    assert_eq!(read.recent[0].last_opened, "2026-10-03T12:00:00Z");
    let after = fs::read_to_string(directory.join(prefs::FILE_NAME)).unwrap();
    assert!(after.contains("missing.json"));
}

#[test]
fn a_corrupt_preferences_file_is_left_unchanged() {
    let directory = new_directory();
    let path = directory.join(prefs::FILE_NAME);
    fs::write(&path, "{not json").unwrap();
    let error = prefs::remember_file(&directory, r"D:\reviews\one.json", "one").unwrap_err();
    assert!(error.contains("left unchanged"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "{not json");
    assert_eq!(prefs::read(&directory).color_vision, 0);
}

#[test]
fn the_same_path_updates_one_entry() {
    let directory = new_directory();
    prefs::remember_file(&directory, r"D:\reviews\a.json", "a").unwrap();
    prefs::remember_file(&directory, r"D:\reviews\b.json", "b").unwrap();
    prefs::remember_file(&directory, r"d:\reviews\a.json", "a again").unwrap();
    let read = prefs::read(&directory);
    assert_eq!(read.recent.len(), 2);
    assert_eq!(read.recent[0].name, "a again");
    assert_eq!(read.recent[0].path, r"d:\reviews\a.json");
}

#[test]
fn series_colors_follow_the_saved_palette() {
    let default = series_palette(0, false);
    assert_eq!(default.left, "4ecdc4");
    assert_eq!(default.right, "ff6b6b");
    assert_eq!(default.label, None);

    let deuteranopia = series_palette(1, true);
    assert_eq!((deuteranopia.left, deuteranopia.right, deuteranopia.mark), ("0077bb", "ee7733", "33bbee"));
    assert_eq!(deuteranopia.label, Some("deuteranopia"));

    let protanopia = series_palette(2, false);
    assert_eq!((protanopia.left, protanopia.right), ("0077bb", "ddcc77"));

    let tritanopia = series_palette(3, false);
    assert_eq!((tritanopia.left, tritanopia.right, tritanopia.mark), ("ee3377", "009988", "ee7733"));

    let contrast = series_palette(0, true);
    assert_eq!((contrast.left, contrast.right, contrast.mark), ("ffff00", "00ffff", "ff00ff"));
    assert_eq!(series_palette(4, false).label, Some("high contrast"));

    let mono = series_palette(5, false);
    assert_eq!((mono.left, mono.right, mono.mark), ("ffffff", "cccccc", "888888"));
    assert_eq!(series_palette(9, false).label, None);
}

#[test]
fn text_scale_is_clamped_and_a_missing_file_stays_default() {
    let directory = new_directory();
    assert_eq!(prefs::read(&directory).text_scale, 1.0);
    fs::write(directory.join(prefs::FILE_NAME), r#"{"TextScale": 4, "RecentFilesLimit": 0}"#).unwrap();
    let read = prefs::read(&directory);
    assert!((read.text_scale - 2.0).abs() < 1e-6);
    assert_eq!(read.recent_limit, 10);
}

#[test]
fn saved_views_keep_the_source_and_skip_a_corrupt_file() {
    let directory = new_directory();
    let gallery = directory.join("gallery");
    fs::create_dir_all(&gallery).unwrap();
    fs::write(
        gallery.join("later.json"),
        r#"{"title":"Later","createdAt":"2026-10-03T12:00:00Z","state":{"dataSource":"D:\\reviews\\trace.json"}}"#,
    )
    .unwrap();
    fs::write(gallery.join("broken.json"), "{not json").unwrap();
    fs::write(
        gallery.join("earlier.json"),
        r#"{"title":"Earlier","createdAt":"2026-10-01T00:00:00Z","state":{"dataSource":"D:\\reviews\\old.json"}}"#,
    )
    .unwrap();
    let views = prefs::saved_views(&directory);
    assert_eq!(views.len(), 2);
    assert_eq!(views[0].title, "Later");
    assert_eq!(views[0].source.as_deref(), Some(r"D:\reviews\trace.json"));
    assert_eq!(views[1].title, "Earlier");
}

fn new_directory() -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let directory = std::env::temp_dir().join(format!("scalarscope-prefs-{}-{nanos}-{id}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    directory
}
