//! A `.scbundle` is the Phase 7.2 archive.
//!
//! `BundleIntegrityService.ComputeBundleHash` sorts paths with ordinal
//! comparison, writes `path:hash\n` for each file, and hashes that UTF-8
//! string with SHA-256. `integrity.json` is the seal. It is not in the
//! preimage. The bytes that are hashed are the bytes stored in the zip.
//! A matching hash checks those bytes. It is not a signature.
//!
//! Reopening reads `review/review.json`: its verdict and tiles are what the page shows. A geometry
//! review also redraws from its stored runs, and today's reading is shown apart (`stored_pair`).
//! Numbers written into the archive are finite, and a zero is positive zero.
//! The hash is of those UTF-8 bytes. It is not a hash of float bits, and it
//! does not call a transcendental.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::readings::Band;
use crate::review::{Finding, InferenceReview, Pair, TrainingReview};

pub const CONTENT_CHECK: &str = "A matching hash checks these bytes. It is not a signature.";

const SPEC: &str = "1.0.0";
const APP: &str = env!("CARGO_PKG_VERSION");
const REVIEW_PATH: &str = "review/review.json";

#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredReview {
    pub kind: String,
    pub verdict: String,
    pub fired: Vec<String>,
    pub caption: String,
    pub left_text: String,
    pub right_text: String,
    pub findings: Vec<Finding>,
    #[serde(default)]
    pub inference: Option<StoredSeries>,
    #[serde(default)]
    pub training: Option<StoredTraining>,
    /// Fingerprint, validation and guardrail notes. Left out of the file when empty, so a
    /// review without them keeps the bytes and hash it had before this field existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<String>,
    /// The ratio headline. Left out when empty, like `notices`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub headline: String,
    /// The delta tiles. Left out when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explanations: Vec<crate::review::Explanation>,
    /// Both geometry exports, for a geometry review. Left out otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<StoredGeometry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
pub struct StoredGeometry {
    pub left: crate::geometry::GeometryRun,
    pub right: crate::geometry::GeometryRun,
}

#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSeries {
    pub left_label: String,
    pub right_label: String,
    pub signal: String,
    pub unit: String,
    pub left: Vec<Option<f64>>,
    pub right: Vec<Option<f64>>,
    pub left_band: Vec<Option<StoredBand>>,
    pub right_band: Vec<Option<StoredBand>>,
    pub left_marks: Vec<usize>,
    pub right_marks: Vec<usize>,
    pub left_steady: Option<usize>,
    pub right_steady: Option<usize>,
    pub left_throughput: Vec<f64>,
    pub right_throughput: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub difference: Vec<crate::views::DifferencePoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_lead: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_lead: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_settle: Option<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_settle: Option<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_segments: Vec<crate::views::Segment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_segments: Vec<crate::views::Segment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_elapsed: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_elapsed: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_cpu: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_cpu: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_gpu: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_gpu: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_memory: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_memory: Vec<f64>,
    pub left_cdf: Vec<[f64; 2]>,
    pub right_cdf: Vec<[f64; 2]>,
    pub left_p50: Option<f64>,
    pub left_p95: Option<f64>,
    pub left_p99: Option<f64>,
    pub right_p50: Option<f64>,
    pub right_p95: Option<f64>,
    pub right_p99: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, serde::Deserialize)]
pub struct StoredBand {
    pub low: f64,
    pub high: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredTraining {
    pub left_run_id: String,
    pub right_run_id: String,
    pub left_loss: Vec<f64>,
    pub right_loss: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BundleDocument {
    pub review: StoredReview,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sealed {
    pub bytes: Vec<u8>,
    pub bundle_hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Verification {
    pub valid: bool,
    pub actual_bundle_hash: String,
    pub stated_bundle_hash: String,
    pub codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpenedBundle {
    pub bundle_hash: String,
    pub review: StoredReview,
    /// False for a 2.0 review-only bundle: it has no integrity.json, so its bytes cannot be checked,
    /// and `bundle_hash` is the hash its JSON states.
    pub verified: bool,
}

#[derive(Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Integrity {
    file_hashes: BTreeMap<String, String>,
    bundle_hash: String,
    computed_at: String,
}

pub fn document_from_pair(pair: &Pair) -> BundleDocument {
    let review = match pair {
        Pair::Inference(review) => inference_review(review),
        Pair::Training(review) => training_review(review),
        Pair::Geometry(review) => geometry_document(review),
    };
    BundleDocument { review }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 0x0f) as usize] as char);
    }
    text
}

/// Ordinal `path:hash\n` lines, then SHA-256 of that UTF-8 string.
/// This is `BundleIntegrityService.ComputeBundleHash`, not the v1 preimage.
pub fn bundle_hash(files: &BTreeMap<String, String>) -> String {
    let mut paths: Vec<&String> = files.keys().collect();
    paths.sort_by(|left, right| ordinal_cmp(left, right));
    let mut combined = String::new();
    for path in paths {
        combined.push_str(path);
        combined.push(':');
        combined.push_str(&files[path]);
        combined.push('\n');
    }
    sha256_hex(combined.as_bytes())
}

pub fn seal(document: &BundleDocument, created_at: &str) -> Result<Sealed, String> {
    if created_at.is_empty() || !created_at.is_ascii() || created_at.chars().any(|character| character.is_control()) {
        return Err("The bundle timestamp has to be a plain UTC stamp.".to_string());
    }
    let review_bytes = to_json(&document.review)?;
    let deltas: Vec<DeltaRow> = document
        .review
        .findings
        .iter()
        .map(|row| DeltaRow {
            id: &row.id,
            name: &row.name,
            explanation: &row.sentence,
            summary_sentence: &row.sentence,
            delta_type: &row.kind,
            status: "Present",
            left_value: row.left,
            right_value: row.right,
            delta: row.delta,
            magnitude: stored_abs(row.delta),
            units: &row.units,
            is_meaningful: true,
        })
        .collect();
    let deltas_bytes = to_json(&deltas)?;
    let why: Vec<WhyRow> = document
        .review
        .findings
        .iter()
        .map(|row| WhyRow {
            delta_id: &row.id,
            delta_name: &row.name,
            explanation: &row.sentence,
            summary_sentence: &row.sentence,
        })
        .collect();
    let why_bytes = to_json(&why)?;
    let bundle_id = bundle_id(&document.review);
    let summary = summary_markdown(&bundle_id, created_at, &document.review);
    let samples = sample_count(&document.review);
    let repro = Repro {
        input_fingerprint: &sha256_hex(&review_bytes),
        delta_hash: &sha256_hex(&deltas_bytes),
        determinism_enabled: true,
        delta_spec_version: SPEC,
        reproducibility_badge: "Content check",
        alignment_mode: "stored-window",
        timestep_count: samples,
    };
    let repro_bytes = to_json(&repro)?;
    let environment = Environment {
        app_version: APP,
        platform: "Windows",
        dot_net_version: "rust",
        is64_bit_process: cfg!(target_pointer_width = "64"),
        processor_count: 0,
    };
    let environment_bytes = to_json(&environment)?;
    let manifest = Manifest {
        bundle_id: &bundle_id,
        bundle_version: SPEC,
        profile: "Review",
        created_at,
        app_version: APP,
        delta_spec_version: SPEC,
        privacy: Privacy {
            includes_raw_data: document.review.inference.is_some() || document.review.training.is_some(),
            includes_file_paths: false,
            includes_machine_name: false,
            data_classification: "local-review",
        },
        contents: Contents {
            includes_findings: true,
            includes_repro: true,
            includes_environment: true,
            includes_insights: false,
            includes_assets: false,
            includes_audit: false,
            includes_evidence: false,
        },
    };
    let manifest_bytes = to_json(&manifest)?;
    let readme = readme_markdown(&bundle_id);
    let mut files = BTreeMap::new();
    files.insert("manifest.json".to_string(), manifest_bytes);
    files.insert("findings/deltas.json".to_string(), deltas_bytes);
    files.insert("findings/why.json".to_string(), why_bytes);
    files.insert("findings/summary.md".to_string(), summary.into_bytes());
    files.insert("repro/repro.json".to_string(), repro_bytes);
    files.insert("environment/environment.json".to_string(), environment_bytes);
    files.insert("README.md".to_string(), readme.into_bytes());
    files.insert(REVIEW_PATH.to_string(), review_bytes);
    let mut hashes = BTreeMap::new();
    for (path, bytes) in &files {
        hashes.insert(path.clone(), sha256_hex(bytes));
    }
    let bundle_hash = bundle_hash(&hashes);
    let integrity = Integrity {
        file_hashes: hashes,
        bundle_hash: bundle_hash.clone(),
        computed_at: created_at.to_string(),
    };
    let integrity_bytes = to_json(&integrity)?;
    files.insert("integrity.json".to_string(), integrity_bytes);
    let entries: Vec<(String, Vec<u8>)> = files.into_iter().collect();
    let bytes = pack(&entries)?;
    Ok(Sealed { bytes, bundle_hash })
}

pub fn verify(bytes: &[u8]) -> Result<Verification, String> {
    let entries = unpack(bytes)?;
    let mut verification = Verification {
        valid: false,
        actual_bundle_hash: String::new(),
        stated_bundle_hash: String::new(),
        codes: Vec::new(),
    };
    let Some(integrity_bytes) = entries.iter().find(|(path, _)| path.eq_ignore_ascii_case("integrity.json")).map(|(_, data)| data) else {
        verification.codes.push("MissingIntegrity".to_string());
        return Ok(verification);
    };
    let integrity: Integrity = match serde_json::from_slice(integrity_bytes) {
        Ok(parsed) => parsed,
        Err(_) => {
            verification.codes.push("InvalidIntegrity".to_string());
            return Ok(verification);
        }
    };
    verification.stated_bundle_hash = integrity.bundle_hash.clone();
    let mut listed = BTreeMap::new();
    for (path, expected) in integrity.file_hashes {
        let key = match normalize(&path) {
            Ok(key) => key,
            Err(_) => {
                verification.codes.push("BadPath".to_string());
                continue;
            }
        };
        if key.eq_ignore_ascii_case("integrity.json") {
            verification.codes.push("SealedFileListed".to_string());
            continue;
        }
        if listed.insert(key.clone(), expected).is_some() {
            verification.codes.push("DuplicateEntry".to_string());
        }
    }

    let mut actual = BTreeMap::new();
    for (path, data) in &entries {
        if path.is_empty() || path.ends_with('/') || path.eq_ignore_ascii_case("integrity.json") {
            continue;
        }
        if actual.contains_key(path) {
            verification.codes.push("DuplicateEntry".to_string());
            continue;
        }
        actual.insert(path.clone(), sha256_hex(data));
    }

    let mut missing = false;
    let mut modified = false;
    let mut undeclared = false;
    for path in listed.keys() {
        if !actual.contains_key(path) {
            missing = true;
        }
    }
    for (path, hash) in &actual {
        match listed.get(path) {
            None => undeclared = true,
            Some(expected) if !expected.eq_ignore_ascii_case(hash) => modified = true,
            Some(_) => {}
        }
    }
    if missing {
        verification.codes.push("MissingFiles".to_string());
    }
    if modified {
        verification.codes.push("ModifiedFiles".to_string());
    }
    if undeclared {
        verification.codes.push("UndeclaredFile".to_string());
    }
    verification.actual_bundle_hash = bundle_hash(&actual);
    if !verification.actual_bundle_hash.eq_ignore_ascii_case(&integrity.bundle_hash) {
        verification.codes.push("BundleHashMismatch".to_string());
    }
    verification.valid = verification.codes.is_empty();
    Ok(verification)
}

pub fn open_bytes(bytes: &[u8]) -> Result<OpenedBundle, String> {
    let verification = verify(bytes)?;
    if !verification.valid && verification.codes == ["MissingIntegrity"] {
        // 2.0 auto-saved an inference review as review/review.json alone, with no integrity.json.
        let entries = unpack(bytes)?;
        if let Some(opened) = dotnet_review(&entries) {
            return opened;
        }
    }
    if !verification.valid {
        return Err(format!(
            "The bundle hash does not match these bytes ({}).",
            verification.codes.join(", ")
        ));
    }
    let entries = unpack(bytes)?;
    let review = if let Some((_, data)) = entries.iter().find(|(path, _)| path == REVIEW_PATH) {
        serde_json::from_slice(data).map_err(|_| "The stored review did not parse.".to_string())?
    } else {
        findings_only(&entries)?
    };
    Ok(OpenedBundle {
        bundle_hash: verification.actual_bundle_hash,
        review,
        verified: true,
    })
}

/// A 2.0 review-only bundle: `review/review.json` holding a `ReviewOnlyBundle` (version, bundleHash,
/// comparison with its deltas). It keeps the deltas, not the samples, so there is no chart, and
/// without integrity.json nothing checks its bytes. None when the archive is not that.
/// A run's label, or the side's letter when the file has none.
fn or_side(label: &str, side: &str) -> String {
    if label.trim().is_empty() { side.to_string() } else { label.to_string() }
}

fn dotnet_review(entries: &[(String, Vec<u8>)]) -> Option<Result<OpenedBundle, String>> {
    let [(path, data)] = entries else {
        return None;
    };
    if path != REVIEW_PATH {
        return None;
    }
    let value: Value = serde_json::from_slice(data).ok()?;
    let comparison = value.get("comparison")?;
    value.get("version")?;
    let text = |item: &Value, key: &str| item.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let number = |item: &Value, key: &str| item.get(key).and_then(Value::as_f64);
    let intent = comparison.get("intent").cloned().unwrap_or(Value::Null);
    let (label_a, label_b) = (text(&intent, "labelA"), text(&intent, "labelB"));
    let mut explanations = Vec::new();
    let mut fired = Vec::new();
    let mut sentences = Vec::new();
    for delta in comparison.get("deltas").and_then(Value::as_array).into_iter().flatten() {
        let symbol = text(delta, "deltaType");
        let is_fired = delta.get("fired").and_then(Value::as_bool).unwrap_or(false);
        let suppressed = delta.get("isSuppressed").and_then(Value::as_bool).unwrap_or(false);
        let headline = Some(text(delta, "interpretation"))
            .filter(|line| !line.is_empty())
            .or_else(|| Some(text(delta, "suppressionReason")).filter(|line| !line.is_empty()))
            .unwrap_or_else(|| if suppressed { "Suppressed".to_string() } else { "Did not fire".to_string() });
        if is_fired {
            fired.push(symbol.clone());
            sentences.push(format!("{symbol} {headline}"));
        }
        let shown = |value: Option<f64>| value.map_or("none".to_string(), |value| format!("{value}"));
        explanations.push(crate::review::Explanation {
            symbol: symbol.clone(),
            status: if is_fired { "fired" } else if suppressed { "withheld" } else { "quiet" }.to_string(),
            headline,
            why: format!(
                "{} This is the finding 2.0 stored; 3.0 does not recompute it, and its rules differ (see the Guide).",
                text(delta, "notes")
            ),
            parameters: vec![
                [or_side(&label_a, "A"), shown(number(delta, "valueA"))],
                [or_side(&label_b, "B"), shown(number(delta, "valueB"))],
                ["Difference".to_string(), shown(number(delta, "absoluteDifference"))],
                ["2.0 confidence".to_string(), shown(number(delta, "confidence"))],
            ],
            anchor: None,
        });
    }
    let stated = text(&value, "bundleHash");
    let review = StoredReview {
        kind: "dotnet-review".to_string(),
        geometry: None,
        verdict: if sentences.is_empty() { "No delta fired in this 2.0 review.".to_string() } else { sentences.join(" ") },
        fired,
        caption: "A 2.0 inference review. It stores the deltas, not the samples, so there is no chart, and 2.0 wrote no integrity.json, so its bytes cannot be checked. Open the two runs to review them in 3.0.".to_string(),
        left_text: label_a,
        right_text: label_b,
        findings: Vec::new(),
        inference: None,
        training: None,
        notices: comparison.get("warnings").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect(),
        headline: String::new(),
        explanations,
    };
    Some(Ok(OpenedBundle {
        bundle_hash: stated,
        review,
        verified: false,
    }))
}

pub fn write_file(path: &Path, sealed: &Sealed) -> Result<(), String> {
    fs::write(path, &sealed.bytes).map_err(|error| format!("Could not write the bundle. {error}"))
}

pub fn open_file(path: &Path) -> Result<OpenedBundle, String> {
    let bytes = fs::read(path).map_err(|error| format!("Could not read the bundle. {error}"))?;
    open_bytes(&bytes)
}

/// Copies the stored drawing. It does not run the detectors.
pub fn stored_pair(review: &StoredReview) -> Option<Pair> {
    match review.kind.as_str() {
        "inference" => {
            let series = review.inference.as_ref()?;
            Some(Pair::Inference(InferenceReview {
                left_label: series.left_label.clone(),
                right_label: series.right_label.clone(),
                signal: series.signal.clone(),
                unit: series.unit.clone(),
                left: series.left.clone(),
                right: series.right.clone(),
                left_band: bands(&series.left_band),
                right_band: bands(&series.right_band),
                left_marks: series.left_marks.clone(),
                right_marks: series.right_marks.clone(),
                left_steady: series.left_steady,
                right_steady: series.right_steady,
                left_throughput: series.left_throughput.clone(),
                right_throughput: series.right_throughput.clone(),
                left_memory: series.left_memory.clone(),
                left_elapsed: series.left_elapsed.clone(),
                right_elapsed: series.right_elapsed.clone(),
                left_cpu: series.left_cpu.clone(),
                right_cpu: series.right_cpu.clone(),
                left_gpu: series.left_gpu.clone(),
                right_gpu: series.right_gpu.clone(),
                difference: series.difference.clone(),
                left_segments: series.left_segments.clone(),
                left_lead: series.left_lead.clone(),
                right_lead: series.right_lead.clone(),
                left_settle: series.left_settle,
                right_settle: series.right_settle,
                right_segments: series.right_segments.clone(),
                right_memory: series.right_memory.clone(),
                left_cdf: pairs(&series.left_cdf),
                right_cdf: pairs(&series.right_cdf),
                left_p50: series.left_p50,
                left_p95: series.left_p95,
                left_p99: series.left_p99,
                right_p50: series.right_p50,
                right_p95: series.right_p95,
                right_p99: series.right_p99,
                fired: review.fired.clone(),
                findings: review.findings.clone(),
                verdict: review.verdict.clone(),
                caption: review.caption.clone(),
                left_text: review.left_text.clone(),
                right_text: review.right_text.clone(),
                notices: review.notices.clone(),
                headline: review.headline.clone(),
                explanations: review.explanations.clone(),
            }))
        }
        "geometry" => {
            let stored = review.geometry.as_ref()?;
            // The runs are stored in full, so the views redraw from them. The verdict and the tiles
            // are the stored ones, which the hash vouches for. The deltas are recomputed by today's
            // rules, which can read differently from the rules the bundle was saved under; when
            // they do, the page shows today's verdict below the stored one, labelled.
            let mut current = crate::review::geometry_with_deltas(
                review.left_text.clone(),
                review.right_text.clone(),
                stored.left.clone(),
                stored.right.clone(),
                review.notices.clone(),
            );
            if current.verdict != review.verdict {
                current.current_verdict = Some(std::mem::replace(&mut current.verdict, review.verdict.clone()));
            }
            if review.explanations.is_empty() {
                current.tiles_are_current = true;
            } else {
                current.explanations = review.explanations.clone();
            }
            Some(Pair::Geometry(current))
        }
        "training" => {
            let training = review.training.as_ref()?;
            Some(Pair::Training(TrainingReview {
                left_text: review.left_text.clone(),
                right_text: review.right_text.clone(),
                caption: review.caption.clone(),
                left: training_entry(&training.left_run_id, &training.left_loss),
                right: training_entry(&training.right_run_id, &training.right_loss),
            }))
        }
        _ => None,
    }
}

pub fn pack(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let mut seen = BTreeMap::<String, ()>::new();
    for (path, bytes) in entries {
        let path = normalize(path)?;
        if path.is_empty() || path.ends_with('/') {
            return Err(format!("A bundle path has to name a file: {path}"));
        }
        if seen.insert(path.clone(), ()).is_some() {
            return Err(format!("Duplicate bundle entry: {path}"));
        }
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .start_file(path, options)
            .map_err(|error| format!("Could not write the bundle entry. {error}"))?;
        writer
            .write_all(bytes)
            .map_err(|error| format!("Could not write the bundle entry. {error}"))?;
    }
    let cursor = writer.finish().map_err(|error| format!("Could not finish the bundle. {error}"))?;
    Ok(cursor.into_inner())
}

pub fn unpack(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| format!("This is not a bundle archive. {error}"))?;
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("Could not read a bundle entry. {error}"))?;
        let path = normalize(file.name())?;
        if path.is_empty() || path.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|error| format!("Could not read a bundle entry. {error}"))?;
        entries.push((path, data));
    }
    Ok(entries)
}

pub fn utc_stamp(unix_seconds: u64) -> String {
    let days = (unix_seconds / 86_400) as i64;
    let time_of_day = (unix_seconds % 86_400) as u32;
    let hour = time_of_day / 3_600;
    let minute = (time_of_day % 3_600) / 60;
    let second = time_of_day % 60;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = if month_part < 10 { month_part + 3 } else { month_part - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub fn utc_now() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    utc_stamp(seconds)
}

fn inference_review(review: &InferenceReview) -> StoredReview {
    let labels = [review.left_label.as_str(), review.right_label.as_str()];
    StoredReview {
        kind: "inference".to_string(),
        geometry: None,
        notices: review.notices.iter().map(|line| scrub(line, &labels)).collect(),
        headline: review.headline.clone(),
        // A tile's text can name the runs, and a run's label can carry a folder.
        explanations: review
            .explanations
            .iter()
            .map(|tile| crate::review::Explanation {
                headline: scrub(&tile.headline, &labels),
                why: scrub(&tile.why, &labels),
                parameters: tile.parameters.iter().map(|[name, value]| [name.clone(), scrub(value, &labels)]).collect(),
                ..tile.clone()
            })
            .collect(),
        verdict: scrub(&review.verdict, &labels),
        fired: review.fired.clone(),
        caption: scrub(&review.caption, &labels),
        left_text: scrub(&review.left_text, &labels),
        right_text: scrub(&review.right_text, &labels),
        findings: review.findings.clone(),
        inference: Some(StoredSeries {
            left_label: public_label(&review.left_label),
            right_label: public_label(&review.right_label),
            signal: review.signal.clone(),
            unit: review.unit.clone(),
            left: review.left.clone(),
            right: review.right.clone(),
            left_band: store_bands(&review.left_band),
            right_band: store_bands(&review.right_band),
            left_marks: review.left_marks.clone(),
            right_marks: review.right_marks.clone(),
            left_steady: review.left_steady,
            right_steady: review.right_steady,
            left_throughput: review.left_throughput.clone(),
            right_throughput: review.right_throughput.clone(),
            left_memory: review.left_memory.clone(),
            left_elapsed: review.left_elapsed.clone(),
            right_elapsed: review.right_elapsed.clone(),
            left_cpu: review.left_cpu.clone(),
            right_cpu: review.right_cpu.clone(),
            left_gpu: review.left_gpu.clone(),
            right_gpu: review.right_gpu.clone(),
            difference: review.difference.clone(),
            left_segments: review.left_segments.clone(),
            left_lead: review.left_lead.clone(),
            right_lead: review.right_lead.clone(),
            left_settle: review.left_settle,
            right_settle: review.right_settle,
            right_segments: review.right_segments.clone(),
            right_memory: review.right_memory.clone(),
            left_cdf: store_pairs(&review.left_cdf),
            right_cdf: store_pairs(&review.right_cdf),
            left_p50: review.left_p50,
            left_p95: review.left_p95,
            left_p99: review.left_p99,
            right_p50: review.right_p50,
            right_p95: review.right_p95,
            right_p99: review.right_p99,
        }),
        training: None,
    }
}

fn geometry_document(review: &crate::review::GeometryReview) -> StoredReview {
    use crate::geometry_deltas::DeltaStatus;
    let present: Vec<&crate::geometry_deltas::GeometryDelta> = review.deltas.iter().filter(|delta| delta.status == DeltaStatus::Present).collect();
    StoredReview {
        kind: "geometry".to_string(),
        verdict: review.verdict.clone(),
        fired: present.iter().map(|delta| delta.symbol.clone()).collect(),
        caption: "Two ASPIRE training-dynamics runs, stored in full so the review redraws exactly.".to_string(),
        left_text: review.left_label.clone(),
        right_text: review.right_label.clone(),
        // 2.0's deltas.json rows, so a 2.0 reader sees the same findings.
        findings: present
            .iter()
            .map(|delta| crate::review::Finding {
                symbol: delta.symbol.clone(),
                id: delta.id.clone(),
                name: delta.name.clone(),
                kind: format!("{:?}", delta.delta_type),
                sentence: delta.summary_sentence.clone().filter(|line| !line.is_empty()).unwrap_or_else(|| delta.explanation.clone()),
                left: delta.left_value,
                right: delta.right_value,
                delta: delta.delta,
                units: delta.units.clone().unwrap_or_default(),
            })
            .collect(),
        inference: None,
        training: None,
        notices: review.warnings.clone(),
        headline: String::new(),
        explanations: review.explanations.clone(),
        geometry: Some(StoredGeometry { left: review.left.clone(), right: review.right.clone() }),
    }
}

fn training_review(review: &TrainingReview) -> StoredReview {
    let labels = [review.left.run_id.as_str(), review.right.run_id.as_str()];
    StoredReview {
        kind: "training".to_string(),
        notices: Vec::new(),
        headline: String::new(),
        explanations: Vec::new(),
        geometry: None,
        verdict: String::new(),
        fired: Vec::new(),
        caption: scrub(&review.caption, &labels),
        left_text: scrub(&review.left_text, &labels),
        right_text: scrub(&review.right_text, &labels),
        findings: Vec::new(),
        inference: None,
        training: Some(StoredTraining {
            left_run_id: public_label(&review.left.run_id),
            right_run_id: public_label(&review.right.run_id),
            left_loss: review.left.loss.clone(),
            right_loss: review.right.loss.clone(),
        }),
    }
}

fn findings_only(entries: &[(String, Vec<u8>)]) -> Result<StoredReview, String> {
    let Some((_, data)) = entries.iter().find(|(path, _)| path == "findings/deltas.json") else {
        return Err("This bundle has no stored review.".to_string());
    };
    let deltas: Vec<Value> = serde_json::from_slice(data).map_err(|_| "The stored findings did not parse.".to_string())?;
    let whys: Vec<Value> = entries
        .iter()
        .find(|(path, _)| path == "findings/why.json")
        .and_then(|(_, data)| serde_json::from_slice(data).ok())
        .unwrap_or_default();
    let mut fired = Vec::new();
    let mut sentences = Vec::new();
    let mut explanations = Vec::new();
    for delta in &deltas {
        let id = delta.get("id").and_then(Value::as_str).unwrap_or("");
        let status = delta.get("status").and_then(Value::as_str).unwrap_or("");
        let why = whys.iter().find(|row| row.get("deltaId").and_then(Value::as_str).is_some_and(|key| key.eq_ignore_ascii_case(id)));
        let line = |item: Option<&Value>, key: &str| item.and_then(|item| item.get(key)).and_then(Value::as_str).unwrap_or("").to_string();
        let headline = [line(Some(delta), "summarySentence"), line(Some(delta), "explanation"), line(Some(delta), "name")]
            .into_iter()
            .find(|text| !text.is_empty())
            .unwrap_or_else(|| id.to_string());
        let mut parameters: Vec<[String; 2]> = why
            .and_then(|row| row.get("parameters"))
            .and_then(Value::as_object)
            .map(|map| map.iter().map(|(key, value)| [key.clone(), value.to_string().trim_matches('"').to_string()]).collect())
            .unwrap_or_default();
        if let Some(confidence) = delta.get("confidence").and_then(Value::as_f64) {
            parameters.push(["2.0 confidence".to_string(), format!("{confidence:.2}")]);
        }
        explanations.push(crate::review::Explanation {
            symbol: symbol_for(id).unwrap_or(id).to_string(),
            status: match status {
                "Present" => "fired",
                "Suppressed" => "withheld",
                _ => "quiet",
            }
            .to_string(),
            headline,
            why: [line(why, "explanation"), "This is the finding 2.0 stored; 3.0 does not recompute it.".to_string()]
                .into_iter()
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            parameters,
            anchor: None,
        });
    }
    for delta in deltas {
        let status = delta.get("status").and_then(Value::as_str).unwrap_or("");
        if status != "Present" {
            continue;
        }
        let id = delta.get("id").and_then(Value::as_str).unwrap_or("");
        let sentence = delta
            .get("summarySentence")
            .and_then(Value::as_str)
            .or_else(|| delta.get("explanation").and_then(Value::as_str))
            .unwrap_or("");
        if sentence.is_empty() {
            continue;
        }
        if let Some(symbol) = symbol_for(id) {
            fired.push(symbol.to_string());
            sentences.push(format!("{symbol} {sentence}"));
        } else {
            sentences.push(sentence.to_string());
        }
    }
    let verdict = if sentences.is_empty() {
        "No stored finding was present.".to_string()
    } else {
        sentences.join(" ")
    };
    Ok(StoredReview {
        kind: "findings".to_string(),
        geometry: None,
        notices: Vec::new(),
        headline: String::new(),
        explanations,
        verdict,
        fired,
        caption: "This bundle stores the findings. It does not store the series, so the chart is not drawn from another file.".to_string(),
        left_text: String::new(),
        right_text: String::new(),
        findings: Vec::new(),
        inference: None,
        training: None,
    })
}

/// The delta symbol for a stored id. 2.0 writes ids in camelCase (`failurePresence`); match either case.
fn symbol_for(id: &str) -> Option<&'static str> {
    match id.to_ascii_lowercase().as_str() {
        "failurepresence" => Some("ΔF"),
        "convergencetiming" => Some("ΔTc"),
        "stabilityoscillation" => Some("ΔO"),
        "structuralemergence" => Some("ΔTd"),
        "evaluatoralignment" => Some("ΔĀ"),
        _ => None,
    }
}

fn to_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut value = serde_json::to_value(value).map_err(|error| format!("Could not write the bundle JSON. {error}"))?;
    canon_json(&mut value)?;
    let mut bytes = serde_json::to_vec_pretty(&value).map_err(|error| format!("Could not write the bundle JSON. {error}"))?;
    if bytes.first() == Some(&0xef) {
        return Err("Bundle JSON must not start with a byte-order mark.".to_string());
    }
    bytes.push(b'\n');
    Ok(bytes)
}

fn canon_json(value: &mut Value) -> Result<(), String> {
    if let Some(parsed) = value.as_f64() {
        if !parsed.is_finite() {
            return Err("A bundle number has to be finite.".to_string());
        }
        if parsed == 0.0 && parsed.is_sign_negative() {
            *value = Value::Number(Number::from_f64(0.0).expect("zero is a JSON number"));
        }
        return Ok(());
    }
    match value {
        Value::Array(items) => {
            for item in items {
                canon_json(item)?;
            }
            Ok(())
        }
        Value::Object(map) => canon_map(map),
        _ => Ok(()),
    }
}

fn canon_map(map: &mut Map<String, Value>) -> Result<(), String> {
    for value in map.values_mut() {
        canon_json(value)?;
    }
    Ok(())
}

fn summary_markdown(bundle_id: &str, created_at: &str, review: &StoredReview) -> String {
    let mut text = String::new();
    text.push_str("# Comparison Summary\n\n");
    text.push_str(&format!("**Bundle ID:** `{bundle_id}`\n"));
    text.push_str(&format!("**Created:** {created_at}\n"));
    text.push_str("**Profile:** Review\n");
    text.push_str(&format!("**App Version:** {APP}\n\n"));
    text.push_str("## Stored review\n\n");
    if !review.verdict.is_empty() {
        text.push_str(&review.verdict);
        text.push_str("\n\n");
    }
    if !review.caption.is_empty() {
        text.push_str(&review.caption);
        text.push_str("\n\n");
    }
    if !review.left_text.is_empty() {
        text.push_str(&review.left_text);
        text.push('\n');
    }
    if !review.right_text.is_empty() {
        text.push_str(&review.right_text);
        text.push('\n');
    }
    text.push_str("\n## Deltas found\n\n");
    if review.findings.is_empty() {
        text.push_str("No finding was stored on this page.\n\n");
    } else {
        for row in &review.findings {
            text.push_str(&format!("- {} {}\n", row.symbol, row.sentence));
        }
        text.push('\n');
    }
    text.push_str("This review does not report a confidence number.\n\n");
    text.push_str(CONTENT_CHECK);
    text.push_str("\n\n---\n*Generated by ScalarScope*\n");
    text
}

fn readme_markdown(bundle_id: &str) -> String {
    format!(
        "# ScalarScope Comparison Bundle\n\n\
         This bundle was exported from ScalarScope {APP}.\n\n\
         Open it in ScalarScope. The review on the page is the review that was stored.\n\n\
         File paths and the machine name are not included.\n\n\
         {CONTENT_CHECK}\n\n\
         ---\n*Bundle ID: {bundle_id}*\n"
    )
}

fn bundle_id(review: &StoredReview) -> String {
    let basis = format!("{}\n{}\n{}", review.kind, review.verdict, review.left_text);
    sha256_hex(basis.as_bytes())[..16].to_string()
}

fn sample_count(review: &StoredReview) -> usize {
    if let Some(series) = &review.inference {
        return series.left.len().max(series.right.len());
    }
    if let Some(training) = &review.training {
        return training.left_loss.len().max(training.right_loss.len());
    }
    0
}

fn stored_abs(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        0.0
    } else if value < 0.0 {
        -value
    } else {
        value
    }
}

fn public_label(label: &str) -> String {
    let normalized = label.replace('\\', "/");
    normalized
        .rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or("run")
        .to_string()
}

fn scrub(text: &str, labels: &[&str]) -> String {
    let mut text = text.to_string();
    for label in labels {
        if label.contains('/') || label.contains('\\') {
            text = text.replace(label, &public_label(label));
        }
    }
    text
}

fn normalize(path: &str) -> Result<String, String> {
    let mut normalized = path.replace('\\', "/");
    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }
    if normalized.split('/').any(|part| part == "..") || normalized.starts_with('/') {
        return Err(format!("A bundle path cannot leave the archive: {path}"));
    }
    Ok(normalized)
}

fn ordinal_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

fn store_bands(bands: &[Option<Band>]) -> Vec<Option<StoredBand>> {
    bands
        .iter()
        .map(|band| band.map(|point| StoredBand { low: point.low, high: point.high }))
        .collect()
}

fn store_pairs(points: &[(f64, f64)]) -> Vec<[f64; 2]> {
    points.iter().map(|(value, probability)| [*value, *probability]).collect()
}

fn bands(stored: &[Option<StoredBand>]) -> Vec<Option<Band>> {
    stored
        .iter()
        .map(|band| band.map(|point| Band { low: point.low, high: point.high }))
        .collect()
}

fn pairs(stored: &[[f64; 2]]) -> Vec<(f64, f64)> {
    stored.iter().map(|point| (point[0], point[1])).collect()
}

fn training_entry(run_id: &str, loss: &[f64]) -> crate::open::TrainingEntry {
    crate::open::TrainingEntry {
        run_id: run_id.to_string(),
        model_name: String::new(),
        loss: loss.to_vec(),
        final_loss: None,
        train_steps: None,
        held_out_loss: None,
        perplexity: None,
        eval_n: None,
        task_metrics: Vec::new(),
        metric_ci: Vec::new(),
        entry_count: 1,
        selection: String::new(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest<'a> {
    bundle_id: &'a str,
    bundle_version: &'a str,
    profile: &'a str,
    created_at: &'a str,
    app_version: &'a str,
    delta_spec_version: &'a str,
    privacy: Privacy,
    contents: Contents,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Privacy {
    includes_raw_data: bool,
    includes_file_paths: bool,
    includes_machine_name: bool,
    data_classification: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Contents {
    includes_findings: bool,
    includes_repro: bool,
    includes_environment: bool,
    includes_insights: bool,
    includes_assets: bool,
    includes_audit: bool,
    includes_evidence: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeltaRow<'a> {
    id: &'a str,
    name: &'a str,
    explanation: &'a str,
    summary_sentence: &'a str,
    delta_type: &'a str,
    status: &'a str,
    left_value: f64,
    right_value: f64,
    delta: f64,
    magnitude: f64,
    units: &'a str,
    is_meaningful: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WhyRow<'a> {
    delta_id: &'a str,
    delta_name: &'a str,
    explanation: &'a str,
    summary_sentence: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Repro<'a> {
    input_fingerprint: &'a str,
    delta_hash: &'a str,
    determinism_enabled: bool,
    delta_spec_version: &'a str,
    reproducibility_badge: &'a str,
    alignment_mode: &'a str,
    timestep_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Environment<'a> {
    app_version: &'a str,
    platform: &'a str,
    dot_net_version: &'a str,
    is64_bit_process: bool,
    processor_count: i32,
}
