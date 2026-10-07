//! The settings a run was made with: batch size, precision, TensorRT, threads, CUDA graphs,
//! input shape. The workbench weighs a knob only on runs that record it.
//!
//! A knob is read from the first of three places that has it: a `knobs` object in the
//! RunTrace `metadata`, a `knobs.json` beside the run, then `key=value` pairs in the run's
//! folder name (`batch=8_precision=fp16`). Each knob keeps where it was read, and the pane
//! shows that.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

/// The file a run may keep beside it.
pub const KNOBS_FILE: &str = "knobs.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Trace,
    File,
    FolderName,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Trace => "RunTrace metadata",
            Source::File => KNOBS_FILE,
            Source::FolderName => "folder name",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Knobs {
    pub values: Map<String, Value>,
    pub sources: BTreeMap<String, Source>,
}

impl Knobs {
    /// The knobs a stored RunTrace names.
    pub fn from_trace(knobs: &Map<String, Value>) -> Knobs {
        let mut out = Knobs::default();
        out.fill(knobs, Source::Trace);
        out
    }

    /// Add the knobs not already set.
    pub fn fill(&mut self, knobs: &Map<String, Value>, source: Source) {
        for (key, value) in knobs {
            if usable(value) && !self.values.contains_key(key) {
                self.values.insert(key.clone(), value.clone());
                self.sources.insert(key.clone(), source);
            }
        }
    }

    /// Fill from `knobs.json` in `folder` and then from the folder's own name.
    pub fn fill_from_folder(&mut self, folder: &Path) {
        if let Some(knobs) = read_file(&folder.join(KNOBS_FILE)) {
            self.fill(&knobs, Source::File);
        }
        if let Some(name) = folder.file_name().and_then(|name| name.to_str()) {
            self.fill(&from_name(name), Source::FolderName);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// A knob is a number, a word or a flag. Lists and objects are left out.
fn usable(value: &Value) -> bool {
    matches!(value, Value::Number(_) | Value::String(_) | Value::Bool(_))
}

/// The object in a knobs file, or nothing when the file is missing or is not an object.
fn read_file(path: &Path) -> Option<Map<String, Value>> {
    let text = fs::read_to_string(path).ok()?;
    match serde_json::from_str::<Value>(&text).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

/// `key=value` pairs in a name, joined by `_` or `,`. A key may hold `_`; a value ends at the
/// next `_` or `,` unless it is the last. A name that does not read cleanly gives nothing.
pub fn from_name(name: &str) -> Map<String, Value> {
    let pieces: Vec<&str> = name.split('=').collect();
    let mut out = Map::new();
    if pieces.len() < 2 {
        return out;
    }
    let mut key = pieces[0].trim();
    for (index, piece) in pieces.iter().enumerate().skip(1) {
        let last = index == pieces.len() - 1;
        let (value, next) = if last {
            (*piece, "")
        } else {
            match piece.find(['_', ',']) {
                Some(at) => (&piece[..at], &piece[at + 1..]),
                None => return Map::new(),
            }
        };
        if !is_key(key) || value.is_empty() {
            return Map::new();
        }
        out.insert(key.to_string(), parse_value(value));
        key = next.trim_start_matches([',', '_']).trim();
    }
    out
}

fn is_key(key: &str) -> bool {
    key.starts_with(|ch: char| ch.is_ascii_alphabetic())
        && key.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn parse_value(text: &str) -> Value {
    if let Ok(number) = text.parse::<i64>() {
        return Value::from(number);
    }
    if let Ok(number) = text.parse::<f64>() {
        if number.is_finite() {
            return Value::from(number);
        }
    }
    match text.to_ascii_lowercase().as_str() {
        "true" | "on" => Value::Bool(true),
        "false" | "off" => Value::Bool(false),
        _ => Value::from(text),
    }
}
