//! The Workbench page: a local model investigates the open inference runs by calling the shared
//! workbench's tools, and the program sets every number and verdict.
//!
//! Order on the page follows the evidence on reliance (spec M6): the runs and their knobs, then
//! the program's verdicts, then the person's own call, then the model's note under a label
//! that says it is the model's words.

use std::sync::mpsc::{Receiver, TryRecvError};

use eframe::egui::{self, RichText};
use serde_json::Value;
use workbench::{BenchReply, Board, Book, Hypothesis, LearnedTool, Step, Verdict, Workbench};

use crate::open::{open_path, InferenceRun, Side};
use crate::workbench as host;

use super::{pick_runs, take_save, ScalarScopeApp};

/// What the Workbench page holds between frames.
#[derive(Default)]
pub struct BenchState {
    /// Runs opened on this page. When empty, the runs being compared are used.
    pub runs: Vec<InferenceRun>,
    pub tools: Vec<LearnedTool>,
    pub hypotheses: Vec<Hypothesis>,
    pub book: Book,
    /// The memory was read for this sitting.
    pub loaded: bool,
    /// The board key the stored hypotheses were last retested on.
    pub retested: String,
    pub your_call: String,
    pub formula: String,
    pub formula_lines: Vec<String>,
    pub ask: Option<Receiver<BenchReply>>,
    pub status: String,
    pub last: Option<LastSession>,
}

/// The session that finished last, as the page shows it.
pub struct LastSession {
    pub model: String,
    pub digest: Option<String>,
    pub stopped: String,
    pub steps: Vec<Step>,
    pub note: Option<String>,
    pub note_dropped: bool,
    pub proposed: Vec<(String, String)>,
    pub learned: Vec<(String, String)>,
    pub not_used: Vec<&'static str>,
    /// The record a person can save as the session's receipt. It holds no paths.
    pub record: Value,
}

/// The session loop against the local Ollama. A test build points it at a closed port, so no test
/// ever reaches a real model or loads one on the GPU.
#[cfg(not(test))]
fn start(bench: Workbench) -> Receiver<BenchReply> {
    workbench::start_bench(bench)
}

#[cfg(test)]
fn start(bench: Workbench) -> Receiver<BenchReply> {
    workbench::start_bench_on(9, bench)
}

fn today() -> String {
    crate::bundle::utc_now().chars().take(10).collect()
}

/// Open each picked file, or each run in a picked folder, as one run of the workbench.
pub fn open_runs(paths: &[std::path::PathBuf]) -> Result<Vec<InferenceRun>, String> {
    let mut runs = Vec::new();
    for path in paths {
        let entries: Vec<std::path::PathBuf> = if path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(path)
                .map_err(|error| format!("Could not read the folder. {error}"))?
                .flatten()
                .map(|entry| entry.path())
                .filter(|entry| {
                    entry.is_dir()
                        || entry.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ["json", "csv", "log", "gz"].contains(&ext))
                })
                .collect();
            entries.sort();
            entries
        } else {
            vec![path.clone()]
        };
        for entry in entries {
            match open_path(&entry) {
                Ok(loaded) => {
                    if let Side::Inference(run) = loaded.side {
                        runs.extend(host::runs_of(&[&run]));
                    }
                }
                Err(error) if !path.is_dir() => return Err(error),
                Err(_) => {}
            }
        }
    }
    if runs.is_empty() {
        return Err("No inference run was found there.".to_string());
    }
    Ok(runs)
}

impl ScalarScopeApp {
    fn memory(&self) -> Option<std::path::PathBuf> {
        self.history_dir.as_deref().map(host::memory_file)
    }

    /// The runs the workbench works on: those opened on this page, else the compared runs.
    pub(super) fn bench_board(&self) -> Option<Board> {
        let runs = if self.bench.runs.is_empty() {
            let sides: Vec<&InferenceRun> = [&self.left, &self.right]
                .into_iter()
                .flatten()
                .filter_map(|loaded| match &loaded.side {
                    Side::Inference(run) => Some(run),
                    _ => None,
                })
                .collect();
            host::runs_of(&sides)
        } else {
            self.bench.runs.clone()
        };
        (!runs.is_empty()).then(|| host::board(runs, crate::stats::AnomalyRule::from_code(self.settings.anomaly_rule)))
    }

    fn load_bench_memory(&mut self) {
        if self.bench.loaded {
            return;
        }
        self.bench.loaded = true;
        if let Some(file) = self.memory() {
            self.bench.tools = workbench::read_tools(&file);
            self.bench.hypotheses = workbench::read_hypotheses(&file);
            self.bench.book = workbench::read_book(&file);
        }
    }

    /// On a board not seen in this sitting: retest the stored hypotheses for its method, and
    /// count it toward the next checkpoint when the workbench has never seen it.
    fn retest(&mut self, board: &Board) {
        if self.bench.retested == board.key {
            return;
        }
        self.bench.retested = board.key.clone();
        let date = today();
        let indexes: Vec<usize> =
            (0..self.bench.hypotheses.len()).filter(|index| self.bench.hypotheses[*index].method == board.method).collect();
        if !indexes.is_empty() {
            let batch: Vec<Hypothesis> = indexes.iter().map(|index| self.bench.hypotheses[*index].clone()).collect();
            let results = workbench::test_all(board, &batch, &self.bench.tools, &date);
            for (index, evaluation) in indexes.into_iter().zip(results) {
                workbench::record(&mut self.bench.hypotheses[index], evaluation);
            }
        }
        let Some(file) = self.memory() else {
            return;
        };
        let kept = workbench::write_hypotheses(&file, &self.bench.hypotheses).and_then(|()| host::note_board(&file, board));
        match kept {
            Ok(true) => {
                if let Some(checkpoint) = workbench::note_new_folder(&mut self.bench.book, &self.bench.hypotheses, &date) {
                    self.bench.status = format!("Checkpoint {} judged the bench.", checkpoint.number);
                }
                if let Err(error) = workbench::write_book(&file, &self.bench.book) {
                    self.bench.status = format!("Could not keep the checkpoints. {error}");
                }
            }
            Ok(false) => {}
            Err(error) => self.bench.status = format!("Could not keep the workbench. {error}"),
        }
    }

    pub(super) fn ask_workbench(&mut self) {
        let Some(board) = self.bench_board() else {
            return;
        };
        let known = self.memory().map(|file| host::known_runs(&file)).unwrap_or_default();
        let bench = Workbench::new(board, self.bench.tools.clone(), self.bench.hypotheses.clone(), &today()).knowing(known);
        self.bench.status = "The workbench is running on the local model.".to_string();
        self.bench.ask = Some(start(bench));
    }

    pub(super) fn poll_workbench(&mut self) {
        let Some(rx) = &self.bench.ask else {
            return;
        };
        match rx.try_recv() {
            Ok(BenchReply::Done { model, digest, bench, stopped }) => {
                self.bench.ask = None;
                self.keep_session(&model, digest, &bench, &stopped);
            }
            Ok(BenchReply::Absent(text)) => {
                self.bench.ask = None;
                self.bench.status = text;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.bench.ask = None;
                self.bench.status = "The workbench stopped without an answer.".to_string();
            }
        }
    }

    /// Keep what a finished session learned and proposed, and show it.
    pub(super) fn keep_session(&mut self, model: &str, digest: Option<String>, bench: &Workbench, stopped: &str) {
        let board = bench.board();
        let mut tools = bench.learned.clone();
        tools.extend(self.bench.tools.iter().cloned());
        for name in &bench.used {
            workbench::note_use(&mut tools, name, board);
        }
        let mut hypotheses = bench.proposed.clone();
        hypotheses.extend(self.bench.hypotheses.iter().cloned());
        self.bench.tools = tools;
        self.bench.hypotheses = hypotheses;
        self.bench.status = match self.memory() {
            Some(file) => match workbench::write_tools(&file, &self.bench.tools)
                .and_then(|()| workbench::write_hypotheses(&file, &self.bench.hypotheses))
            {
                Ok(()) => format!("{model}: {stopped}"),
                Err(error) => format!("{model}: {stopped} Could not keep the workbench. {error}"),
            },
            None => format!("{model}: {stopped} This build is not the Store package, so nothing is kept after it closes."),
        };
        self.bench.last = Some(LastSession {
            model: model.to_string(),
            digest: digest.clone(),
            stopped: stopped.to_string(),
            steps: bench.steps.clone(),
            note: bench.note.clone(),
            note_dropped: bench.note_dropped,
            proposed: bench
                .proposed
                .iter()
                .map(|hypothesis| {
                    let state = hypothesis.state().map(|state| state.word().to_string()).unwrap_or_default();
                    (hypothesis.statement_on(board), state)
                })
                .collect(),
            learned: bench.learned.iter().map(|tool| (format!("{} = {}", tool.name, tool.formula), tool.meaning.clone())).collect(),
            not_used: host::measures_not_used(bench),
            record: host::session_record(model, digest.as_deref(), stopped, &self.bench.your_call, bench),
        });
    }

    pub(super) fn open_bench_runs(&mut self) {
        let Some(paths) = pick_runs() else {
            return;
        };
        match open_runs(&paths) {
            Ok(runs) => {
                self.bench.runs = runs;
                self.bench.status.clear();
            }
            Err(error) => self.bench.status = error,
        }
    }

    pub(super) fn save_session_record(&mut self) {
        let Some(last) = &self.bench.last else {
            return;
        };
        let path = match take_save() {
            Some(queued) => queued,
            None => rfd::FileDialog::new().add_filter("Session record", &["json"]).set_file_name("workbench-session.json").save_file(),
        };
        let Some(path) = path else {
            return;
        };
        let body = serde_json::to_string_pretty(&last.record).unwrap_or_default() + "\n";
        self.bench.status = match std::fs::write(&path, body) {
            Ok(()) => "Saved the session record.".to_string(),
            Err(error) => format!("Could not save the session record. {error}"),
        };
    }

    pub(super) fn draw_workbench(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        self.load_bench_memory();
        self.poll_workbench();
        if self.bench.ask.is_some() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        }
        ui.heading(RichText::new("Workbench").color(paint.text));
        ui.label(
            RichText::new(
                "A local model that can call tools investigates the runs: it measures, builds formula tools, and proposes what each knob does. \
ScalarScope computes every number and sets every verdict. The model talks only to Ollama on this computer, and cloud models are refused.",
            )
            .color(paint.note),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Open runs…").on_hover_text("Pick run files, or a folder of runs, to weigh together").clicked() {
                self.open_bench_runs();
            }
            if !self.bench.runs.is_empty() && ui.button("Use the compared runs").clicked() {
                self.bench.runs.clear();
            }
        });
        let Some(board) = self.bench_board() else {
            ui.label(RichText::new("No inference runs are open. Open runs here, or compare two inference runs first.").color(paint.note));
            return;
        };
        self.retest(&board);
        self.draw_bench_runs(ui, &board);
        ui.separator();
        self.draw_bench_verdicts(ui, &board);
        ui.separator();
        ui.label(RichText::new("Your call first (optional)").strong().color(paint.text));
        ui.label(RichText::new("Write what you expect before you ask. It is kept with the session record.").color(paint.note));
        ui.add(egui::TextEdit::multiline(&mut self.bench.your_call).desired_rows(2).desired_width(f32::INFINITY));
        let running = self.bench.ask.is_some();
        let reviewing = self.opened.is_some();
        ui.horizontal(|ui| {
            let ask = ui.add_enabled(!running && !reviewing, egui::Button::new(RichText::new("Ask").strong()));
            let ask = if reviewing { ask.on_disabled_hover_text("A stored review is open. Close it to use the workbench.") } else { ask };
            if ask.clicked() {
                self.ask_workbench();
            }
            if running {
                ui.spinner();
            }
            if !self.bench.status.is_empty() {
                ui.label(RichText::new(&self.bench.status).color(paint.note));
            }
        });
        self.draw_last_session(ui);
        ui.separator();
        ui.label(RichText::new("Try a formula").strong().color(paint.text));
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.bench.formula).hint_text("p99 / p50").desired_width(320.0));
            if ui.button("Evaluate").clicked() {
                self.bench.formula_lines = match workbench::evaluate(&board, &self.bench.formula, &self.bench.tools) {
                    Ok(column) => column.lines(),
                    Err(reason) => vec![reason],
                };
            }
        });
        for line in &self.bench.formula_lines {
            ui.label(RichText::new(line).monospace().color(paint.text));
        }
        egui::CollapsingHeader::new("Measures").show(ui, |ui| {
            for measure in workbench::catalogue(host::MEASURES) {
                let name = if measure.args.is_empty() { measure.name.to_string() } else { format!("{}({})", measure.name, measure.args) };
                ui.label(RichText::new(format!("{name}: {}", measure.means)).color(paint.note));
            }
        });
    }

    fn draw_bench_runs(&self, ui: &mut egui::Ui, board: &Board) {
        let paint = self.paint;
        let source = if self.bench.runs.is_empty() { "the compared runs" } else { "runs opened here" };
        ui.label(RichText::new(format!("{} runs, from {source}. Hypotheses are filed under {}.", board.runs.len(), board.method)).color(paint.text));
        let runs: Vec<&InferenceRun> = if self.bench.runs.is_empty() {
            Vec::new()
        } else {
            self.bench.runs.iter().collect()
        };
        if board.varying.is_empty() {
            ui.label(RichText::new("No knob differs between these runs, so no knob can be weighed here.").color(paint.note));
        } else {
            for key in &board.varying {
                let values: Vec<String> = board
                    .runs
                    .iter()
                    .map(|run| run.knobs.get(key).map(workbench::knob_text).unwrap_or_else(|| "none".to_string()))
                    .collect();
                let from = runs.iter().find_map(|run| run.knobs.sources.get(key)).map(|source| format!(" (from {})", source.label())).unwrap_or_default();
                ui.label(RichText::new(format!("Varies: {}: {}{from}", board.label(key), values.join(", "))).color(paint.text));
            }
        }
        if !board.shared.is_empty() {
            let shared: Vec<String> = board.shared.iter().map(|(key, value)| format!("{} {}", board.label(key), workbench::knob_text(value))).collect();
            ui.label(RichText::new(format!("Shared by every run, so not tested here: {}.", shared.join(", "))).color(paint.note));
        }
    }

    fn draw_bench_verdicts(&self, ui: &mut egui::Ui, board: &Board) {
        let paint = self.paint;
        let mine: Vec<&Hypothesis> = self.bench.hypotheses.iter().filter(|hypothesis| hypothesis.method == board.method).collect();
        let hold = mine
            .iter()
            .filter(|hypothesis| matches!(self.bench.book.latest_for(&hypothesis.id), Some((_, (_, Verdict::Supported, _, _)))))
            .count();
        ui.label(RichText::new(format!("The program's verdicts: {} tried, {hold} hold", mine.len())).strong().color(paint.text));
        ui.label(
            RichText::new(format!(
                "A verdict is issued only at a checkpoint, one every {} new sets of runs, by e-BH at a 5% false discovery rate. The next checkpoint is {} sets away.",
                workbench::CHECKPOINT_EVERY,
                self.bench.book.until_next()
            ))
            .color(paint.note),
        );
        for hypothesis in mine {
            let here = hypothesis
                .evaluations
                .iter()
                .find(|evaluation| evaluation.board == board.key)
                .map(|evaluation| format!("On these runs alone: {}. {}", evaluation.state.word(), evaluation.detail))
                .unwrap_or_else(|| "Not tested on these runs.".to_string());
            let so_far = workbench::evidence(hypothesis);
            let verdict = match self.bench.book.latest_for(&hypothesis.id) {
                Some((checkpoint, (_, verdict, _, _))) => format!("Checkpoint {}: {}.", checkpoint.number, verdict.word()),
                None => "No checkpoint has judged it yet.".to_string(),
            };
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(RichText::new(hypothesis.statement_on(board)).color(paint.text));
                ui.label(RichText::new(verdict).color(paint.mark));
                ui.label(
                    RichText::new(format!(
                        "Evidence so far: {} for, {} against, from {} sets of runs.",
                        workbench::format_measure(so_far.e_for),
                        workbench::format_measure(so_far.e_against),
                        so_far.counted.len()
                    ))
                    .color(paint.note),
                );
                ui.label(RichText::new(here).color(paint.note));
            });
        }
    }

    fn draw_last_session(&mut self, ui: &mut egui::Ui) {
        let paint = self.paint;
        let mut save = false;
        if let Some(last) = &self.bench.last {
            ui.add_space(6.0);
            let digest = last.digest.as_deref().map(|digest| format!(", digest {}", &digest[..digest.len().min(12)])).unwrap_or_default();
            ui.label(RichText::new(format!("Last session: {}{digest}. {}", last.model, last.stopped)).color(paint.text));
            for (statement, state) in &last.proposed {
                ui.label(RichText::new(format!("Proposed: {statement} On these runs: {state}.")).color(paint.text));
            }
            for (formula, meaning) in &last.learned {
                ui.label(RichText::new(format!("Learned: {formula} ({meaning})")).color(paint.text));
            }
            ui.label(RichText::new("The model's words, a proposal and not a measurement:").italics().color(paint.note));
            match (&last.note, last.note_dropped) {
                (Some(note), _) => {
                    ui.label(RichText::new(note).color(paint.text));
                }
                (None, true) => {
                    ui.label(RichText::new("Its note was dropped: it carried a number or a verdict word.").color(paint.note));
                }
                (None, false) => {
                    ui.label(RichText::new("It left no note.").color(paint.note));
                }
            }
            if !last.not_used.is_empty() {
                ui.label(RichText::new(format!("Measures it did not look at: {}.", last.not_used.join(", "))).color(paint.note));
            }
            egui::CollapsingHeader::new(format!("Its {} calls", last.steps.len())).show(ui, |ui| {
                for step in &last.steps {
                    ui.label(RichText::new(format!("{} {}", step.tool, step.args)).monospace().color(paint.mark));
                    ui.label(RichText::new(&step.result).color(paint.text));
                }
            });
            save = ui.button("Save session record").on_hover_text("The record holds the model, its digest, every call and answer, and no paths").clicked();
        }
        if save {
            self.save_session_record();
        }
    }
}
