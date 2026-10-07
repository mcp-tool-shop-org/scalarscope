//! History: projects, change points per measure, and code or environment changes beside them
//! (spec 5c). The fixture is a simulated comparison log (`tests/fixtures/history`).

use std::path::PathBuf;

use scalarscope::history;
use scalarscope::stats::AnomalyRule;
use scalarscope::trends::{entry_facts, marks, mean_shift_pelt, projects, trend, MIN_REVIEWS, PENALTY};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/history")
}

#[test]
fn reviews_group_by_side_b_dataset_and_model() {
    let entries = history::read(&fixture_dir());
    assert_eq!(entries.len(), 11);
    let projects = projects(&entries);
    // The bundle opening is not a review of a project; the 2.0 entry has no fingerprints.
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].label, "dataset aaaaaaaa · model 55555555");
    assert_eq!(projects[0].entries.len(), 9);
    assert_eq!(projects[0].entries[0].right_name, "nightly 1", "oldest first");
    assert_eq!(projects[1].label, "Unsorted");
    assert_eq!(projects[1].entries[0].id, "dotnet-1");
}

#[test]
fn a_shift_beside_an_environment_change_names_it_and_a_flat_measure_has_none() {
    let entries = history::read(&fixture_dir());
    let project = &projects(&entries)[0];
    assert_eq!(marks(project).iter().map(|mark| (mark.at, mark.what)).collect::<Vec<_>>(), vec![(5, "environment")]);

    let p99 = trend(project, "p99");
    assert!(p99.searched);
    assert_eq!(p99.shifts, vec![5]);
    assert_eq!(p99.sentences, vec!["p99 shifted at 2026-10-15, with an environment change."]);

    let p50 = trend(project, "p50");
    assert!(p50.searched && p50.shifts.is_empty(), "{:?}", p50.shifts);
    assert!(p50.sentences.is_empty());

    // A measure no review recorded has no points and is not searched.
    let memory = trend(project, "memory_peak");
    assert!(memory.points.is_empty() && !memory.searched);
}

#[test]
fn too_few_reviews_are_drawn_but_not_segmented() {
    let mut entries = history::read(&fixture_dir());
    entries.retain(|entry| entry.right_name.starts_with("nightly") && entry.right_name.as_str() <= "nightly 5");
    let project = &projects(&entries)[0];
    assert_eq!(project.entries.len(), 5);
    assert!(project.entries.len() < MIN_REVIEWS);
    let p99 = trend(project, "p99");
    assert_eq!(p99.points.len(), 5);
    assert!(!p99.searched && p99.shifts.is_empty());
}

#[test]
fn mean_shift_pelt_finds_a_step_and_ignores_noise() {
    let step: Vec<f64> = [1.0, 1.02, 0.98, 1.01, 0.99, 2.0, 2.01, 1.99, 2.02, 1.98].to_vec();
    assert_eq!(mean_shift_pelt(&step, PENALTY, 3), vec![5]);
    let noise: Vec<f64> = [1.0, 1.02, 0.98, 1.01, 0.99, 1.0, 1.01, 0.99, 1.02, 0.98].to_vec();
    assert!(mean_shift_pelt(&noise, PENALTY, 3).is_empty());
    let flat = vec![3.0; 8];
    assert!(mean_shift_pelt(&flat, PENALTY, 3).is_empty());
    assert!(mean_shift_pelt(&step[..5], PENALTY, 3).is_empty(), "too short for two segments");
    // Two steps.
    let stairs: Vec<f64> = [1.0, 1.01, 0.99, 2.0, 2.01, 1.99, 3.0, 3.01, 2.99].to_vec();
    assert_eq!(mean_shift_pelt(&stairs, PENALTY, 3), vec![3, 6]);
}

#[test]
fn a_review_records_side_b_fingerprints_and_measures() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workbench/batch4_seed1_runtrace.json");
    let scalarscope::open::Side::Inference(run) = scalarscope::open::open_path(&path).unwrap().side else { panic!("inference") };
    let (fingerprints, measures) = entry_facts(&run, AnomalyRule::Mad);
    let fingerprints = fingerprints.unwrap();
    assert_eq!(fingerprints.dataset, "a".repeat(64));
    assert_eq!(fingerprints.environment, "c".repeat(64));
    let measures = measures.unwrap();
    assert!((measures["p50"] / 10.0 - 1.0).abs() < 0.03, "{}", measures["p50"]);
    assert!(measures.contains_key("p99") && measures.contains_key("memory_peak"));
    // The history fields round-trip and stay out of a 2.0 entry.
    let dir = std::env::temp_dir().join(format!("scalarscope-trends-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut entry = history::read(&fixture_dir()).into_iter().find(|entry| entry.id == "h0").unwrap();
    entry.measures = Some(measures.clone());
    history::record(entry, &dir).unwrap();
    let back = history::read(&dir);
    assert_eq!(back[0].measures.as_ref(), Some(&measures));
    let text = std::fs::read_to_string(history::file_path(&dir)).unwrap();
    assert!(text.contains("\"fingerprints\""));
    let old = history::read(&fixture_dir()).into_iter().find(|entry| entry.id == "dotnet-1").unwrap();
    let written = serde_json::to_string(&old).unwrap();
    assert!(!written.contains("fingerprints") && !written.contains("measures"));
}
