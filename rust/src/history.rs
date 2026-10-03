//! Local history of finished reviews.
//!
//! The file is `comparison-log.json`. A packaged build with this Store
//! identity reads and writes the package LocalState folder, which is the
//! same folder the .NET app uses. An unpackaged run does not invent that
//! folder. The file stays on the machine.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::bundle::{sha256_hex, utc_now};

pub const FILE_NAME: &str = "comparison-log.json";
pub const MAX_ENTRIES: usize = 40;
pub const HOME_COUNT: usize = 12;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub finished_at: String,
    #[serde(default)]
    pub left_name: String,
    #[serde(default)]
    pub right_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_hash: Option<String>,
    #[serde(default)]
    pub alignment: String,
    #[serde(default)]
    pub deltas_fired: Vec<String>,
    #[serde(default = "compare_kind")]
    pub kind: String,
}

fn compare_kind() -> String {
    "compare".to_string()
}

impl LogEntry {
    pub fn title(&self) -> String {
        if self.kind == "bundle" || self.right_name.trim().is_empty() {
            self.left_name.clone()
        } else {
            format!("{} vs {}", self.left_name, self.right_name)
        }
    }

    pub fn subtitle(&self) -> String {
        let deltas = if self.deltas_fired.is_empty() {
            "No deltas fired".to_string()
        } else {
            self.deltas_fired.join(" · ")
        };
        match self.bundle_hash.as_deref().filter(|hash| hash.len() >= 8) {
            Some(hash) => format!("{deltas} · {}", &hash[..8]),
            None => deltas,
        }
    }
}

pub fn file_path(directory: &Path) -> PathBuf {
    directory.join(FILE_NAME)
}

pub fn read(directory: &Path) -> Vec<LogEntry> {
    let path = file_path(directory);
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn record(incoming: LogEntry, directory: &Path) -> Result<LogEntry, String> {
    if directory.as_os_str().is_empty() {
        return Err("The history folder is missing.".to_string());
    }
    let mut entries = read(directory);
    let match_index = entries.iter().position(|existing| same_sitting(existing, &incoming));
    let mut saved = incoming;
    saved.id = match match_index.and_then(|index| entries.get(index)) {
        Some(existing) if !existing.id.trim().is_empty() => existing.id.clone(),
        _ => new_id(&saved),
    };
    saved.finished_at = utc_now();
    if saved.kind.trim().is_empty() {
        saved.kind = compare_kind();
    }
    if let Some(index) = match_index {
        let previous = &entries[index];
        if saved.bundle_path.as_deref().unwrap_or("").trim().is_empty() {
            saved.bundle_path = previous.bundle_path.clone();
            saved.bundle_hash = previous.bundle_hash.clone();
        }
        entries.remove(index);
    }
    entries.insert(0, saved.clone());
    if entries.len() > MAX_ENTRIES {
        entries.truncate(MAX_ENTRIES);
    }
    write(directory, &entries)?;
    Ok(saved)
}

pub fn attach_bundle(bundle_path: &str, bundle_hash: Option<&str>, directory: &Path) -> Result<LogEntry, String> {
    if bundle_path.trim().is_empty() {
        return Err("The bundle path is missing.".to_string());
    }
    let mut entries = read(directory);
    if let Some(top) = entries.first_mut() {
        if top.kind != "bundle" {
            top.bundle_path = Some(bundle_path.to_string());
            top.bundle_hash = bundle_hash.map(str::to_string);
            top.finished_at = utc_now();
            let saved = top.clone();
            write(directory, &entries)?;
            return Ok(saved);
        }
        if top.kind == "bundle" && paths_equal(top.bundle_path.as_deref().unwrap_or(""), bundle_path) {
            top.bundle_hash = bundle_hash.map(str::to_string);
            top.finished_at = utc_now();
            let saved = top.clone();
            write(directory, &entries)?;
            return Ok(saved);
        }
    }
    let name = Path::new(bundle_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("bundle")
        .to_string();
    record(
        LogEntry {
            id: String::new(),
            finished_at: String::new(),
            left_name: name,
            right_name: String::new(),
            left_path: None,
            right_path: None,
            left_run_id: None,
            right_run_id: None,
            bundle_path: Some(bundle_path.to_string()),
            bundle_hash: bundle_hash.map(str::to_string),
            alignment: String::new(),
            deltas_fired: Vec::new(),
            kind: "bundle".to_string(),
        },
        directory,
    )
}

pub fn clear(directory: &Path) -> Result<(), String> {
    let path = file_path(directory);
    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("Could not clear the history. {error}"))?;
    }
    Ok(())
}

/// The package LocalState folder when this process is the Store package.
/// An unpackaged process gets nothing here.
pub fn package_local_state() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        windows_local_state()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn same_sitting(existing: &LogEntry, incoming: &LogEntry) -> bool {
    if incoming.kind == "bundle" {
        return existing.kind == "bundle"
            && incoming.bundle_path.as_deref().is_some_and(|path| !path.trim().is_empty())
            && paths_equal(
                existing.bundle_path.as_deref().unwrap_or(""),
                incoming.bundle_path.as_deref().unwrap_or(""),
            );
    }
    if incoming.left_path.as_deref().is_some_and(|path| !path.trim().is_empty())
        && incoming.right_path.as_deref().is_some_and(|path| !path.trim().is_empty())
    {
        return paths_equal(existing.left_path.as_deref().unwrap_or(""), incoming.left_path.as_deref().unwrap_or(""))
            && paths_equal(
                existing.right_path.as_deref().unwrap_or(""),
                incoming.right_path.as_deref().unwrap_or(""),
            )
            && existing.kind != "bundle";
    }
    existing.kind == incoming.kind
        && existing.left_name == incoming.left_name
        && existing.right_name == incoming.right_name
        && existing.left_path.as_deref().unwrap_or("").trim().is_empty()
        && incoming.left_path.as_deref().unwrap_or("").trim().is_empty()
}

fn paths_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn write(directory: &Path, entries: &[LogEntry]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| format!("Could not create the history folder. {error}"))?;
    let mut bytes = serde_json::to_vec_pretty(entries).map_err(|error| format!("Could not write the history. {error}"))?;
    bytes.push(b'\n');
    fs::write(file_path(directory), bytes).map_err(|error| format!("Could not write the history. {error}"))
}

fn new_id(entry: &LogEntry) -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
    let basis = format!(
        "{}\n{}\n{}\n{}\n{nanos}",
        entry.kind,
        entry.left_path.as_deref().unwrap_or(&entry.left_name),
        entry.right_path.as_deref().unwrap_or(&entry.right_name),
        entry.bundle_path.as_deref().unwrap_or("")
    );
    sha256_hex(basis.as_bytes())[..32].to_string()
}

#[cfg(windows)]
fn windows_local_state() -> Option<PathBuf> {
    let family = package_family_name()?;
    if family.is_empty() || family.contains('\\') || family.contains('/') || family.contains("..") {
        return None;
    }
    let local = local_app_data()?;
    Some(local.join("Packages").join(family).join("LocalState"))
}

#[cfg(windows)]
fn package_family_name() -> Option<String> {
    use std::ptr;
    const ERROR_INSUFFICIENT_BUFFER: i32 = 122;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentPackageFamilyName(length: *mut u32, name: *mut u16) -> i32;
    }
    unsafe {
        let mut length = 0u32;
        let first = GetCurrentPackageFamilyName(&mut length, ptr::null_mut());
        if first != ERROR_INSUFFICIENT_BUFFER || length == 0 {
            return None;
        }
        let mut buffer = vec![0u16; length as usize];
        let second = GetCurrentPackageFamilyName(&mut length, buffer.as_mut_ptr());
        if second != 0 {
            return None;
        }
        let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
        String::from_utf16(&buffer[..end]).ok()
    }
}

#[cfg(windows)]
fn local_app_data() -> Option<PathBuf> {
    use std::ffi::c_void;
    use std::ptr;
    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }
    const LOCAL_APP_DATA: Guid = Guid {
        data1: 0xF1B3_2785,
        data2: 0x6FBA,
        data3: 0x4FCF,
        data4: [0x9D, 0x55, 0x7B, 0x8E, 0x7F, 0x15, 0x70, 0x91],
    };
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(id: *const Guid, flags: u32, token: *mut c_void, path: *mut *mut u16) -> i32;
    }
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(pointer: *mut c_void);
    }
    unsafe {
        let mut raw = ptr::null_mut();
        let result = SHGetKnownFolderPath(&LOCAL_APP_DATA, 0, ptr::null_mut(), &mut raw);
        if result != 0 || raw.is_null() {
            return None;
        }
        let mut length = 0usize;
        while *raw.add(length) != 0 {
            length += 1;
        }
        let text = String::from_utf16(std::slice::from_raw_parts(raw, length)).ok();
        CoTaskMemFree(raw.cast());
        text.map(PathBuf::from)
    }
}
