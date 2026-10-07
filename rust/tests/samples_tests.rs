//! The files in `samples/`, which TESTING.md and the Store certification notes name. Each must be
//! the fixture it was copied from, byte for byte, and must open as TESTING.md says.

use std::path::{Path, PathBuf};

use scalarscope::open::{open_path, Side};
use scalarscope::review::{pair, Pair};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn same(sample: &str, fixture: &str) {
    let (a, b) = (std::fs::read(repo().join(sample)).unwrap(), std::fs::read(repo().join(fixture)).unwrap());
    // Line endings may differ by checkout; the content may not.
    let strip = |bytes: Vec<u8>| bytes.into_iter().filter(|byte| *byte != b'\r').collect::<Vec<u8>>();
    assert!(strip(a) == strip(b), "{sample} is no longer the same as {fixture}");
}

fn side(path: &Path) -> Side {
    open_path(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())).side
}

#[test]
fn every_sample_is_its_fixture() {
    same("samples/inference/baseline.runtrace.json", "tests/Fixtures/InferenceOptimization/baseline_tfrt_runtrace.json");
    same("samples/inference/optimized.runtrace.json", "tests/Fixtures/InferenceOptimization/optimized_tfrt_runtrace.json");
    for name in ["local-teacher", "composite-teacher", "local-teacher.drift", "composite-teacher.drift"] {
        same(&format!("samples/geometry/{name}.geometry.json"), &format!("rust/tests/fixtures/aspire-si/real-{name}.geometry.json"));
    }
    let mut count = 0;
    for entry in std::fs::read_dir(repo().join("samples/workbench")).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        same(&format!("samples/workbench/{name}"), &format!("rust/tests/fixtures/workbench/{name}"));
        count += 1;
    }
    assert_eq!(count, 9);
}

#[test]
fn the_samples_open_as_testing_md_says() {
    let inference = pair(&side(&repo().join("samples/inference/baseline.runtrace.json")), &side(&repo().join("samples/inference/optimized.runtrace.json"))).unwrap();
    let Pair::Inference(review) = inference else { panic!("inference") };
    assert!(!review.headline.is_empty());

    let geometry = pair(
        &side(&repo().join("samples/geometry/local-teacher.drift.geometry.json")),
        &side(&repo().join("samples/geometry/composite-teacher.drift.geometry.json")),
    )
    .unwrap();
    let Pair::Geometry(review) = geometry else { panic!("geometry") };
    let status = |symbol: &str| review.explanations.iter().find(|tile| tile.symbol == symbol).unwrap().status.clone();
    for symbol in ["ΔF", "ΔTc", "ΔO"] {
        assert_eq!(status(symbol), "withheld", "{symbol}");
    }
    assert_eq!(status("Δ\u{0100}"), "fired");

    for entry in std::fs::read_dir(repo().join("samples/workbench")).unwrap().flatten() {
        let Side::Inference(run) = side(&entry.path()) else { panic!("inference") };
        assert!(run.knobs.values.contains_key("batch"));
    }
}
