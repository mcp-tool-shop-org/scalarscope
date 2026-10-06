//! Where a run settles, if it does.
//!
//! Barrett et al. (2017) found that only about half of benchmark runs warm up and settle; the
//! rest are flat, slow down, or never reach a steady state (W1). So the steady state is a tested
//! outcome here. PELT (Killick, Fearnhead and Eckley 2012) segments the log-latency series with a
//! normal mean-and-variance cost (W2), the run is classified by its segments, and the steady
//! start is reported as a range over three penalties instead of one step (W3). The 2.0 window
//! heuristic stays as a cross-check (W4).

/// Penalty multipliers on ln(n) per change point: the base, and the two used for the range.
pub const PENALTIES: [f64; 3] = [3.0, 4.0, 6.0];
/// The multiplier whose segmentation sets the shape.
pub const BASE_PENALTY: f64 = 4.0;
/// The shortest segment PELT may cut.
pub const MIN_SEGMENT: usize = 5;
/// A run shorter than this is too short to classify.
pub const MIN_SAMPLES: usize = 20;
/// The final segment must cover this share of the run to count as a steady state.
pub const MIN_STEADY_SHARE: f64 = 0.25;
/// Two segment medians closer than this ratio (2%) are the same level, or closer than the run's
/// own noise when that is larger: one robust standard deviation of the final segment's log-latency.
pub const EQUIVALENT_SHARE: f64 = 0.02;
/// Variance floor on log-latency, so a constant segment has a finite cost.
const VARIANCE_FLOOR: f64 = 1e-8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// One level from start to end.
    Flat,
    /// Slower at first, then settles at its fastest level.
    Warmup,
    /// Settles at a level slower than an earlier one.
    Slowdown,
    /// The last level is too short to call a steady state.
    NoSteadyState,
    /// Too few samples to tell.
    TooShort,
}

impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Shape::Flat => "flat",
            Shape::Warmup => "warmup",
            Shape::Slowdown => "slowdown",
            Shape::NoSteadyState => "no steady state",
            Shape::TooShort => "too short to tell",
        }
    }

    /// Only a flat run or a run that warmed up has a stabilization time to compare.
    pub fn settles(self) -> bool {
        matches!(self, Shape::Flat | Shape::Warmup)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunShape {
    pub shape: Shape,
    /// Segment start indices, the first is 0.
    pub segments: Vec<usize>,
    /// The steady start index at the base penalty, when the run settles.
    pub steady: Option<usize>,
    /// The lowest and highest steady start index across [`PENALTIES`], when every one settles.
    pub steady_range: Option<(usize, usize)>,
}

/// Change points (segment starts after the first) of `values` by PELT with a normal
/// mean-and-variance cost and penalty `beta · ln(n)` per change.
pub fn pelt(values: &[f64], beta: f64) -> Vec<usize> {
    let n = values.len();
    if n < 2 * MIN_SEGMENT {
        return Vec::new();
    }
    let mut sum = vec![0.0; n + 1];
    let mut squares = vec![0.0; n + 1];
    for (index, value) in values.iter().enumerate() {
        sum[index + 1] = sum[index] + value;
        squares[index + 1] = squares[index] + value * value;
    }
    let cost = |from: usize, to: usize| {
        let length = (to - from) as f64;
        let mean = (sum[to] - sum[from]) / length;
        let variance = ((squares[to] - squares[from]) / length - mean * mean).max(VARIANCE_FLOOR);
        length * variance.ln()
    };
    let penalty = beta * (n as f64).ln();
    let mut best = vec![f64::INFINITY; n + 1];
    best[0] = -penalty;
    let mut previous = vec![0_usize; n + 1];
    let mut candidates: Vec<usize> = vec![0];
    for end in MIN_SEGMENT..=n {
        let mut chosen = (f64::INFINITY, 0);
        for &start in candidates.iter().filter(|start| end - **start >= MIN_SEGMENT) {
            let total = best[start] + cost(start, end) + penalty;
            if total < chosen.0 {
                chosen = (total, start);
            }
        }
        best[end] = chosen.0;
        previous[end] = chosen.1;
        // Prune starts that can never win again; keep ones still too close to `end` to test.
        let floor = best[end];
        candidates.retain(|start| end - *start < MIN_SEGMENT || best[*start] + cost(*start, end) <= floor);
        if end + MIN_SEGMENT <= n {
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

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// Classify one segmentation. Returns the shape and, when the run settles, the index where the
/// final level begins (merging trailing segments at the same level).
fn classify(values: &[f64], changes: &[usize]) -> (Shape, Option<usize>) {
    let (shape, start, _) = classify_level(values, changes);
    (shape, start)
}

/// [`classify`], also returning the final level and the tolerance that defines "the same level".
fn classify_level(values: &[f64], changes: &[usize]) -> (Shape, Option<usize>, (f64, f64)) {
    let n = values.len();
    let mut starts = vec![0];
    starts.extend_from_slice(changes);
    let levels: Vec<f64> = starts
        .iter()
        .enumerate()
        .map(|(index, start)| median(&values[*start..starts.get(index + 1).copied().unwrap_or(n)]))
        .collect();
    let last = *levels.last().unwrap_or(&0.0);
    // Levels are medians of log-latency, so a ratio tolerance is a difference of logs.
    let final_start = *starts.last().unwrap_or(&0);
    let noise = crate::stats::robust_center(&values[final_start..]).map_or(0.0, |(_, scale)| scale);
    let tolerance = (1.0 + EQUIVALENT_SHARE).ln().max(noise);
    let same = |level: f64| (level - last).abs() <= tolerance;
    // The steady level starts at the first segment from which every later one is the same level.
    // A short segment with the same level on both sides is a spike inside the steady state, not a
    // level of its own, so the merge steps over it; its samples still show as anomaly marks.
    let length = |segment: usize| starts.get(segment + 1).copied().unwrap_or(n) - starts[segment];
    let mut steady_segment = levels.len() - 1;
    loop {
        if steady_segment > 0 && same(levels[steady_segment - 1]) {
            steady_segment -= 1;
        } else if steady_segment > 1 && length(steady_segment - 1) <= 2 * MIN_SEGMENT && same(levels[steady_segment - 2]) {
            steady_segment -= 2;
        } else {
            break;
        }
    }
    let steady_start = starts[steady_segment];
    if ((n - steady_start) as f64) < MIN_STEADY_SHARE * n as f64 {
        return (Shape::NoSteadyState, None, (last, tolerance));
    }
    if steady_segment == 0 {
        return (Shape::Flat, Some(0), (last, tolerance));
    }
    if levels[..steady_segment].iter().all(|level| *level > last || same(*level)) {
        (Shape::Warmup, Some(steady_start), (last, tolerance))
    } else {
        (Shape::Slowdown, None, (last, tolerance))
    }
}

/// The first index where a centred rolling median of `2 * MIN_SEGMENT + 1` samples comes within
/// `tolerance` of `level`: the earliest the run can be said to have reached its steady level.
/// A median ignores isolated spikes. Only used for a run that settles, where the series comes
/// down to its level, so the first entry is the arrival.
fn settled_from(values: &[f64], level: f64, tolerance: f64) -> Option<usize> {
    let radius = MIN_SEGMENT;
    let n = values.len();
    (0..n).find(|index| {
        let window = &values[index.saturating_sub(radius)..(index + radius + 1).min(n)];
        (median(window) - level).abs() <= tolerance
    })
}

/// The shape of a latency series. Non-positive or non-finite samples are left out of the log.
pub fn run_shape(latency_ms: &[f64]) -> RunShape {
    let logs: Vec<f64> = latency_ms.iter().filter(|value| value.is_finite() && **value > 0.0).map(|value| value.ln()).collect();
    if logs.len() < MIN_SAMPLES {
        return RunShape {
            shape: Shape::TooShort,
            segments: vec![0],
            steady: None,
            steady_range: None,
        };
    }
    let base = pelt(&logs, BASE_PENALTY);
    let (shape, steady, _) = classify_level(&logs, &base);
    let starts: Vec<Option<usize>> = PENALTIES
        .iter()
        .map(|beta| {
            let (shape, start) = classify(&logs, &pelt(&logs, *beta));
            start.filter(|_| shape.settles())
        })
        .collect();
    let steady_range = if shape.settles() && starts.iter().all(Option::is_some) {
        let found: Vec<usize> = starts.into_iter().flatten().collect();
        let low = *found.iter().min().unwrap();
        // PELT cuts where the level changes, which can be later than the point the series is
        // already within its own noise of the final level. The earliest such point bounds the
        // range from below.
        // The level and the noise come from the whole steady region the base cut found.
        let region = &logs[steady.unwrap_or(low)..];
        let (level, noise) = crate::stats::robust_center(region).unwrap_or((0.0, 0.0));
        let tolerance = (1.0 + EQUIVALENT_SHARE).ln().max(noise);
        let settled = settled_from(&logs, level, tolerance).unwrap_or(low).min(low);
        Some((settled, *found.iter().max().unwrap()))
    } else {
        None
    };
    let mut segments = vec![0];
    segments.extend(base);
    RunShape {
        shape,
        segments,
        steady,
        steady_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noisy(levels: &[(usize, f64)], seed: u64) -> Vec<f64> {
        let mut rng = crate::stats::Rng::new(seed);
        levels
            .iter()
            .flat_map(|(count, level)| (0..*count).map(|_| *level).collect::<Vec<_>>())
            .map(|level| level * (1.0 + ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * 0.02))
            .collect()
    }

    #[test]
    fn pelt_finds_one_clear_step() {
        let series: Vec<f64> = noisy(&[(60, 20.0), (140, 10.0)], 1).iter().map(|v| v.ln()).collect();
        let changes = pelt(&series, BASE_PENALTY);
        assert_eq!(changes.len(), 1, "{changes:?}");
        assert!((58..=62).contains(&changes[0]), "{changes:?}");
        assert!(pelt(&series[..8], BASE_PENALTY).is_empty());
    }

    #[test]
    fn a_run_that_settles_low_is_warmup_with_a_range() {
        let shape = run_shape(&noisy(&[(30, 40.0), (30, 25.0), (240, 12.0)], 2));
        assert_eq!(shape.shape, Shape::Warmup);
        let steady = shape.steady.unwrap();
        assert!((57..=63).contains(&steady), "{steady}");
        let (low, high) = shape.steady_range.unwrap();
        assert!(low <= steady && steady <= high);
    }

    #[test]
    fn one_level_is_flat_from_the_first_step() {
        let shape = run_shape(&noisy(&[(200, 12.0)], 3));
        assert_eq!((shape.shape, shape.steady), (Shape::Flat, Some(0)));
    }

    #[test]
    fn settling_slower_is_a_slowdown_and_has_no_stabilization_time() {
        let shape = run_shape(&noisy(&[(100, 10.0), (100, 15.0)], 4));
        assert_eq!(shape.shape, Shape::Slowdown);
        assert!(shape.steady.is_none() && shape.steady_range.is_none());
        assert!(!shape.shape.settles());
    }

    #[test]
    fn a_short_last_level_is_no_steady_state() {
        let shape = run_shape(&noisy(&[(180, 12.0), (20, 30.0)], 5));
        assert_eq!(shape.shape, Shape::NoSteadyState);
    }

    #[test]
    fn a_short_run_is_too_short_to_tell() {
        assert_eq!(run_shape(&[10.0; 10]).shape, Shape::TooShort);
        assert_eq!(Shape::TooShort.label(), "too short to tell");
    }

    #[test]
    fn a_spike_inside_the_steady_state_does_not_end_it() {
        let mut series = noisy(&[(40, 30.0), (260, 10.0)], 7);
        for index in 255..260 {
            series[index] = 16.0;
        }
        let shape = run_shape(&series);
        assert_eq!(shape.shape, Shape::Warmup, "{shape:?}");
        assert!(shape.steady.is_some_and(|start| start < 50), "{shape:?}");
    }

    #[test]
    fn two_runs_that_settle_together_have_overlapping_ranges() {
        // The same decay constant with different amplitudes: both are within noise near step 50,
        // though PELT may cut them at different segment boundaries.
        let decay = |amplitude: f64, base: f64, seed: u64| -> Vec<f64> {
            let mut rng = crate::stats::Rng::new(seed);
            (0..300)
                .map(|step| {
                    let noise = ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * 0.14;
                    base * (1.0 + noise) + amplitude * (-(step as f64) / 12.0).exp()
                })
                .collect()
        };
        let a = run_shape(&decay(30.0, 12.4, 1)).steady_range.unwrap();
        let b = run_shape(&decay(22.0, 8.1, 2)).steady_range.unwrap();
        assert!(a.0 <= b.1 && b.0 <= a.1, "{a:?} {b:?}");
    }

    #[test]
    fn levels_within_two_percent_are_one_level() {
        let shape = run_shape(&noisy(&[(100, 12.0), (100, 12.1)], 6));
        assert_eq!(shape.shape, Shape::Flat, "{shape:?}");
    }
}
