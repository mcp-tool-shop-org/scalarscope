//! Warmup and steady-state detection for an inference latency series.
//!
//! These match the packaged detector. A series shorter than 10 samples has
//! no warmup end. Steady state is reported only when the tail after warmup
//! has at least 10 samples and a coefficient of variation under 0.1.

const MIN_WARMUP_STEPS: usize = 5;

pub fn detect_warmup_end(values: &[f64]) -> Option<usize> {
    if values.len() < MIN_WARMUP_STEPS * 2 {
        return None;
    }
    let valid: Vec<(usize, f64)> = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.is_finite().then_some((index, *value)))
        .collect();
    if valid.len() < MIN_WARMUP_STEPS * 2 {
        return None;
    }
    let window = (valid.len() / 20).max(3);
    for index in window..(valid.len() - window) {
        let previous = mean(&valid[index - window..index]);
        let next = mean(&valid[index..index + window]);
        let relative_change = (previous - next) / previous;
        if relative_change < 0.05 {
            return Some(valid[index].0);
        }
    }
    Some(MIN_WARMUP_STEPS)
}

pub fn detect_steady_start(values: &[f64], warmup_end: usize) -> Option<usize> {
    if values.len() <= warmup_end {
        return None;
    }
    let steady: Vec<f64> = values
        .iter()
        .skip(warmup_end)
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if steady.len() < 10 {
        return None;
    }
    let average = mean_slice(&steady);
    let standard_deviation = (steady.iter().map(|value| (value - average).powi(2)).sum::<f64>() / steady.len() as f64).sqrt();
    let coefficient = standard_deviation / average.max(1e-10);
    (coefficient < 0.1).then_some(warmup_end)
}

fn mean(window: &[(usize, f64)]) -> f64 {
    window.iter().map(|(_, value)| value).sum::<f64>() / window.len() as f64
}

fn mean_slice(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}
