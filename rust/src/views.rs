//! Geometry for the comparison views, computed from the samples a review keeps.
//!
//! Each view answers one reading question (spec, "V. Views"): the difference plot puts B − A on
//! its own axis instead of asking the reader to subtract two curves (V6); the percentile spectrum
//! shows the tail on a log axis (C8); the threshold reads the chance of exceeding a latency
//! straight off both CDFs (V4); the quantile strip shows outcome variability as 20 dots (V3);
//! the heat map shows how the distribution moves over steps (V8). Nothing here animates (V7).

use serde::{Deserialize, Serialize};

use crate::stats::{self, Support};

/// Percentiles on the difference plot. Each is drawn only where both runs have enough samples.
pub const DIFFERENCE_PROBABILITIES: [f64; 6] = [0.5, 0.75, 0.9, 0.95, 0.99, 0.999];
/// Dots in a quantile strip; each is 5% of the samples.
pub const STRIP_DOTS: usize = 20;
/// Step bins and latency bins in a heat map.
pub const HEAT_COLUMNS: usize = 48;
pub const HEAT_ROWS: usize = 24;

/// B − A at one percentile, in ms, with its 95% block-bootstrap interval.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifferencePoint {
    pub probability: f64,
    pub estimate: f64,
    pub low: f64,
    pub high: f64,
}

/// One level of a run, in the review window's sample indices, with its median latency.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub level: f64,
}

fn supported(count: usize, probability: f64) -> bool {
    matches!(stats::quantile_support(count, probability), Support::Ranks { .. })
}

/// B − A at each of [`DIFFERENCE_PROBABILITIES`] both sides can support. Each side is one or
/// more runs; with several, the interval resamples runs as well as samples.
pub fn difference(a: &[&[f64]], b: &[&[f64]]) -> Vec<DifferencePoint> {
    let count = |runs: &[&[f64]]| runs.iter().map(|run| run.len()).sum::<usize>();
    let (count_a, count_b) = (count(a), count(b));
    DIFFERENCE_PROBABILITIES
        .iter()
        .filter(|probability| supported(count_a, **probability) && supported(count_b, **probability))
        .filter_map(|probability| {
            let interval = stats::runs_interval(a, b, *probability, stats::Contrast::Difference, stats::BOOTSTRAP_SEED)?;
            Some(DifferencePoint {
                probability: *probability,
                estimate: interval.estimate,
                low: interval.low,
                high: interval.high,
            })
        })
        .collect()
}

/// The spectrum's x for a probability: the number of nines, −log10(1 − p). p50 is 0.3, p90 is 1,
/// p99 is 2, p99.9 is 3.
pub fn nines(probability: f64) -> f64 {
    -(1.0 - probability).max(1e-12).log10()
}

/// The percentile spectrum of a run: (nines, latency) from p0 up to the highest percentile its
/// sample count supports, on a grid of tenth-nines.
pub fn spectrum(values: &[Option<f64>]) -> Vec<[f64; 2]> {
    let mut sorted: Vec<f64> = values.iter().flatten().copied().filter(|value| value.is_finite()).collect();
    if sorted.is_empty() {
        return Vec::new();
    }
    sorted.sort_by(f64::total_cmp);
    (0..=40)
        .map(|tenth| 1.0 - 10f64.powf(-(tenth as f64) / 10.0))
        .filter(|probability| *probability == 0.0 || supported(sorted.len(), *probability))
        .filter_map(|probability| {
            let rank = ((probability * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
            Some([nines(probability), sorted[rank - 1]])
        })
        .collect()
}

/// The share of samples above `threshold`.
pub fn exceedance(values: &[Option<f64>], threshold: f64) -> Option<f64> {
    let finite: Vec<f64> = values.iter().flatten().copied().filter(|value| value.is_finite()).collect();
    if finite.is_empty() {
        return None;
    }
    Some(finite.iter().filter(|value| **value > threshold).count() as f64 / finite.len() as f64)
}

/// [`STRIP_DOTS`] quantiles at the middle of each 5% slice: a quantile dotplot laid flat.
pub fn quantile_strip(values: &[Option<f64>]) -> Vec<f64> {
    let mut finite: Vec<f64> = values.iter().flatten().copied().filter(|value| value.is_finite()).collect();
    if finite.len() < STRIP_DOTS {
        return Vec::new();
    }
    finite.sort_by(f64::total_cmp);
    (0..STRIP_DOTS)
        .map(|dot| {
            let probability = (dot as f64 + 0.5) / STRIP_DOTS as f64;
            let rank = ((probability * finite.len() as f64).ceil() as usize).clamp(1, finite.len());
            finite[rank - 1]
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatCell {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    /// The share of this column's samples in this cell, 0 to 1.
    pub share: f64,
}

/// The latency range a pair of heat maps shares: p0.5 to p99.5 of both runs together, so one
/// spike does not flatten every other cell.
pub fn heat_range(a: &[Option<f64>], b: &[Option<f64>]) -> Option<(f64, f64)> {
    let mut all: Vec<f64> = a.iter().chain(b).flatten().copied().filter(|value| value.is_finite()).collect();
    if all.len() < 2 {
        return None;
    }
    let low = stats::quantile_unsorted(&mut all, 0.005)?;
    let high = stats::quantile_unsorted(&mut all, 0.995)?;
    (high > low).then_some((low, high))
}

/// A step × latency heat map of one run. Each column is normalised to its own sample count, so a
/// column reads as that stretch's distribution. Samples outside `range` go to the edge rows.
pub fn heat_cells(values: &[Option<f64>], range: (f64, f64)) -> Vec<HeatCell> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let columns = HEAT_COLUMNS.min(n);
    let (low, high) = range;
    let row_height = (high - low) / HEAT_ROWS as f64;
    let mut cells = Vec::new();
    for column in 0..columns {
        let from = column * n / columns;
        let to = (column + 1) * n / columns;
        let mut counts = [0_usize; HEAT_ROWS];
        let mut total = 0;
        for value in values[from..to].iter().flatten().filter(|value| value.is_finite()) {
            let row = (((value - low) / row_height).floor().max(0.0) as usize).min(HEAT_ROWS - 1);
            counts[row] += 1;
            total += 1;
        }
        if total == 0 {
            continue;
        }
        for (row, count) in counts.iter().enumerate().filter(|(_, count)| **count > 0) {
            cells.push(HeatCell {
                x0: from as f64,
                x1: to as f64,
                y0: low + row as f64 * row_height,
                y1: low + (row + 1) as f64 * row_height,
                share: *count as f64 / total as f64,
            });
        }
    }
    cells
}

/// A run's levels moved into the review window: `skip` samples were cut before it and it is
/// `count` samples long. `starts` are segment starts over the whole run, the first is 0.
pub fn window_segments(latency_ms: &[f64], starts: &[usize], skip: usize, count: usize) -> Vec<Segment> {
    let n = latency_ms.len();
    starts
        .iter()
        .enumerate()
        .filter_map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(n);
            let (from, to) = ((*start).max(skip), end.min(skip + count));
            if from >= to {
                return None;
            }
            let mut level: Vec<f64> = latency_ms[*start..end].iter().copied().filter(|value| value.is_finite()).collect();
            let level = stats::quantile_unsorted(&mut level, 0.5)?;
            Some(Segment {
                start: from - skip,
                end: to - skip,
                level,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn some(values: &[f64]) -> Vec<Option<f64>> {
        values.iter().map(|value| Some(*value)).collect()
    }

    #[test]
    fn the_difference_skips_percentiles_too_few_samples_support() {
        let a: Vec<f64> = (0..400).map(|i| 10.0 + (i % 9) as f64 * 0.1).collect();
        let b: Vec<f64> = a.iter().map(|value| value - 2.0).collect();
        let points = difference(&[&a], &[&b]);
        let probabilities: Vec<f64> = points.iter().map(|point| point.probability).collect();
        assert_eq!(probabilities, vec![0.5, 0.75, 0.9, 0.95, 0.99]);
        assert!(points.iter().all(|point| (point.estimate + 2.0).abs() < 1e-9 && point.low <= point.high));
    }

    #[test]
    fn nines_count_the_tail() {
        assert!((nines(0.9) - 1.0).abs() < 1e-12);
        assert!((nines(0.99) - 2.0).abs() < 1e-9);
        assert_eq!(nines(0.0), 0.0);
    }

    #[test]
    fn the_spectrum_stops_where_support_ends() {
        let values = some(&(1..=100).map(f64::from).collect::<Vec<_>>());
        let spectrum = spectrum(&values);
        assert_eq!(spectrum[0], [0.0, 1.0]);
        // 100 samples bound p90 (one nine) but not p99.
        let top = spectrum.last().unwrap();
        assert!(top[0] >= 1.0 && top[0] < 2.0, "{top:?}");
        assert!(spectrum.windows(2).all(|pair| pair[0][1] <= pair[1][1]));
        assert!(super::spectrum(&[]).is_empty());
    }

    #[test]
    fn exceedance_is_the_share_above_the_threshold() {
        let values = some(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(exceedance(&values, 2.5), Some(0.5));
        assert_eq!(exceedance(&values, 4.0), Some(0.0));
        assert_eq!(exceedance(&[None], 1.0), None);
    }

    #[test]
    fn the_strip_is_twenty_middle_quantiles() {
        let values = some(&(1..=200).map(f64::from).collect::<Vec<_>>());
        let strip = quantile_strip(&values);
        assert_eq!(strip.len(), STRIP_DOTS);
        assert_eq!((strip[0], strip[19]), (5.0, 195.0));
        assert!(quantile_strip(&some(&[1.0; 5])).is_empty());
    }

    #[test]
    fn heat_columns_are_normalised_and_spikes_go_to_the_edge() {
        let mut values = some(&[10.0; 96]);
        values[5] = Some(1000.0);
        let range = heat_range(&values, &values).unwrap_or((9.0, 11.0));
        let cells = heat_cells(&values, (9.0, 11.0));
        for column in 0..HEAT_COLUMNS {
            let share: f64 = cells.iter().filter(|cell| cell.x0 == (column * 2) as f64).map(|cell| cell.share).sum();
            assert!((share - 1.0).abs() < 1e-9, "column {column}");
        }
        assert!(cells.iter().any(|cell| cell.y1 == 11.0 && cell.x0 == 4.0));
        assert!(range.0 <= 10.0);
        assert!(heat_cells(&[], (0.0, 1.0)).is_empty());
        assert!(heat_range(&some(&[3.0]), &[]).is_none());
    }

    #[test]
    fn segments_are_clipped_to_the_window() {
        let latency: Vec<f64> = (0..100).map(|i| if i < 30 { 20.0 } else { 10.0 }).collect();
        let segments = window_segments(&latency, &[0, 30], 20, 60);
        assert_eq!(
            segments,
            vec![Segment { start: 0, end: 10, level: 20.0 }, Segment { start: 10, end: 60, level: 10.0 }]
        );
        assert!(window_segments(&latency, &[0, 30], 40, 10).len() == 1);
    }
}
