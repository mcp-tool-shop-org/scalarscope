use std::collections::BTreeMap;

use scalarscope::bundle::{
    bundle_hash, document_from_pair, open_bytes, pack, seal, sha256_hex, unpack, utc_stamp, verify, CONTENT_CHECK,
};
use scalarscope::open::{open_text, InferenceRun, Side};
use scalarscope::review::{pair, Pair};

const STAMP: &str = "2026-10-03T12:00:00Z";
const ALPHA: &str = "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8";
const BETA: &str = "f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753";
const BUNDLE: &str = "5d0591b8a141f38746974cf2deb9304874bb86e2041b2bf4a5c7e7232ea2c4aa";

#[test]
fn phase72_preimage_matches_the_dotnet_sha256() {
    assert_eq!(sha256_hex(b"alpha"), ALPHA);
    assert_eq!(sha256_hex(b"beta"), BETA);
    let mut files = BTreeMap::new();
    files.insert("a.txt".to_string(), ALPHA.to_string());
    files.insert("b.txt".to_string(), BETA.to_string());
    assert_eq!(bundle_hash(&files), BUNDLE);
    let reversed = format!("b.txt:{BETA}\na.txt:{ALPHA}\n");
    assert_ne!(sha256_hex(reversed.as_bytes()), BUNDLE);
}

#[test]
fn utc_stamp_matches_the_known_instant() {
    assert_eq!(utc_stamp(0), "1970-01-01T00:00:00Z");
    assert_eq!(utc_stamp(1_791_028_800), STAMP);
}

#[test]
fn sealed_bundle_reopens_the_stored_review() {
    let sealed = seal(&document_from_pair(&spike()), STAMP).unwrap();
    let again = seal(&document_from_pair(&spike()), STAMP).unwrap();
    assert_eq!(sealed.bytes, again.bytes);

    let entries = unpack(&sealed.bytes).unwrap();
    let manifest = entry(&entries, "manifest.json");
    assert_eq!(manifest[0], b'{');
    assert!(!manifest.starts_with(&[0xef, 0xbb, 0xbf]));
    let manifest_text = String::from_utf8(manifest).unwrap();
    assert!(manifest_text.contains("\"includesFilePaths\": false"));
    assert!(!manifest_text.contains("machineName"));

    let integrity = String::from_utf8(entry(&entries, "integrity.json")).unwrap();
    assert!(!integrity.contains("\"integrity.json\""));
    assert!(integrity.contains(&sealed.bundle_hash));

    let deltas = String::from_utf8(entry(&entries, "findings/deltas.json")).unwrap();
    assert!(deltas.contains("FailurePresence"));
    assert!(deltas.contains("Introduced 8 new runtime anomalies"));
    assert!(!deltas.contains("confidence"));
    assert!(!deltas.contains("StructuralEmergence"));

    let summary = String::from_utf8(entry(&entries, "findings/summary.md")).unwrap();
    assert!(summary.contains(CONTENT_CHECK));
    assert!(summary.contains("does not report a confidence number"));
    assert!(summary.contains("not a confidence interval"));

    let verification = verify(&sealed.bytes).unwrap();
    assert!(verification.valid, "{:?}", verification.codes);
    assert_eq!(verification.actual_bundle_hash, sealed.bundle_hash);

    let opened = open_bytes(&sealed.bytes).unwrap();
    let Pair::Inference(live) = spike() else { panic!("inference") };
    assert_eq!(opened.review.verdict, live.verdict);
    // Spikes are ΔF; the spread of the typical samples (ΔO) did not change.
    assert_eq!(opened.review.fired, vec!["ΔF".to_string()]);
    assert_eq!(opened.review.inference.unwrap().left, live.left);
}

#[test]
fn a_planted_verdict_is_what_reopens() {
    let mut document = document_from_pair(&spike());
    document.review.verdict = "PLANTED review sentence".to_string();
    document.review.inference.as_mut().unwrap().left = vec![Some(-0.0)];
    let sealed = seal(&document, STAMP).unwrap();
    let opened = open_bytes(&sealed.bytes).unwrap();
    assert_eq!(opened.review.verdict, "PLANTED review sentence");
    let Pair::Inference(live) = spike() else { panic!("inference") };
    assert_ne!(opened.review.verdict, live.verdict);
    let stored = opened.review.inference.unwrap();
    assert_eq!(stored.left, vec![Some(0.0)]);
    assert!(stored.left[0].unwrap().is_sign_positive());
    let review = String::from_utf8(entry(&unpack(&sealed.bytes).unwrap(), "review/review.json")).unwrap();
    assert!(!review.contains("-0.0"));
}

#[test]
fn a_changed_byte_or_an_undeclared_file_fails() {
    let sealed = seal(&document_from_pair(&spike()), STAMP).unwrap();
    let mut changed = unpack(&sealed.bytes).unwrap();
    let deltas = changed.iter_mut().find(|(path, _)| path == "findings/deltas.json").unwrap();
    deltas.1.push(b' ');
    let changed = pack(&changed).unwrap();
    let verification = verify(&changed).unwrap();
    assert!(!verification.valid);
    assert!(verification.codes.iter().any(|code| code == "ModifiedFiles"));
    assert!(verification.codes.iter().any(|code| code == "BundleHashMismatch"));
    assert!(open_bytes(&changed).is_err());

    let mut extra = unpack(&sealed.bytes).unwrap();
    extra.push(("notes.txt".to_string(), b"extra".to_vec()));
    let extra = pack(&extra).unwrap();
    let verification = verify(&extra).unwrap();
    assert!(!verification.valid);
    assert!(verification.codes.iter().any(|code| code == "UndeclaredFile"));
    assert!(open_bytes(&extra).is_err());

    let mut missing = unpack(&sealed.bytes).unwrap();
    missing.retain(|(path, _)| path != "findings/deltas.json");
    let missing = pack(&missing).unwrap();
    let verification = verify(&missing).unwrap();
    assert!(verification.codes.iter().any(|code| code == "MissingFiles"));
}

#[test]
fn rewriting_the_bytes_and_the_hash_still_verifies() {
    let sealed = seal(&document_from_pair(&spike()), STAMP).unwrap();
    let mut entries = unpack(&sealed.bytes).unwrap();
    let readme = entries.iter_mut().find(|(path, _)| path == "README.md").unwrap();
    readme.1.extend_from_slice(b"\nEdited.\n");
    let mut hashes = BTreeMap::new();
    for (path, data) in &entries {
        if path == "integrity.json" {
            continue;
        }
        hashes.insert(path.clone(), sha256_hex(data));
    }
    let rewritten = bundle_hash(&hashes);
    assert_ne!(rewritten, sealed.bundle_hash);
    let integrity = integrity_bytes(&hashes, &rewritten);
    let slot = entries.iter_mut().find(|(path, _)| path == "integrity.json").unwrap();
    slot.1 = integrity;
    let bytes = pack(&entries).unwrap();
    let verification = verify(&bytes).unwrap();
    assert!(verification.valid, "{:?}", verification.codes);
    assert_eq!(verification.actual_bundle_hash, rewritten);
}

#[test]
fn the_seal_is_outside_the_preimage() {
    let sealed = seal(&document_from_pair(&spike()), STAMP).unwrap();
    let mut entries = unpack(&sealed.bytes).unwrap();
    let slot = entries.iter_mut().find(|(path, _)| path == "integrity.json").unwrap();
    let text = String::from_utf8(slot.1.clone()).unwrap().replace(STAMP, "2026-10-03T13:00:00Z");
    slot.1 = text.into_bytes();
    let bytes = pack(&entries).unwrap();
    let verification = verify(&bytes).unwrap();
    assert!(verification.valid, "{:?}", verification.codes);
    assert_eq!(verification.actual_bundle_hash, sealed.bundle_hash);
}

#[test]
fn a_training_bundle_does_not_fire_an_inference_delta() {
    let left = r#"[{"run_id":"left-run","status":"completed","loss_history":[1.0,0.5],"final_loss":0.4}]"#;
    let right = r#"[{"run_id":"right-run","status":"completed","loss_history":[0.9,0.2],"final_loss":0.1}]"#;
    let built = pair(&open_text(left, "left").unwrap(), &open_text(right, "right").unwrap()).unwrap();
    let sealed = seal(&document_from_pair(&built), STAMP).unwrap();
    let opened = open_bytes(&sealed.bytes).unwrap();
    assert_eq!(opened.review.kind, "training");
    assert!(opened.review.fired.is_empty());
    assert!(opened.review.findings.is_empty());
    assert!(opened.review.caption.contains("not computed"));
    let training = opened.review.training.unwrap();
    assert_eq!(training.left_loss, vec![1.0, 0.5]);
    assert_eq!(training.right_loss, vec![0.9, 0.2]);
    let deltas = String::from_utf8(entry(&unpack(&sealed.bytes).unwrap(), "findings/deltas.json")).unwrap();
    assert!(!deltas.contains("FailurePresence"));
}

#[test]
fn a_directory_in_the_label_stays_out_of_the_archive() {
    let left = marked("desk/baseline", vec![10.0; 8], None);
    let right = marked("other/candidate", vec![10.0; 8], Some(2));
    let sealed = seal(&document_from_pair(&pair(&left, &right).unwrap()), STAMP).unwrap();
    let text = String::from_utf8(sealed.bytes).unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned());
    assert!(!text.contains("desk/"));
    assert!(!text.contains("other/"));
    assert!(text.contains("baseline"));
    assert!(text.contains("candidate"));
}

#[test]
fn dotnet_zip_reader_recomputes_the_bundle_hash() {
    let sealed = seal(&document_from_pair(&spike()), STAMP).unwrap();
    let bundle_path = std::env::temp_dir().join(format!("scalarscope-phase72-{}.scbundle", std::process::id()));
    let script_path = std::env::temp_dir().join(format!("scalarscope-phase72-{}.ps1", std::process::id()));
    std::fs::write(&bundle_path, &sealed.bytes).unwrap();
    std::fs::write(&script_path, DOTNET_HASH).unwrap();
    let output = std::process::Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-File")
        .arg(&script_path)
        .arg(&bundle_path)
        .output()
        .expect("pwsh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&bundle_path);
    let _ = std::fs::remove_file(&script_path);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout.trim(), sealed.bundle_hash);
}

#[test]
fn findings_without_a_series_stay_text() {
    let mut hashes = BTreeMap::new();
    let mut entries = Vec::new();
    let deltas = b"[{\"id\":\"FailurePresence\",\"status\":\"Present\",\"summarySentence\":\"Introduced 1 new runtime anomalies\"}]\n";
    entries.push(("findings/deltas.json".to_string(), deltas.to_vec()));
    hashes.insert("findings/deltas.json".to_string(), sha256_hex(deltas));
    let bundle = bundle_hash(&hashes);
    entries.push(("integrity.json".to_string(), integrity_bytes(&hashes, &bundle)));
    let bytes = pack(&entries).unwrap();
    let opened = open_bytes(&bytes).unwrap();
    assert_eq!(opened.review.kind, "findings");
    assert!(opened.review.inference.is_none());
    assert!(opened.review.verdict.contains("Introduced 1 new runtime anomalies"));
    assert!(opened.review.verdict.contains("ΔF"));
}

fn integrity_bytes(hashes: &BTreeMap<String, String>, bundle_hash: &str) -> Vec<u8> {
    let mut text = String::from("{\"fileHashes\":{");
    let mut first = true;
    for (path, hash) in hashes {
        if !first {
            text.push(',');
        }
        first = false;
        text.push_str(&format!("\"{path}\":\"{hash}\""));
    }
    text.push('}');
    text.push_str(&format!(",\"bundleHash\":\"{bundle_hash}\",\"computedAt\":\"{STAMP}\"}}"));
    text.into_bytes()
}

fn entry(entries: &[(String, Vec<u8>)], path: &str) -> Vec<u8> {
    entries.iter().find(|(name, _)| name == path).unwrap().1.clone()
}

/// Eight spikes in 200 samples against none: an excess ΔF calls beyond chance (p = 0.5^8).
/// A single spike is not enough for that, so it no longer fires.
fn spike() -> Pair {
    let calm = vec![10.0; 200];
    let mut spiked = vec![10.0; 200];
    for index in [20, 45, 70, 95, 120, 145, 170, 195] {
        spiked[index] = 100.0;
    }
    pair(&marked("a", calm, None), &marked("b", spiked, None)).unwrap()
}

fn marked(label: &str, latency: Vec<f64>, steady: Option<i64>) -> Side {
    let steps = (0..latency.len() as i64).collect();
    Side::Inference(InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput: Vec::new(),
        warmup_end: None,
        steady_step: steady,
        memory_mb: Vec::new(),
        elapsed_s: Vec::new(),
        cpu_percent: Vec::new(),
        gpu_percent: Vec::new(),
        trace: None,
        replicates: Vec::new(),
    })
}

const DOTNET_HASH: &str = r#"
$ErrorActionPreference = 'Stop'
$Path = $args[0]
Add-Type -AssemblyName System.IO.Compression.FileSystem
function Get-Sha([byte[]]$data) {
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    $hash = $sha.ComputeHash($data)
    return ([System.BitConverter]::ToString($hash)).Replace('-', '').ToLower()
  } finally {
    $sha.Dispose()
  }
}
$zip = [System.IO.Compression.ZipFile]::OpenRead($Path)
$map = New-Object 'System.Collections.Generic.Dictionary[string,string]'
$stated = $null
try {
  foreach ($entry in $zip.Entries) {
    $name = $entry.FullName.Replace('\', '/')
    while ($name.StartsWith('./')) { $name = $name.Substring(2) }
    if ($name.Length -eq 0 -or $name.EndsWith('/')) { continue }
    $stream = $entry.Open()
    try {
      $memory = New-Object System.IO.MemoryStream
      $stream.CopyTo($memory)
      $payload = $memory.ToArray()
      $memory.Dispose()
    } finally {
      $stream.Dispose()
    }
    if ($name -eq 'integrity.json') {
      $stated = [System.Text.Encoding]::UTF8.GetString($payload)
      continue
    }
    $map[$name] = Get-Sha $payload
  }
} finally {
  $zip.Dispose()
}
$keys = New-Object 'System.Collections.Generic.List[string]'
foreach ($key in $map.Keys) { $keys.Add($key) }
$keys.Sort([System.StringComparer]::Ordinal)
$builder = New-Object System.Text.StringBuilder
foreach ($key in $keys) {
  [void]$builder.Append($key)
  [void]$builder.Append(':')
  [void]$builder.Append($map[$key])
  [void]$builder.Append("`n")
}
$computed = Get-Sha ([System.Text.Encoding]::UTF8.GetBytes($builder.ToString()))
$info = $stated | ConvertFrom-Json
if ($info.bundleHash -ne $computed) {
  Write-Error "stated $($info.bundleHash) recomputed $computed"
  exit 1
}
Write-Output $computed
"#;
