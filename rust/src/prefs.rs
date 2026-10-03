//! Settings the .NET app already saved in `preferences.json`.
//!
//! A packaged run reads that file and writes it back without dropping
//! fields this review does not use. An unpackaged run does not call this.
//! Plugin DLLs in the same folder are not loaded.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::bundle::utc_now;

pub const FILE_NAME: &str = "preferences.json";

#[derive(Clone, Debug, PartialEq)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedView {
    pub title: String,
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewPrefs {
    pub color_vision: u8,
    pub high_contrast: bool,
    pub text_scale: f32,
    pub recent_limit: usize,
    pub recent: Vec<RecentFile>,
}

impl Default for ReviewPrefs {
    fn default() -> Self {
        Self {
            color_vision: 0,
            high_contrast: false,
            text_scale: 1.0,
            recent_limit: 10,
            recent: Vec::new(),
        }
    }
}

/// Colors for the two series and the anomaly marks. Hex digits, no leading `#`.
/// The default pair stays the review's own teal and red. A saved color-vision
/// mode replaces that pair. The high-contrast checkbox uses the high-contrast
/// palette when the dropdown is still Default. A named mode wins over the checkbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeriesPalette {
    pub left: &'static str,
    pub right: &'static str,
    pub mark: &'static str,
    pub note: &'static str,
    pub background: &'static str,
    pub label: Option<&'static str>,
}

pub fn series_palette(color_vision: u8, high_contrast: bool) -> SeriesPalette {
    match color_vision {
        1 => palette("0077bb", "ee7733", "33bbee", "12121f", "9aa0b4", Some("deuteranopia")),
        2 => palette("0077bb", "ddcc77", "33bbee", "12121f", "9aa0b4", Some("protanopia")),
        3 => palette("ee3377", "009988", "ee7733", "12121f", "9aa0b4", Some("tritanopia")),
        4 => high_contrast_palette(),
        5 => palette("ffffff", "cccccc", "888888", "000000", "9aa0b4", Some("monochrome")),
        0 if high_contrast => high_contrast_palette(),
        _ => palette("4ecdc4", "ff6b6b", "ffd93d", "12121f", "9aa0b4", None),
    }
}

pub fn read(directory: &Path) -> ReviewPrefs {
    let Ok(text) = fs::read_to_string(directory.join(FILE_NAME)) else {
        return ReviewPrefs::default();
    };
    let Ok(Value::Object(map)) = serde_json::from_str(&text) else {
        return ReviewPrefs::default();
    };
    prefs_from(&map)
}

/// Move a file to the front of `RecentFiles`. Every other key stays.
/// A file that is not a JSON object is left on disk.
pub fn remember_file(directory: &Path, path: &str, name: &str) -> Result<(), String> {
    if directory.as_os_str().is_empty() || path.trim().is_empty() {
        return Err("The preferences folder is missing.".to_string());
    }
    let file = directory.join(FILE_NAME);
    let mut root = if file.exists() {
        let text = fs::read_to_string(&file).map_err(|error| format!("Could not read preferences. {error}"))?;
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(map)) => map,
            _ => return Err("preferences.json is not an object. It was left unchanged.".to_string()),
        }
    } else {
        Map::new()
    };
    let limit = recent_limit(root.get("RecentFilesLimit"));
    let mut files = match root.get("RecentFiles") {
        Some(Value::Array(rows)) => rows.clone(),
        _ => Vec::new(),
    };
    files.retain(|entry| !paths_equal(&entry_str(entry, "Path").unwrap_or_default(), path));
    let display = if name.trim().is_empty() {
        Path::new(path).file_stem().and_then(|stem| stem.to_str()).unwrap_or("run").to_string()
    } else {
        name.to_string()
    };
    files.insert(
        0,
        serde_json::json!({
            "Path": path,
            "Name": display,
            "LastOpened": utc_now(),
        }),
    );
    files.truncate(limit);
    root.insert("RecentFiles".to_string(), Value::Array(files));
    write_object(directory, &root)
}

/// Saved views in `gallery/`. This review opens `dataSource` when that file
/// is a trace or a training history. It does not restore a geometry viewport.
pub fn saved_views(directory: &Path) -> Vec<SavedView> {
    let folder = directory.join("gallery");
    let Ok(entries) = fs::read_dir(&folder) else {
        return Vec::new();
    };
    let mut views = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let title = string_field(&map, "title").unwrap_or_else(|| "Saved view".to_string());
        let source = map
            .get("state")
            .and_then(Value::as_object)
            .and_then(|state| string_field(state, "dataSource"))
            .filter(|path| !path.trim().is_empty());
        let created = string_field(&map, "createdAt").unwrap_or_default();
        views.push((created, SavedView { title, source }));
    }
    views.sort_by(|left, right| right.0.cmp(&left.0));
    views.into_iter().map(|(_, view)| view).collect()
}

fn prefs_from(map: &Map<String, Value>) -> ReviewPrefs {
    let text_scale = map
        .get("TextScale")
        .and_then(Value::as_f64)
        .filter(|scale| scale.is_finite())
        .map(|scale| scale as f32)
        .unwrap_or(1.0);
    let recent = match map.get("RecentFiles") {
        Some(Value::Array(rows)) => rows
            .iter()
            .filter_map(|entry| {
                let path = entry_str(entry, "Path")?;
                if path.trim().is_empty() {
                    return None;
                }
                let name = entry_str(entry, "Name").filter(|name| !name.trim().is_empty()).unwrap_or_else(|| {
                    Path::new(&path).file_stem().and_then(|stem| stem.to_str()).unwrap_or("run").to_string()
                });
                Some(RecentFile {
                    path,
                    name,
                    last_opened: entry_str(entry, "LastOpened").unwrap_or_default(),
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    ReviewPrefs {
        color_vision: json_u8(map.get("ColorVisionMode")).unwrap_or(0),
        high_contrast: map.get("HighContrastMode").and_then(Value::as_bool).unwrap_or(false),
        text_scale: text_scale.clamp(0.75, 2.0),
        recent_limit: recent_limit(map.get("RecentFilesLimit")),
        recent,
    }
}

fn recent_limit(value: Option<&Value>) -> usize {
    match json_u8(value) {
        Some(limit) if (1..=40).contains(&limit) => limit as usize,
        _ => 10,
    }
}

fn write_object(directory: &Path, map: &Map<String, Value>) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| format!("Could not create the preferences folder. {error}"))?;
    let mut bytes = serde_json::to_vec_pretty(&Value::Object(map.clone())).map_err(|error| format!("Could not write preferences. {error}"))?;
    bytes.push(b'\n');
    fs::write(directory.join(FILE_NAME), bytes).map_err(|error| format!("Could not write preferences. {error}"))
}

fn entry_str(entry: &Value, key: &str) -> Option<String> {
    entry.as_object().and_then(|map| string_field(map, key))
}

fn string_field(map: &Map<String, Value>, key: &str) -> Option<String> {
    match map.get(key)? {
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn json_u8(value: Option<&Value>) -> Option<u8> {
    let value = value?;
    value
        .as_u64()
        .and_then(|number| u8::try_from(number).ok())
        .or_else(|| value.as_i64().and_then(|number| u8::try_from(number).ok()))
}

fn paths_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn palette(
    left: &'static str,
    right: &'static str,
    mark: &'static str,
    background: &'static str,
    note: &'static str,
    label: Option<&'static str>,
) -> SeriesPalette {
    SeriesPalette {
        left,
        right,
        mark,
        note,
        background,
        label,
    }
}

fn high_contrast_palette() -> SeriesPalette {
    palette("ffff00", "00ffff", "ff00ff", "000000", "ffff00", Some("high contrast"))
}
