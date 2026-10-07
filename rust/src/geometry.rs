//! An ASPIRE training-dynamics export: the geometry run 2.0 read (`Models/GeometryRun.cs`).
//!
//! The fields and their JSON names are 2.0's. One difference: 2.0 read exactly five evaluator
//! dimensions (correctness, coherence, calibration, tradeoffs, clarity) by name and dropped any
//! other; here every dimension named in `scalars.dimensions` is kept, so an export from a teacher
//! with other dimensions is read in full. Validation follows `FileValidationService.cs`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 2.0's file size limit for a geometry export.
pub const MAX_BYTES: usize = 100 * 1024 * 1024;
/// The five dimensions 2.0 knew by name, in its order.
pub const DOTNET_DIMENSIONS: [&str; 5] = ["correctness", "coherence", "calibration", "tradeoffs", "clarity"];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GeometryRun {
    #[serde(default = "schema_one")]
    pub schema_version: String,
    #[serde(default, rename = "run_metadata")]
    pub metadata: RunMetadata,
    #[serde(default)]
    pub reduction: Reduction,
    #[serde(default)]
    pub trajectory: Trajectory,
    #[serde(default)]
    pub scalars: Scalars,
    #[serde(default)]
    pub geometry: GeometryMetrics,
    #[serde(default)]
    pub evaluators: Evaluators,
    #[serde(default)]
    pub failures: Vec<Failure>,
}

fn schema_one() -> String {
    "1.0".to_string()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunMetadata {
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub seed: i64,
    #[serde(default)]
    pub training_items: i64,
    #[serde(default)]
    pub cycles: i64,
    #[serde(default)]
    pub holdout_professor: Option<String>,
    #[serde(default = "unknown")]
    pub conscience_tier: String,
    /// `training_step` (time-ordered) or `checkpoint_by_item` (blocks per checkpoint over a fixed
    /// item order, not time within a block). New in schema 1.1; see the spec's export contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_axis: Option<String>,
    /// With `checkpoint_by_item`: how many checkpoint blocks the steps hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoints: Option<i64>,
    /// `live`, `replayed` (epochs after the first replay cached scores) or `fixed_per_item`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scalar_source: Option<String>,
}

/// Step axes the contract names.
pub const STEP_AXES: [&str; 2] = ["training_step", "checkpoint_by_item"];
/// Score sources the contract names.
pub const SCALAR_SOURCES: [&str; 3] = ["live", "replayed", "fixed_per_item"];

impl RunMetadata {
    /// False when the export says its steps are checkpoint blocks over items, not time. A file
    /// that says nothing (schema 1.0, or an unknown value) is read as time, as 2.0 read it.
    pub fn steps_are_time(&self) -> bool {
        self.step_axis.as_deref() != Some("checkpoint_by_item")
    }

    /// `Some(source)` when the export says its scores repeat (`replayed` or `fixed_per_item`).
    pub fn repeated_scores(&self) -> Option<&str> {
        self.scalar_source.as_deref().filter(|source| *source == "replayed" || *source == "fixed_per_item")
    }
}

fn unknown() -> String {
    "UNKNOWN".to_string()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Reduction {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub input_dim: i64,
    #[serde(default)]
    pub output_dim: i64,
    #[serde(default)]
    pub explained_variance: Vec<f64>,
    #[serde(default)]
    pub components: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Trajectory {
    #[serde(default)]
    pub timesteps: Vec<Timestep>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Timestep {
    #[serde(default)]
    pub t: f64,
    #[serde(default)]
    pub state_2d: Vec<f64>,
    #[serde(default)]
    pub velocity: Vec<f64>,
    #[serde(default)]
    pub curvature: f64,
    #[serde(default)]
    pub effective_dim: f64,
}

impl Timestep {
    /// 2.0's `VelocityMagnitude`: the length of the 2-D velocity, or 0 without one.
    pub fn speed(&self) -> f64 {
        if self.velocity.len() >= 2 {
            (self.velocity[0].powi(2) + self.velocity[1].powi(2)).sqrt()
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Scalars {
    #[serde(default)]
    pub dimensions: Vec<String>,
    #[serde(default)]
    pub values: Vec<ScalarStep>,
}

/// One step of evaluator scores: `t` and a score per dimension name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScalarStep {
    #[serde(default)]
    pub t: f64,
    #[serde(flatten)]
    pub scores: BTreeMap<String, f64>,
}

impl ScalarStep {
    /// A dimension's score; 0 when the step does not have it, as 2.0 defaulted.
    pub fn score(&self, dimension: &str) -> f64 {
        self.scores.get(dimension).copied().unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GeometryMetrics {
    #[serde(default)]
    pub eigenvalues: Vec<EigenStep>,
    #[serde(default)]
    pub anisotropy: Vec<AnisotropyStep>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EigenStep {
    #[serde(default)]
    pub t: f64,
    #[serde(default)]
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnisotropyStep {
    #[serde(default)]
    pub t: f64,
    #[serde(default)]
    pub ratio: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Evaluators {
    #[serde(default)]
    pub latent_dim: i64,
    #[serde(default)]
    pub professors: Vec<Professor>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Professor {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub vector: Vec<f64>,
    #[serde(default)]
    pub holdout: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Failure {
    #[serde(default)]
    pub t: f64,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub description: String,
}

impl GeometryRun {
    /// The dimensions in the order the file lists them, or the order the first step has them.
    pub fn dimension_names(&self) -> Vec<String> {
        if !self.scalars.dimensions.is_empty() {
            return self.scalars.dimensions.clone();
        }
        self.scalars.values.first().map(|step| step.scores.keys().cloned().collect()).unwrap_or_default()
    }

    /// A display name: the run id, else the condition.
    pub fn name(&self) -> String {
        if self.metadata.run_id.trim().is_empty() {
            self.metadata.condition.clone()
        } else {
            self.metadata.run_id.clone()
        }
    }
}

/// What reading a geometry export found: the run, and the warnings 2.0 gave for usable files.
#[derive(Clone, Debug, PartialEq)]
pub struct Opened {
    pub run: GeometryRun,
    pub warnings: Vec<String>,
}

/// A JSON object is a geometry export when it has a `trajectory`, as 2.0 decided.
pub fn looks_like_geometry(value: &Value) -> bool {
    value.get("trajectory").is_some()
}

/// Parse and validate a geometry export with 2.0's errors and warnings.
pub fn read(value: &Value) -> Result<Opened, String> {
    let run: GeometryRun = serde_json::from_value(value.clone()).map_err(|error| format!("This geometry export does not have the expected fields. {error}"))?;
    let mut errors = Vec::new();
    if run.trajectory.timesteps.is_empty() {
        errors.push("Missing trajectory data - no timesteps found".to_string());
    }
    if run.trajectory.timesteps.first().is_some_and(|step| step.state_2d.len() < 2) {
        errors.push("Trajectory needs at least 2D coordinates (State2D)".to_string());
    }
    if run.metadata.run_id.trim().is_empty() && run.metadata.condition.trim().is_empty() {
        errors.push("Missing run metadata (run_id or condition)".to_string());
    }
    if !run.schema_version.is_empty() && run.schema_version.split('.').next() != Some("1") {
        errors.push(format!("Schema version {} may not be fully compatible (expected 1.x)", run.schema_version));
    }
    if !errors.is_empty() {
        return Err(format!("This geometry export cannot be read: {}", errors.join("; ")));
    }
    let mut warnings = Vec::new();
    if run.trajectory.timesteps.iter().any(|step| step.state_2d.iter().any(|value| !value.is_finite())) {
        warnings.push("Some trajectory values are NaN or Infinity - visualization may be incomplete".to_string());
    }
    if run.geometry.eigenvalues.is_empty() {
        warnings.push("No eigenvalue data - Eigen-Spectrum view will be empty".to_string());
    }
    if run.scalars.values.is_empty() {
        warnings.push("No scalar metrics - some visualizations may be limited".to_string());
    }
    if run.evaluators.professors.is_empty() {
        warnings.push("No evaluator vectors - professor arrows won't be shown".to_string());
    }
    if let Some(axis) = run.metadata.step_axis.as_deref().filter(|axis| !STEP_AXES.contains(axis)) {
        warnings.push(format!("Unknown step_axis \"{axis}\" - steps are read as training time"));
    }
    if let Some(source) = run.metadata.scalar_source.as_deref().filter(|source| !SCALAR_SOURCES.contains(source)) {
        warnings.push(format!("Unknown scalar_source \"{source}\" - scores are read as live"));
    }
    Ok(Opened { run, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> Value {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/ScalarScope/Resources/Raw/Samples").join(name);
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn the_2_0_samples_read_in_full() {
        for name in ["orthogonal_professors.json", "correlated_professors.json"] {
            let value = sample(name);
            assert!(looks_like_geometry(&value));
            let opened = read(&value).unwrap();
            let run = &opened.run;
            assert_eq!(run.trajectory.timesteps.len(), 11, "{name}");
            assert_eq!(run.dimension_names(), DOTNET_DIMENSIONS.map(str::to_string).to_vec(), "{name}");
            assert_eq!(run.evaluators.professors.len(), 3);
            assert!(!run.failures.is_empty() || name.starts_with("correlated"));
            assert!(opened.warnings.is_empty(), "{name}: {:?}", opened.warnings);
            assert!(run.scalars.values[0].score("correctness") > 0.0);
        }
    }

    #[test]
    fn other_dimensions_are_kept_and_bad_files_say_why() {
        let value = serde_json::json!({
            "run_metadata": {"run_id": "r"},
            "trajectory": {"timesteps": [{"t": 0.0, "state_2d": [0.0, 1.0]}]},
            "scalars": {"dimensions": ["reasoning", "nuance"], "values": [{"t": 0.0, "reasoning": 7.5, "nuance": 6.0}]}
        });
        let opened = read(&value).unwrap();
        assert_eq!(opened.run.scalars.values[0].score("nuance"), 6.0);
        assert_eq!(opened.run.dimension_names(), vec!["reasoning", "nuance"]);
        assert!(opened.warnings.iter().any(|warning| warning.contains("eigenvalue")));
        let empty = serde_json::json!({"trajectory": {"timesteps": []}});
        let error = read(&empty).unwrap_err();
        assert!(error.contains("no timesteps") && error.contains("run_id or condition"), "{error}");
        let future = serde_json::json!({"schema_version": "2.0", "run_metadata": {"condition": "c"}, "trajectory": {"timesteps": [{"state_2d": [0.0, 0.0]}]}});
        assert!(read(&future).unwrap_err().contains("expected 1.x"));
    }
}
