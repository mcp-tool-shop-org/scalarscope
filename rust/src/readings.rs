//! The numbers the drawing is allowed to show.
//!
//! A percentile is nearest-rank: the index is `ceil(p * n) - 1` on the sorted
//! samples. A spread band is a centered five-sample rolling mean ± the
//! population standard deviation. It is not a confidence interval.

pub const DEVIATION_RADIUS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub low: f64,
    pub high: f64,
}

pub fn percentile(sorted_ascending: &[f64], probability: f64) -> Option<f64> {
    if sorted_ascending.is_empty() {
        return None;
    }
    let mut rank = (probability * sorted_ascending.len() as f64).ceil() as i64 - 1;
    if rank < 0 {
        rank = 0;
    }
    if rank >= sorted_ascending.len() as i64 {
        rank = sorted_ascending.len() as i64 - 1;
    }
    Some(sorted_ascending[rank as usize])
}

/// Centered window, clipped at the ends. Fewer than two finite numbers leaves a gap.
pub fn deviation_band(values: &[Option<f64>]) -> Vec<Option<Band>> {
    let mut result = vec![None; values.len()];
    for index in 0..values.len() {
        let start = index.saturating_sub(DEVIATION_RADIUS);
        let end = (index + DEVIATION_RADIUS).min(values.len().saturating_sub(1));
        let mut window = Vec::new();
        for cursor in start..=end {
            if let Some(number) = values[cursor].filter(|number| number.is_finite()) {
                window.push(number);
            }
        }
        if window.len() < 2 {
            continue;
        }
        let mean = window.iter().sum::<f64>() / window.len() as f64;
        let variance = window
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / window.len() as f64;
        let standard_deviation = variance.sqrt();
        result[index] = Some(Band {
            low: mean - standard_deviation,
            high: mean + standard_deviation,
        });
    }
    result
}

/// Indices more than three population standard deviations from the window mean.
/// Fewer than three finite numbers produces no marks.
pub fn three_sigma_indices(values: &[Option<f64>]) -> Vec<usize> {
    let finite: Vec<(usize, f64)> = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            value
                .filter(|number| number.is_finite())
                .map(|number| (index, number))
        })
        .collect();
    if finite.len() < 3 {
        return Vec::new();
    }
    let mean = finite.iter().map(|(_, value)| value).sum::<f64>() / finite.len() as f64;
    let variance = finite
        .iter()
        .map(|(_, value)| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / finite.len() as f64;
    let threshold = 3.0 * variance.sqrt();
    finite
        .into_iter()
        .filter(|(_, value)| (value - mean).abs() > threshold)
        .map(|(index, _)| index)
        .collect()
}

/// First aligned step at or after the steady-state milestone. No milestone means no span.
pub fn steady_index(steps: &[i64], skip: usize, count: usize, steady_step: Option<i64>) -> Option<usize> {
    let steady = steady_step?;
    if count == 0 {
        return None;
    }
    steps
        .iter()
        .skip(skip)
        .take(count)
        .position(|step| *step >= steady)
}

/// Probability is rank / n, so the last point is 1.
pub fn empirical_cdf(sorted_ascending: &[f64]) -> Vec<(f64, f64)> {
    let count = sorted_ascending.len();
    sorted_ascending
        .iter()
        .enumerate()
        .map(|(index, value)| (*value, (index + 1) as f64 / count as f64))
        .collect()
}

pub fn finite_sorted(values: &[Option<f64>]) -> Vec<f64> {
    let mut numbers: Vec<f64> = values.iter().copied().flatten().filter(|value| value.is_finite()).collect();
    numbers.sort_by(|left, right| left.total_cmp(right));
    numbers
}
