//! The ScalarScope review. Inference traces and backpropagate training
//! histories are different instruments, and this crate keeps them apart.

pub mod bundle;
pub mod geometry;
pub mod geometry_deltas;
pub mod history;
pub mod knobs;
pub mod milestones;
pub mod open;
pub mod prefs;
pub mod readings;
pub mod review;
pub mod runtrace;
pub mod shape;
pub mod stats;
pub mod svg;
pub mod trends;
pub mod ui;
pub mod views;
pub mod workbench;

pub use open::{open_path, open_text, Loaded, Side};
pub use review::{pair, Pair};
