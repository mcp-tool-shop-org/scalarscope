//! Bundles across the two apps, using files the 2.0 code wrote (tests/Fixtures/Bundles, made by
//! tests/ScalarScope.FixtureTests/BundleFixtureWriter.cs) and one this review writes for the 2.0
//! importer to read back.

use std::fs;
use std::path::PathBuf;

use scalarscope::bundle::{document_from_pair, open_file, seal, verify};
use scalarscope::open::open_text;
use scalarscope::review::pair;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/Fixtures").join(name)
}

#[test]
fn a_2_0_inference_review_opens_unverified_with_its_deltas() {
    let opened = open_file(&fixture("Bundles/dotnet-inference-review.scbundle")).unwrap();
    assert!(!opened.verified, "2.0 wrote no integrity.json");
    assert_eq!(opened.review.kind, "dotnet-review");
    assert!(opened.review.verdict.contains("ΔTc Stabilizes 6 steps earlier"), "{}", opened.review.verdict);
    assert_eq!((opened.review.left_text.as_str(), opened.review.right_text.as_str()), ("Baseline", "Optimized"));
    let tc = opened.review.explanations.iter().find(|tile| tile.symbol == "ΔTc").expect("a ΔTc tile");
    assert_eq!(tc.status, "fired");
    assert!(tc.parameters.iter().any(|[name, value]| name == "Baseline" && value == "13"), "{:?}", tc.parameters);
    assert_eq!(opened.bundle_hash.len(), 64, "the stated hash");
}

#[test]
fn a_2_0_comparison_bundle_opens_verified_with_tiles_and_symbols() {
    let opened = open_file(&fixture("Bundles/dotnet-geometry-compare.scbundle")).unwrap();
    assert!(opened.verified);
    assert_eq!(opened.review.kind, "findings");
    // 2.0 writes delta ids in camelCase; each still maps to its symbol.
    for symbol in ["ΔF", "ΔTd", "ΔĀ", "ΔO"] {
        assert!(opened.review.fired.contains(&symbol.to_string()), "{symbol}: {:?}", opened.review.fired);
    }
    assert_eq!(opened.review.explanations.len(), 4);
    assert!(opened.review.explanations.iter().all(|tile| tile.why.contains("2.0 stored")));
}

/// The golden pair sealed with a fixed time. The committed file must match, so a change to what
/// this review writes shows up here; SCALARSCOPE_WRITE_BUNDLE_FIXTURES=1 rewrites it.
#[test]
fn the_bundle_2_0_reads_back_is_the_one_this_review_writes() {
    let read = |name: &str| fs::read_to_string(fixture("InferenceOptimization").join(name)).unwrap();
    let left = open_text(&read("baseline_tfrt_runtrace.json"), "baseline").unwrap();
    let right = open_text(&read("optimized_tfrt_runtrace.json"), "optimized").unwrap();
    let sealed = seal(&document_from_pair(&pair(&left, &right).unwrap()), "2026-10-06T00:00:00Z").unwrap();
    assert!(verify(&sealed.bytes).unwrap().valid);
    let path = fixture("Bundles/rust-inference-review.scbundle");
    if std::env::var("SCALARSCOPE_WRITE_BUNDLE_FIXTURES").as_deref() == Ok("1") {
        fs::write(&path, &sealed.bytes).unwrap();
    }
    let committed = fs::read(&path).expect("tests/Fixtures/Bundles/rust-inference-review.scbundle");
    assert!(committed == sealed.bytes, "the review writes different bytes now; rewrite the fixture and rerun the 2.0 import test");
}

#[test]
fn a_geometry_review_has_2_0_deltas_as_tiles_and_round_trips_through_a_bundle() {
    use scalarscope::review::Pair;
    let sample = |name: &str| fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/ScalarScope/Resources/Raw/Samples").join(name)).unwrap();
    let left = open_text(&sample("orthogonal_professors.json"), "a").unwrap();
    let right = open_text(&sample("correlated_professors.json"), "b").unwrap();
    let built = pair(&left, &right).unwrap();
    let Pair::Geometry(review) = &built else { panic!("geometry") };
    let fired: Vec<&str> = review.explanations.iter().filter(|tile| tile.status == "fired").map(|tile| tile.symbol.as_str()).collect();
    assert_eq!(fired.len(), 4, "{fired:?}");
    assert!(review.verdict.starts_with("Only Path A experienced correctness_dip"), "{}", review.verdict);
    let sealed = seal(&document_from_pair(&built), "2026-10-06T00:00:00Z").unwrap();
    let path = std::env::temp_dir().join(format!("scalarscope-geometry-{}.scbundle", std::process::id()));
    fs::write(&path, &sealed.bytes).unwrap();
    let opened = open_file(&path).unwrap();
    assert!(opened.verified);
    assert_eq!(opened.review.fired.len(), 4);
    let Some(Pair::Geometry(again)) = scalarscope::bundle::stored_pair(&opened.review) else { panic!("stored geometry") };
    assert_eq!(again.verdict, review.verdict);
    assert_eq!(again.deltas, review.deltas);
}
