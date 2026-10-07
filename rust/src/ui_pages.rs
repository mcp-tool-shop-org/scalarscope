//! The Welcome and Guide pages, and the keyboard shortcuts.

use eframe::egui::{self, RichText};

use super::{Page, ScalarScopeApp, View};
use crate::open::{open_text, Loaded};

/// The sample comparison: two synthetic latency runs of one model, before and after an
/// optimization. Generated for the app; no real run is in them.
pub const SAMPLE_BASELINE: &str = include_str!("../samples/baseline.csv");
pub const SAMPLE_OPTIMIZED: &str = include_str!("../samples/optimized.csv");

/// One Guide section: a title and its text.
pub struct Section {
    pub title: &'static str,
    pub body: &'static str,
}

pub const GUIDE: &[Section] = &[
    Section {
        title: "Reading a comparison",
        body: "Start with the headline: how B compares with A at p50, p90 and p99, each with a 95% interval. Then the tiles: which deltas fired, which stayed quiet, and which were withheld, each with its reason. Then the views, to see what the numbers say.",
    },
    Section {
        title: "The headline",
        body: "B/A at a percentile is B's latency there divided by A's: 0.65 means B is 35% faster. The interval is a 95% interval from 1000 moving-block bootstrap resamples, which keep neighbouring samples together because they are not independent. With one run per side it covers the variation within that run only, so the headline calls itself indicative: run-to-run variation is often larger. Open several runs per side to cover it. A percentile is shown only when there are enough samples to bound it; p99 needs 368.",
    },
    Section {
        title: "ΔF: new runtime anomalies",
        body: "An anomaly is a steady sample far from its run's median: by default more than 5 robust deviations (1.4826 × MAD), or 3 standard deviations under the 2.0 rule in Settings. Warmup samples are not counted. ΔF fires when B has more anomalies than A beyond chance, by a one-sided exact test at 0.05. One extra spike is not enough.",
    },
    Section {
        title: "ΔTc: stabilization time",
        body: "Each run gets a shape: flat, warmup, slowdown, no steady state, or too short to tell. Only a flat run or one that warms up has a stabilization time. Where it settles is a range: from where an 11-sample rolling median first reaches the steady level, to the latest PELT change point across three penalties. ΔTc fires only when the two ranges do not overlap. When a file states its steady-state step, that step is used as written. ΔTc counts steps; with elapsed time on both runs, the page also says how they compare in seconds when that differs.",
    },
    Section {
        title: "ΔO: runtime variability",
        body: "ΔO compares relative spread, (p90 − p10) / p50 of the steady samples, so a faster run with the same proportional jitter is not called steadier. It fires when the 95% interval on B's spread over A's excludes 1.",
    },
    Section {
        title: "ΔTd and ΔĀ",
        body: "Structural emergence (ΔTd) and spectrum concentration (ΔĀ) describe training dynamics: the geometry of a training trajectory, and how much of the evaluators' spread sits in its first direction (λ1/Σλ). 2.0 called ΔĀ evaluator agreement, but it measures concentration, not agreement. They do not apply to an inference run, so they stay off the inference page. They come with the geometry views.",
    },
    Section {
        title: "The views",
        body: "Series: latency by step from the steady state, with the p10–p90 band, anomaly marks and each run's levels. Warmup: each run from its first sample, with where it settles shaded. Distribution: the cumulative distribution with a threshold you click or drag to read P(latency > x), and 20 dots per run, each 5% of the samples. Difference: B − A by percentile with its interval and a zero line. Spectrum: latency by percentile on a log tail axis. Heat map: where samples fall, step by step, on a shared scale.",
    },
    Section {
        title: "Several runs per side",
        body: "Pick several files together, or open a folder whose subfolders are each a run. The headline and the difference then resample whole runs as well as samples, so their intervals include run-to-run variation. Three runs per side are needed before the headline stops calling itself indicative. The series view and the deltas use the first run.",
    },
    Section {
        title: "Bundles and the content check",
        body: "Save bundle writes the review as a .scbundle. Its SHA-256 checks that the bytes are intact. It is a content check, not a signature: it does not say who wrote the file. Opening a bundle shows the stored review exactly as saved, in review mode, with loading off until you close it.",
    },
    Section {
        title: "What you can open",
        body: "A latency CSV, a benchmark JSON, a Chrome or PyTorch profiler trace (.json or .json.gz), a runtime log, a ScalarScope RunTrace JSON, a run folder, or a backpropagate run_history.json. CSV columns for elapsed time (time_s), throughput, memory, CPU and GPU are read when present.",
    },
    Section {
        title: "The workbench",
        body: "The Workbench tab weighs knobs across many runs. Open runs there, or it uses the runs being compared. When you press Ask, a local model that can call tools (through Ollama on this computer; cloud models are refused) measures the runs with formulas, builds formula tools and proposes what a knob does. ScalarScope computes every number and sets every verdict. A hypothesis fixes its knob, formula and direction when it is proposed. On one set of runs it is not testable, confounded, inconclusive, supported or refuted by an exact rank test. Verdicts across sets of runs come only at checkpoints, one every five new sets, by e-BH at a 5% false discovery rate. Write your own call before you ask. The model's note sits below the verdicts, labelled as its words. Save session record keeps the model, its digest, every call and answer, and no paths.",
    },
    Section {
        title: "Knobs",
        body: "A knob is a setting a run was made with: batch size, precision, TensorRT, threads, CUDA graphs, input shape. It is read from the first place that has it: a knobs object in the RunTrace metadata, a knobs.json beside the run, then key=value pairs in the folder name, such as batch=8_precision=fp16. The page says where each came from. A run set counts as evidence about a knob only when that knob differs and the others match.",
    },
    Section {
        title: "Keyboard shortcuts",
        body: "F1: Guide. Ctrl+,: Settings. Ctrl+H: Welcome. On Compare, 1 to 6 choose Series, Warmup, Distribution, Difference, Spectrum and Heat map. Esc closes the Why panel.",
    },
];

/// The Guide sections whose title or text contains `query`, ignoring case.
pub fn search(query: &str) -> Vec<&'static Section> {
    let query = query.trim().to_lowercase();
    GUIDE
        .iter()
        .filter(|section| query.is_empty() || section.title.to_lowercase().contains(&query) || section.body.to_lowercase().contains(&query))
        .collect()
}

/// The view a number key chooses on Compare.
pub fn view_for_key(key: egui::Key) -> Option<View> {
    let index = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4, egui::Key::Num5, egui::Key::Num6]
        .iter()
        .position(|candidate| *candidate == key)?;
    let order = [View::Series, View::Warmup, View::Distribution, View::Difference, View::Spectrum, View::HeatMap];
    Some(order[index])
}

impl ScalarScopeApp {
    /// F1, Ctrl+,, Ctrl+H, 1–6 and Esc. Ignored while a text field has the keyboard.
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (help, settings, home, escape, number) = ctx.input(|input| {
            let number = input.events.iter().find_map(|event| match event {
                egui::Event::Key { key, pressed: true, modifiers, .. } if modifiers.is_none() => view_for_key(*key),
                _ => None,
            });
            (
                input.key_pressed(egui::Key::F1),
                input.modifiers.command && input.key_pressed(egui::Key::Comma),
                input.modifiers.command && input.key_pressed(egui::Key::H),
                input.key_pressed(egui::Key::Escape),
                number,
            )
        });
        if help {
            self.page = Page::Guide;
        }
        if settings {
            self.page = Page::Settings;
        }
        if home {
            self.page = Page::Welcome;
        }
        if escape {
            self.why = None;
            self.highlight = None;
        }
        if let (Some(view), Page::Compare) = (number, self.page) {
            self.view = view;
        }
    }

    /// Load the sample pair and go to Compare.
    pub(super) fn load_sample(&mut self) {
        let side = |text: &str, label: &str| open_text(text, label).map(|side| Loaded { path: format!("sample: {label}"), side });
        match (side(SAMPLE_BASELINE, "sample baseline"), side(SAMPLE_OPTIMIZED, "sample optimized")) {
            (Ok(left), Ok(right)) => {
                self.opened = None;
                self.built = None;
                self.left = Some(left);
                self.right = Some(right);
                self.note.clear();
                self.page = Page::Compare;
            }
            (Err(error), _) | (_, Err(error)) => self.note = error,
        }
    }

    pub(super) fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        ui.heading(RichText::new("Compare two inference runs, or two training runs.").color(paint.text));
        ui.label(
            RichText::new("ScalarScope reads two runs, says how B compares with A and how sure that is, and shows why. It works offline and sends nothing anywhere.")
                .color(paint.note),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Compare two runs").strong()).clicked() {
                self.page = Page::Compare;
            }
            if ui.button("Try the sample comparison").on_hover_text("Two synthetic runs of one model, before and after an optimization").clicked() {
                self.load_sample();
            }
            if ui.button("Open a review bundle").clicked() {
                self.page = Page::Compare;
                self.open_bundle();
            }
        });
        ui.add_space(10.0);
        let cards = [
            ("Ratios with intervals", "How much faster B is at p50, p90 and p99, with a 95% interval, and when there are too few samples to say."),
            ("Where runs settle", "Each run's shape, where its warmup ends as a range, and whether the two really differ."),
            ("Six views", "Series, warmup, distribution with a threshold, difference by percentile, tail spectrum, and heat map."),
            ("Reproducible bundles", "Save a review with a SHA-256 content check and reopen it exactly as saved."),
        ];
        ui.horizontal_wrapped(|ui| {
            for (title, text) in cards {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).color(paint.text).strong());
                        ui.add(egui::Label::new(RichText::new(text).color(paint.note)).wrap());
                    });
                });
            }
        });
        if !self.note.is_empty() {
            ui.label(RichText::new(&self.note).color(paint.mark));
        }
        self.draw_recent(ui);
        ui.add_space(8.0);
        ui.label(RichText::new("Local only. Nothing leaves your computer.").color(paint.note));
    }

    pub(super) fn draw_guide(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.add(egui::TextEdit::singleline(&mut self.guide_query).hint_text("ΔTc, bundle, warmup, p99…").desired_width(320.0));
        });
        let found = search(&self.guide_query);
        if found.is_empty() {
            ui.label(RichText::new("No section mentions that.").color(paint.note));
        }
        for section in found {
            ui.add_space(6.0);
            ui.label(RichText::new(section.title).color(paint.text).strong());
            ui.add(egui::Label::new(RichText::new(section.body).color(paint.note)).wrap());
        }
    }
}
