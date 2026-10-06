//! The ScalarScope review. Inference traces and backpropagate training
//! histories are different instruments, and this crate keeps them apart.

pub mod bundle;
pub mod history;
pub mod milestones;
pub mod open;
pub mod prefs;
pub mod readings;
pub mod review;
pub mod runtrace;
pub mod shape;
pub mod stats;
pub mod ui;
pub mod views;

pub use open::{open_path, open_text, Loaded, Side};
pub use review::{pair, Pair};
