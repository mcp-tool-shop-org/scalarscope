//! The comparison log matches the .NET file: same two runs update one entry,
//! a later record keeps an existing bundle hash, and a corrupt file reads empty.

use std::fs;
use std::path::Path;

use scalarscope::history::{self, LogEntry};

#[test]
fn record_updates_the_same_two_runs_instead_of_stacking() {
    let directory = new_directory();
    let first = history::record(pair("left.json", "right.json", vec!["ΔF"], "ByStep"), &directory).unwrap();
    let second = history::record(pair("left.json", "right.json", vec!["ΔF", "ΔTc"], "ByConvergence"), &directory).unwrap();
    let other = history::record(pair("other.json", "right.json", vec!["ΔO"], "ByStep"), &directory).unwrap();

    let entries = history::read(&directory);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, other.id);
    assert_eq!(entries[1].id, first.id);
    assert_eq!(second.id, first.id);
    assert_eq!(entries[1].deltas_fired, vec!["ΔF", "ΔTc"]);
    assert_eq!(entries[1].alignment, "ByConvergence");
    assert_eq!(entries[1].bundle_path, None);
}

#[test]
fn record_keeps_an_existing_bundle_hash_when_the_runs_are_recorded_again() {
    let directory = new_directory();
    let first = history::record(pair("left.json", "right.json", vec!["ΔF"], "ByStep"), &directory).unwrap();
    history::attach_bundle(r"D:\reviews\one.scbundle", Some("abcdef0123456789"), &directory).unwrap();

    let again = history::record(pair("left.json", "right.json", vec!["ΔTc"], "ByStep"), &directory).unwrap();

    assert_eq!(again.id, first.id);
    assert_eq!(again.bundle_path.as_deref(), Some(r"D:\reviews\one.scbundle"));
    assert_eq!(again.bundle_hash.as_deref(), Some("abcdef0123456789"));
    assert_eq!(again.deltas_fired, vec!["ΔTc"]);
    assert_eq!(history::read(&directory).len(), 1);
}

#[test]
fn attach_bundle_stamps_the_newest_live_review() {
    let directory = new_directory();
    history::record(pair("left.json", "right.json", vec!["ΔO"], "ByStep"), &directory).unwrap();

    let stamped = history::attach_bundle(r"D:\reviews\one.scbundle", Some("0123456789abcdef"), &directory).unwrap();

    assert_eq!(stamped.kind, "compare");
    assert_eq!(stamped.bundle_hash.as_deref(), Some("0123456789abcdef"));
    assert_eq!(stamped.subtitle(), "ΔO · 01234567");
}

#[test]
fn attach_appends_when_the_newest_entry_is_a_different_bundle() {
    let directory = new_directory();
    history::attach_bundle(r"D:\reviews\one.scbundle", Some("aaaaaaaaaaaaaaaa"), &directory).unwrap();
    history::attach_bundle(r"D:\reviews\two.scbundle", Some("bbbbbbbbbbbbbbbb"), &directory).unwrap();

    let entries = history::read(&directory);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].kind, "bundle");
    assert_eq!(entries[0].bundle_path.as_deref(), Some(r"D:\reviews\two.scbundle"));
    assert_eq!(entries[1].bundle_path.as_deref(), Some(r"D:\reviews\one.scbundle"));
}

#[test]
fn record_caps_the_file_at_forty_entries() {
    let directory = new_directory();
    for index in 0..=history::MAX_ENTRIES {
        history::record(pair(&format!("l{index}.json"), &format!("r{index}.json"), vec!["ΔF"], "ByStep"), &directory).unwrap();
    }

    let entries = history::read(&directory);
    assert_eq!(entries.len(), history::MAX_ENTRIES);
    let newest = format!("l{}.json", history::MAX_ENTRIES);
    assert_eq!(entries[0].left_path.as_deref(), Some(newest.as_str()));
    assert!(entries.iter().all(|entry| entry.left_path.as_deref() != Some("l0.json")));
}

#[test]
fn read_returns_empty_when_the_file_is_corrupt() {
    let directory = new_directory();
    fs::write(history::file_path(&directory), "{not json").unwrap();
    assert!(history::read(&directory).is_empty());
}

#[test]
fn clear_removes_the_log() {
    let directory = new_directory();
    history::record(pair("left.json", "right.json", vec![], "ByStep"), &directory).unwrap();
    history::clear(&directory).unwrap();
    assert!(history::read(&directory).is_empty());
    assert!(!history::file_path(&directory).exists());
}

#[test]
fn a_maui_log_with_an_offset_timestamp_still_reads() {
    let directory = new_directory();
    let json = r#"[{
        "id": "abc",
        "finishedAt": "2026-10-03T12:00:00+00:00",
        "leftName": "left",
        "rightName": "right",
        "leftPath": "left.json",
        "rightPath": "right.json",
        "alignment": "ByStep",
        "deltasFired": ["ΔO"],
        "kind": "compare"
    }]"#;
    fs::write(history::file_path(&directory), json).unwrap();
    let entries = history::read(&directory);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title(), "left vs right");
    assert_eq!(entries[0].subtitle(), "ΔO");
    assert_eq!(entries[0].finished_at, "2026-10-03T12:00:00+00:00");
}

#[test]
fn an_unpackaged_process_does_not_invent_the_store_folder() {
    assert!(history::package_local_state().is_none());
}

#[test]
fn a_bundle_title_is_the_file_stem_and_an_empty_right_name_is_not_a_versus() {
    let directory = new_directory();
    let saved = history::attach_bundle(r"D:\reviews\one.scbundle", None, &directory).unwrap();
    assert_eq!(saved.title(), "one");
    assert_eq!(saved.subtitle(), "No deltas fired");
}

fn pair(left: &str, right: &str, deltas: Vec<&str>, alignment: &str) -> LogEntry {
    LogEntry {
        id: String::new(),
        finished_at: String::new(),
        left_name: Path::new(left).file_stem().and_then(|stem| stem.to_str()).unwrap_or(left).to_string(),
        right_name: Path::new(right).file_stem().and_then(|stem| stem.to_str()).unwrap_or(right).to_string(),
        left_path: Some(left.to_string()),
        right_path: Some(right.to_string()),
        left_run_id: None,
        right_run_id: None,
        bundle_path: None,
        bundle_hash: None,
        alignment: alignment.to_string(),
        deltas_fired: deltas.into_iter().map(str::to_string).collect(),
        kind: "compare".to_string(),
        fingerprints: None,
        measures: None,
    }
}

fn new_directory() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
    let directory = std::env::temp_dir().join(format!("scalarscope-log-{}-{nanos}-{id}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    directory
}
