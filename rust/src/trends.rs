//! History: how a project's measures moved across its logged reviews (spec H, K4).
//!
//! A project is the pair (dataset fingerprint, model fingerprint) of side B. Code and
//! environment are left out of the key, because their changes are what the history is for. A
//! review without fingerprints goes to "Unsorted".
//!
//! For each measure, the project's reviews in date order are segmented by PELT (Killick,
//! Fearnhead and Eckley 2012) with a mean-shift cost on the logarithm, scaled by one robust noise
//! estimate for the whole series. A per-segment variance is unstable on segments of a few
//! reviews, so it is not used here. Daly et al. (2020) found change-point detection over CI
//! history caught regressions sooner, with fewer false alarms, than thresholds.
//!
//! A review whose code or environment fingerprint differs from the one before it is marked. A
//! shift within one review of a mark names it: the sentence claims coincidence, not cause.

use std::collections::BTreeMap;

use crate::history::LogEntry;

/// Reviews needed before change points are looked for.
pub const MIN_REVIEWS: usize = 6;
/// The shortest stretch of reviews a segment may have.
pub const MIN_SEGMENT: usize = 3;
/// Penalty per change point, as a multiple of ln(n). With the noise-scaled squared error as the
/// cost, a change adds a location and a level, so 2 · ln(n) is the BIC penalty.
pub const PENALTY: f64 = 2.0;

/// The measures the history follows, in the order the page lists them.
pub const MEASURES: &[(&str, &str)] = &[
    ("p50", "p50 (ms)"),
    ("p90", "p90 (ms)"),
    ("p99", "p99 (ms)"),
    ("p99_over_p50", "p99 / p50"),
    ("steady_mean", "steady mean (ms)"),
    ("throughput_p50", "throughput p50"),
    ("memory_peak", "memory peak (MiB)"),
];

/// One project's reviews, oldest first.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub label: String,
    pub entries: Vec<LogEntry>,
}

/// Group logged inference reviews into projects. Bundle openings and reviews of other kinds are
/// left out. Projects with the most reviews come first.
pub fn projects(entries: &[LogEntry]) -> Vec<Project> {
    let mut groups: BTreeMap<String, Vec<LogEntry>> = BTreeMap::new();
    for entry in entries.iter().filter(|entry| entry.kind == "compare" && entry.alignment != "loss" && entry.alignment != "geometry") {
        let label = match &entry.fingerprints {
            Some(prints) if !prints.dataset.is_empty() || !prints.model.is_empty() => {
                format!("dataset {} · model {}", short(&prints.dataset), short(&prints.model))
            }
            _ => "Unsorted".to_string(),
        };
        groups.entry(label).or_default().push(entry.clone());
    }
    let mut out: Vec<Project> = groups
        .into_iter()
        .map(|(label, mut entries)| {
            entries.sort_by(|a, b| a.finished_at.cmp(&b.finished_at));
            Project { label, entries }
        })
        .collect();
    out.sort_by(|a, b| b.entries.len().cmp(&a.entries.len()).then_with(|| a.label.cmp(&b.label)));
    out
}

fn short(print: &str) -> String {
    if print.is_empty() {
        "unknown".to_string()
    } else {
        print.chars().take(8).collect()
    }
}

/// What changed on a review, against the one before it.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    /// The review's index in the project, oldest first.
    pub at: usize,
    pub what: &'static str,
}

/// One measure across a project's reviews.
#[derive(Clone, Debug, PartialEq)]
pub struct Trend {
    pub measure: String,
    /// (review index, value) for the reviews that recorded this measure.
    pub points: Vec<(usize, f64)>,
    /// Review indices where a new level starts.
    pub shifts: Vec<usize>,
    pub marks: Vec<Mark>,
    /// The program's sentences about the shifts.
    pub sentences: Vec<String>,
    /// False when there were too few reviews to look for change points.
    pub searched: bool,
}

/// Code and environment changes between consecutive reviews.
pub fn marks(project: &Project) -> Vec<Mark> {
    let mut out = Vec::new();
    for (index, pair) in project.entries.windows(2).enumerate() {
        let (Some(before), Some(after)) = (&pair[0].fingerprints, &pair[1].fingerprints) else {
            continue;
        };
        if !after.environment.is_empty() && before.environment != after.environment {
            out.push(Mark { at: index + 1, what: "environment" });
        }
        if !after.code.is_empty() && before.code != after.code {
            out.push(Mark { at: index + 1, what: "code" });
        }
    }
    out
}

/// One measure's trend over the project's reviews.
pub fn trend(project: &Project, measure: &str) -> Trend {
    let points: Vec<(usize, f64)> = project
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            entry.measures.as_ref()?.get(measure).copied().filter(|value| value.is_finite() && *value > 0.0).map(|value| (index, value))
        })
        .collect();
    let marks = marks(project);
    let searched = points.len() >= MIN_REVIEWS;
    let shifts: Vec<usize> = if searched {
        let logs: Vec<f64> = points.iter().map(|(_, value)| value.ln()).collect();
        mean_shift_pelt(&logs, PENALTY, MIN_SEGMENT).into_iter().map(|cut| points[cut].0).collect()
    } else {
        Vec::new()
    };
    let label = MEASURES.iter().find(|(name, _)| *name == measure).map_or(measure, |(name, _)| name);
    let sentences = shifts
        .iter()
        .map(|at| {
            let date: String = project.entries[*at].finished_at.chars().take(10).collect();
            let near: Vec<&str> = marks.iter().filter(|mark| mark.at.abs_diff(*at) <= 1).map(|mark| mark.what).collect();
            match near.as_slice() {
                [] => format!("{label} shifted at {date}."),
                [one] => format!("{label} shifted at {date}, with {} {one} change.", if one.starts_with('e') { "an" } else { "a" }),
                _ => format!("{label} shifted at {date}, with a code and an environment change."),
            }
        })
        .collect();
    Trend { measure: measure.to_string(), points, shifts, marks, sentences, searched }
}

/// Change points of `values` by PELT with a mean-shift cost: the squared error of each segment
/// around its mean, divided by one noise variance for the whole series, and `beta · ln(n)` per
/// change. The noise is the robust spread of first differences (1.4826 × MAD / √2), so a level
/// shift does not inflate it.
pub fn mean_shift_pelt(values: &[f64], beta: f64, min_segment: usize) -> Vec<usize> {
    let n = values.len();
    let min_segment = min_segment.max(1);
    if n < 2 * min_segment {
        return Vec::new();
    }
    let mut differences: Vec<f64> = values.windows(2).map(|pair| (pair[1] - pair[0]).abs()).collect();
    differences.sort_by(f64::total_cmp);
    let mad = differences[differences.len() / 2];
    // A floor so a perfectly flat series still has a finite cost: 0.5% on the log scale.
    let sigma = (1.4826 * mad / std::f64::consts::SQRT_2).max(0.005);
    let variance = sigma * sigma;
    let mut sum = vec![0.0; n + 1];
    let mut squares = vec![0.0; n + 1];
    for (index, value) in values.iter().enumerate() {
        sum[index + 1] = sum[index] + value;
        squares[index + 1] = squares[index] + value * value;
    }
    let cost = |from: usize, to: usize| {
        let length = (to - from) as f64;
        let total = sum[to] - sum[from];
        ((squares[to] - squares[from]) - total * total / length) / variance
    };
    let penalty = beta * (n as f64).ln();
    let mut best = vec![f64::INFINITY; n + 1];
    best[0] = -penalty;
    let mut previous = vec![0_usize; n + 1];
    let mut candidates: Vec<usize> = vec![0];
    for end in min_segment..=n {
        let mut chosen = (f64::INFINITY, 0);
        for &start in candidates.iter().filter(|start| end - **start >= min_segment) {
            let total = best[start] + cost(start, end) + penalty;
            if total < chosen.0 {
                chosen = (total, start);
            }
        }
        best[end] = chosen.0;
        previous[end] = chosen.1;
        let floor = best[end];
        candidates.retain(|start| end - *start < min_segment || best[*start] + cost(*start, end) <= floor);
        if end + min_segment <= n {
            candidates.push(end);
        }
    }
    let mut changes = Vec::new();
    let mut at = n;
    while at > 0 {
        let start = previous[at];
        if start > 0 {
            changes.push(start);
        }
        at = start;
    }
    changes.reverse();
    changes
}

/// What a review records about side B for the history: its fingerprints, when it was a RunTrace,
/// and the built-in measures the workbench computes, those the run can supply.
pub fn entry_facts(
    run: &crate::open::InferenceRun,
    rule: crate::stats::AnomalyRule,
) -> (Option<crate::history::EntryFingerprints>, Option<BTreeMap<String, f64>>) {
    let fingerprints = run.trace.as_ref().map(|trace| crate::history::EntryFingerprints {
        dataset: trace.fingerprints.dataset.clone(),
        model: trace.fingerprints.model.clone(),
        code: trace.fingerprints.code.clone(),
        environment: trace.fingerprints.environment.clone(),
    });
    let mut single = run.clone();
    single.replicates.clear();
    let board = crate::workbench::board(vec![single], rule);
    let mut measures = BTreeMap::new();
    for (name, _) in MEASURES {
        if let Ok(column) = workbench::evaluate(&board, name, &[]) {
            if let Some((_, Ok(value))) = column.values.first() {
                measures.insert((*name).to_string(), *value);
            }
        }
    }
    (fingerprints, (!measures.is_empty()).then_some(measures))
}
