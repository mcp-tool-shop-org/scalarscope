//! Comparison statistics that say how sure they are.
//!
//! The grounding is in `docs/parity-and-beyond.spec.md`. Latency samples within a run are
//! not independent (C4, block bootstrap pointer), timing noise is not normal (C5), a
//! percentile needs enough samples behind it (C7), and a change is stated as a ratio with an
//! interval (C1, C2). Every random draw comes from a fixed seed, so the same two files always
//! give the same intervals (K5).

use crate::readings::Band;

/// The seed for every bootstrap. Fixed so a review repeats exactly; recorded in the bundle.
pub const BOOTSTRAP_SEED: u64 = 0x5CA1_A5C0_9E_2026;
/// Bootstrap replicates for an interval.
pub const BOOTSTRAP_REPLICATES: usize = 1000;
/// Two-sided confidence for every interval on this page.
pub const CONFIDENCE: f64 = 0.95;
/// Half-width of the centred window for the spread band: 11 samples.
pub const BAND_RADIUS: usize = 5;
/// A sample is an anomaly when it is this many scaled MADs from the median.
pub const MAD_LIMIT: f64 = 5.0;
/// Consistency constant: a MAD times this estimates a normal standard deviation.
const MAD_SCALE: f64 = 1.4826;
/// Consistency constant for the mean absolute deviation, used when the MAD is 0.
const MEAN_ABS_SCALE: f64 = 1.2533;

/// SplitMix64: small, fast, and the same on every machine.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A whole number in `0..bound`.
    pub fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

/// The block length for a moving-block bootstrap of `n` samples: the cube root, at least 1.
pub fn block_length(n: usize) -> usize {
    ((n as f64).cbrt().ceil() as usize).clamp(1, n.max(1))
}

/// One moving-block resample of `values`, the same length, with blocks wrapping at the end.
pub fn block_resample(values: &[f64], block: usize, rng: &mut Rng, out: &mut Vec<f64>) {
    out.clear();
    let n = values.len();
    if n == 0 {
        return;
    }
    while out.len() < n {
        let start = rng.below(n);
        for offset in 0..block.min(n - out.len()) {
            out.push(values[(start + offset) % n]);
        }
    }
}

/// The nearest-rank quantile without sorting the whole slice.
pub fn quantile_unsorted(values: &mut [f64], probability: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let rank = ((probability * values.len() as f64).ceil() as usize).clamp(1, values.len());
    let (_, value, _) = values.select_nth_unstable_by(rank - 1, f64::total_cmp);
    Some(*value)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub estimate: f64,
    pub low: f64,
    pub high: f64,
}

impl Interval {
    pub fn excludes(&self, value: f64) -> bool {
        value < self.low || value > self.high
    }
}

/// B/A at one quantile, with a percentile interval from independent moving-block resamples of
/// each run. None when either side is empty or A's quantile is not positive.
pub fn ratio_interval(a: &[f64], b: &[f64], probability: f64, seed: u64) -> Option<Interval> {
    let qa = quantile_unsorted(&mut a.to_vec(), probability)?;
    let qb = quantile_unsorted(&mut b.to_vec(), probability)?;
    if qa <= 0.0 || !qa.is_finite() || !qb.is_finite() {
        return None;
    }
    let mut rng = Rng::new(seed ^ probability.to_bits());
    let (block_a, block_b) = (block_length(a.len()), block_length(b.len()));
    let (mut resample_a, mut resample_b) = (Vec::with_capacity(a.len()), Vec::with_capacity(b.len()));
    let mut ratios = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        block_resample(a, block_a, &mut rng, &mut resample_a);
        block_resample(b, block_b, &mut rng, &mut resample_b);
        let (Some(ra), Some(rb)) = (
            quantile_unsorted(&mut resample_a, probability),
            quantile_unsorted(&mut resample_b, probability),
        ) else {
            continue;
        };
        if ra > 0.0 {
            ratios.push(rb / ra);
        }
    }
    let tail = (1.0 - CONFIDENCE) / 2.0;
    Some(Interval {
        estimate: qb / qa,
        low: quantile_unsorted(&mut ratios.clone(), tail)?,
        high: quantile_unsorted(&mut ratios, 1.0 - tail)?,
    })
}

/// B − A at one quantile, with a percentile interval from independent moving-block resamples.
pub fn difference_interval(a: &[f64], b: &[f64], probability: f64, seed: u64) -> Option<Interval> {
    let qa = quantile_unsorted(&mut a.to_vec(), probability)?;
    let qb = quantile_unsorted(&mut b.to_vec(), probability)?;
    let mut rng = Rng::new(seed ^ probability.to_bits() ^ 0xD1FF);
    let (block_a, block_b) = (block_length(a.len()), block_length(b.len()));
    let (mut resample_a, mut resample_b) = (Vec::with_capacity(a.len()), Vec::with_capacity(b.len()));
    let mut differences = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        block_resample(a, block_a, &mut rng, &mut resample_a);
        block_resample(b, block_b, &mut rng, &mut resample_b);
        if let (Some(ra), Some(rb)) = (quantile_unsorted(&mut resample_a, probability), quantile_unsorted(&mut resample_b, probability)) {
            differences.push(rb - ra);
        }
    }
    let tail = (1.0 - CONFIDENCE) / 2.0;
    Some(Interval {
        estimate: qb - qa,
        low: quantile_unsorted(&mut differences.clone(), tail)?,
        high: quantile_unsorted(&mut differences, 1.0 - tail)?,
    })
}

/// Runs per side at which an interval covers run-to-run variation well enough to drop
/// "indicative" (C1, C6).
pub const MIN_RUNS: usize = 3;

/// How B is set against A at one quantile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contrast {
    /// B / A.
    Ratio,
    /// B − A.
    Difference,
}

fn pooled_quantile(runs: &[&[f64]], probability: f64) -> Option<f64> {
    let mut pooled: Vec<f64> = runs.iter().flat_map(|run| run.iter().copied()).collect();
    quantile_unsorted(&mut pooled, probability)
}

/// B against A at one quantile when a side has several runs. Each replicate draws that side's
/// runs with replacement, block-resamples each drawn run, pools them, and takes the quantile, so
/// the interval carries run-to-run variation as well as the variation within a run (Kalibera and
/// Jones 2013). With one run per side it is exactly [`ratio_interval`] or [`difference_interval`].
pub fn runs_interval(a: &[&[f64]], b: &[&[f64]], probability: f64, contrast: Contrast, seed: u64) -> Option<Interval> {
    if a.len() == 1 && b.len() == 1 {
        return match contrast {
            Contrast::Ratio => ratio_interval(a[0], b[0], probability, seed),
            Contrast::Difference => difference_interval(a[0], b[0], probability, seed),
        };
    }
    if a.is_empty() || b.is_empty() || a.iter().chain(b).any(|run| run.is_empty()) {
        return None;
    }
    let combine = |qa: f64, qb: f64| match contrast {
        Contrast::Ratio => (qa > 0.0).then(|| qb / qa),
        Contrast::Difference => Some(qb - qa),
    };
    let estimate = combine(pooled_quantile(a, probability)?, pooled_quantile(b, probability)?)?;
    let mut rng = Rng::new(seed ^ probability.to_bits() ^ 0x2E9_11CA7E);
    let draw = |runs: &[&[f64]], rng: &mut Rng| -> Option<f64> {
        let mut pooled = Vec::new();
        let mut resample = Vec::new();
        for _ in 0..runs.len() {
            let run = runs[rng.below(runs.len())];
            block_resample(run, block_length(run.len()), rng, &mut resample);
            pooled.extend_from_slice(&resample);
        }
        quantile_unsorted(&mut pooled, probability)
    };
    let mut values = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        if let (Some(qa), Some(qb)) = (draw(a, &mut rng), draw(b, &mut rng)) {
            if let Some(value) = combine(qa, qb) {
                values.push(value);
            }
        }
    }
    let tail = (1.0 - CONFIDENCE) / 2.0;
    Some(Interval {
        estimate,
        low: quantile_unsorted(&mut values.clone(), tail)?,
        high: quantile_unsorted(&mut values, 1.0 - tail)?,
    })
}

/// The relative spread of a run, (p90 − p10) / p50, so a faster run with the same proportional
/// jitter is not called steadier.
pub fn relative_spread(values: &mut [f64]) -> Option<f64> {
    let high = quantile_unsorted(values, 0.9)?;
    let low = quantile_unsorted(values, 0.1)?;
    let middle = quantile_unsorted(values, 0.5)?;
    (middle > 0.0).then(|| (high - low) / middle)
}

/// B's relative spread over A's, with a block-bootstrap interval. None when A's spread is 0.
pub fn spread_ratio_interval(a: &[f64], b: &[f64], seed: u64) -> Option<Interval> {
    let width = |values: &mut Vec<f64>| -> Option<f64> { relative_spread(values) };
    let wa = width(&mut a.to_vec())?;
    let wb = width(&mut b.to_vec())?;
    if wa <= 0.0 {
        return None;
    }
    let mut rng = Rng::new(seed ^ 0x5157_EAD);
    let (block_a, block_b) = (block_length(a.len()), block_length(b.len()));
    let (mut resample_a, mut resample_b) = (Vec::new(), Vec::new());
    let mut ratios = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        block_resample(a, block_a, &mut rng, &mut resample_a);
        block_resample(b, block_b, &mut rng, &mut resample_b);
        if let (Some(ra), Some(rb)) = (width(&mut resample_a), width(&mut resample_b)) {
            if ra > 0.0 {
                ratios.push(rb / ra);
            }
        }
    }
    let tail = (1.0 - CONFIDENCE) / 2.0;
    Some(Interval {
        estimate: wb / wa,
        low: quantile_unsorted(&mut ratios.clone(), tail)?,
        high: quantile_unsorted(&mut ratios, 1.0 - tail)?,
    })
}

/// Whether a quantile has enough samples behind it to be printed as a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Support {
    /// The distribution-free interval is the `low`-th to `high`-th smallest sample (1-based).
    Ranks { low: usize, high: usize },
    /// No such interval exists with this many samples; `needed` samples would give one.
    TooFew { needed: usize },
}

/// The binomial order-statistic interval for the `probability` quantile of `n` samples at
/// [`CONFIDENCE`]: the ranks `l` and `u` with P(X(l) ≤ ξ ≤ X(u)) at least that confidence,
/// each tail at most half the rest. It needs no assumption about the distribution, only that
/// samples are independent; within one run they are not, so it is a floor on the uncertainty.
pub fn quantile_support(n: usize, probability: f64) -> Support {
    match order_ranks(n, probability) {
        Some((low, high)) => Support::Ranks { low, high },
        None => {
            let mut needed = n.max(1);
            while order_ranks(needed, probability).is_none() {
                needed = needed + needed / 8 + 1;
            }
            // Walk back to the smallest count that works.
            while needed > 1 && order_ranks(needed - 1, probability).is_some() {
                needed -= 1;
            }
            Support::TooFew { needed }
        }
    }
}

fn order_ranks(n: usize, probability: f64) -> Option<(usize, usize)> {
    if n == 0 || !(0.0..1.0).contains(&probability) || probability <= 0.0 {
        return None;
    }
    let tail = (1.0 - CONFIDENCE) / 2.0;
    // cdf[k] = P(Bin(n, p) ≤ k), built in log space so large n does not underflow.
    let (ln_p, ln_q) = (probability.ln(), (1.0 - probability).ln());
    let mut log_pmf = n as f64 * ln_q;
    let mut cdf = Vec::with_capacity(n + 1);
    let mut total = 0.0;
    for k in 0..=n {
        total += log_pmf.exp();
        cdf.push(total.min(1.0));
        if k < n {
            log_pmf += ((n - k) as f64).ln() - ((k + 1) as f64).ln() + ln_p - ln_q;
        }
    }
    // Lower rank l: the largest l ≥ 1 with P(Bin ≤ l − 1) ≤ tail.
    let low = (1..=n).rev().find(|l| cdf[l - 1] <= tail)?;
    // Upper rank u: the smallest u ≤ n with P(Bin ≤ u − 1) ≥ 1 − tail.
    let high = (1..=n).find(|u| cdf[u - 1] >= 1.0 - tail)?;
    Some((low, high))
}

/// P(Bin(n, p) ≥ k), in log space so a large n does not underflow.
pub fn binomial_upper_tail(n: usize, probability: f64, k: usize) -> f64 {
    if k == 0 {
        return 1.0;
    }
    if k > n || probability <= 0.0 {
        return 0.0;
    }
    if probability >= 1.0 {
        return 1.0;
    }
    let (ln_p, ln_q) = (probability.ln(), (1.0 - probability).ln());
    let mut log_pmf = n as f64 * ln_q;
    let mut below = 0.0;
    for j in 0..k {
        below += log_pmf.exp();
        log_pmf += ((n - j) as f64).ln() - ((j + 1) as f64).ln() + ln_p - ln_q;
    }
    (1.0 - below).clamp(0.0, 1.0)
}

/// Whether B's anomaly count is higher than A's beyond chance, given each side's sample count.
/// Conditional on the total, B's count is binomial with B's share of the samples when the rates
/// are equal; the one-sided p-value is that binomial's upper tail at B's count.
pub fn more_anomalies(count_a: usize, samples_a: usize, count_b: usize, samples_b: usize) -> Option<f64> {
    if count_b <= count_a * samples_b / samples_a.max(1) || samples_a + samples_b == 0 {
        return None;
    }
    let share = samples_b as f64 / (samples_a + samples_b) as f64;
    Some(binomial_upper_tail(count_a + count_b, share, count_b))
}

/// The p10–p90 spread of a centred window of `2 * BAND_RADIUS + 1` samples around each point.
/// A point with fewer than five finite neighbours has no band.
pub fn quantile_band(values: &[Option<f64>]) -> Vec<Option<Band>> {
    (0..values.len())
        .map(|index| {
            let from = index.saturating_sub(BAND_RADIUS);
            let to = (index + BAND_RADIUS + 1).min(values.len());
            let mut window: Vec<f64> = values[from..to].iter().flatten().copied().filter(|value| value.is_finite()).collect();
            if values[index].is_none() || window.len() < 5 {
                return None;
            }
            Some(Band {
                low: quantile_unsorted(&mut window, 0.1)?,
                high: quantile_unsorted(&mut window, 0.9)?,
            })
        })
        .collect()
}

/// The median and a robust scale: 1.4826 × MAD, or 1.2533 × the mean absolute deviation when
/// more than half the samples sit on the median and the MAD is 0.
pub fn robust_center(values: &[f64]) -> Option<(f64, f64)> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let median = median_sorted(&sorted);
    let mut deviations: Vec<f64> = sorted.iter().map(|value| (value - median).abs()).collect();
    deviations.sort_by(f64::total_cmp);
    let mad = median_sorted(&deviations) * MAD_SCALE;
    let scale = if mad > 0.0 {
        mad
    } else {
        deviations.iter().sum::<f64>() / deviations.len() as f64 * MEAN_ABS_SCALE
    };
    Some((median, scale))
}

fn median_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// Indices of samples more than [`MAD_LIMIT`] robust scales from the median. A constant
/// series has none.
pub fn mad_indices(values: &[Option<f64>]) -> Vec<usize> {
    let finite: Vec<f64> = values.iter().flatten().copied().filter(|value| value.is_finite()).collect();
    let Some((median, scale)) = robust_center(&finite) else {
        return Vec::new();
    };
    if scale <= 0.0 {
        return Vec::new();
    }
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.filter(|value| (value - median).abs() > MAD_LIMIT * scale).map(|_| index))
        .collect()
}

/// Which samples count as anomalies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnomalyRule {
    /// More than [`MAD_LIMIT`] robust deviations (1.4826 × MAD) from the median. The default.
    #[default]
    Mad,
    /// More than 3 population standard deviations from the mean, the 2.0 rule. The standard
    /// deviation grows with the spikes it is meant to find (C5), so it is a setting, not the default.
    ThreeSigma,
}

impl AnomalyRule {
    /// The rule as `preferences.json` stores it (`AnomalyRule`).
    pub fn code(self) -> u8 {
        match self {
            AnomalyRule::Mad => 0,
            AnomalyRule::ThreeSigma => 1,
        }
    }

    pub fn from_code(code: u8) -> Self {
        if code == 1 { AnomalyRule::ThreeSigma } else { AnomalyRule::Mad }
    }

    /// The rule in words, for the caption and the ΔF panel.
    pub fn describe(self) -> String {
        match self {
            AnomalyRule::Mad => format!("more than {MAD_LIMIT} robust deviations (1.4826 × MAD) from the median"),
            AnomalyRule::ThreeSigma => "more than 3 population standard deviations from the mean (the 2.0 rule, chosen in Settings)".to_string(),
        }
    }
}

/// The anomalies among `values` under `rule`.
pub fn anomalies(rule: AnomalyRule, values: &[Option<f64>]) -> Vec<usize> {
    match rule {
        AnomalyRule::Mad => mad_indices(values),
        AnomalyRule::ThreeSigma => crate::readings::three_sigma_indices(values),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_generator_repeats_from_its_seed() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        assert!((0..100).all(|_| a.next_u64() == b.next_u64()));
        assert!(Rng::new(1).below(10) < 10);
    }

    #[test]
    fn block_length_is_the_cube_root() {
        assert_eq!(block_length(0), 1);
        assert_eq!(block_length(1), 1);
        assert_eq!(block_length(27), 3);
        assert_eq!(block_length(1000), 10);
        assert_eq!(block_length(1001), 11);
    }

    #[test]
    fn a_resample_keeps_length_and_draws_contiguous_blocks() {
        let values: Vec<f64> = (0..10).map(f64::from).collect();
        let mut out = Vec::new();
        block_resample(&values, 3, &mut Rng::new(3), &mut out);
        assert_eq!(out.len(), 10);
        for chunk in out.chunks(3).filter(|chunk| chunk.len() == 3) {
            assert_eq!((chunk[0] + 1.0) % 10.0, chunk[1]);
        }
        block_resample(&[], 3, &mut Rng::new(3), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn a_ratio_interval_brackets_its_estimate_and_repeats() {
        let a: Vec<f64> = (0..400).map(|i| 10.0 + (i % 7) as f64 * 0.1).collect();
        let b: Vec<f64> = a.iter().map(|value| value * 0.8).collect();
        let first = ratio_interval(&a, &b, 0.5, BOOTSTRAP_SEED).unwrap();
        assert!((first.estimate - 0.8).abs() < 1e-9);
        assert!(first.low <= first.estimate && first.estimate <= first.high);
        assert!(first.excludes(1.0));
        assert_eq!(ratio_interval(&a, &b, 0.5, BOOTSTRAP_SEED), Some(first));
        assert!(ratio_interval(&[], &b, 0.5, BOOTSTRAP_SEED).is_none());
        assert!(ratio_interval(&[0.0, 0.0], &b, 0.5, BOOTSTRAP_SEED).is_none());
    }

    #[test]
    fn the_same_run_twice_does_not_exclude_one() {
        let mut rng = Rng::new(11);
        let a: Vec<f64> = (0..500).map(|_| 10.0 + (rng.next_u64() % 1000) as f64 / 500.0).collect();
        let interval = ratio_interval(&a, &a, 0.9, BOOTSTRAP_SEED).unwrap();
        assert!(!interval.excludes(1.0), "{interval:?}");
    }

    #[test]
    fn spread_ratio_reads_a_wider_run() {
        let a: Vec<f64> = (0..300).map(|i| 10.0 + (i % 10) as f64 * 0.1).collect();
        let b: Vec<f64> = (0..300).map(|i| 10.0 + (i % 10) as f64 * 0.3).collect();
        let interval = spread_ratio_interval(&a, &b, BOOTSTRAP_SEED).unwrap();
        assert!(interval.estimate > 2.5 && interval.excludes(1.0), "{interval:?}");
        // The same proportional jitter at a lower level is the same relative spread.
        let scaled: Vec<f64> = a.iter().map(|value| value * 0.6).collect();
        let same = spread_ratio_interval(&a, &scaled, BOOTSTRAP_SEED).unwrap();
        assert!((same.estimate - 1.0).abs() < 1e-9 && !same.excludes(1.0), "{same:?}");
        assert!(spread_ratio_interval(&[5.0; 20], &b, BOOTSTRAP_SEED).is_none());
    }

    #[test]
    fn order_statistic_support_matches_the_binomial_bound() {
        // The median of 100 samples: ranks 40 and 61 at 95% (the textbook interval).
        assert_eq!(quantile_support(100, 0.5), Support::Ranks { low: 40, high: 61 });
        // p99 needs the maximum to bound it: 1 − 0.99^n ≥ 0.975, so n ≥ 368.
        assert_eq!(quantile_support(100, 0.99), Support::TooFew { needed: 368 });
        assert!(matches!(quantile_support(368, 0.99), Support::Ranks { high: 368, .. }));
        assert!(matches!(quantile_support(2000, 0.99), Support::Ranks { .. }));
        assert!(matches!(quantile_support(0, 0.5), Support::TooFew { .. }));
    }

    #[test]
    fn runs_interval_is_the_single_run_interval_for_one_run_each() {
        let a: Vec<f64> = (0..300).map(|i| 10.0 + (i % 7) as f64 * 0.1).collect();
        let b: Vec<f64> = a.iter().map(|value| value * 0.9).collect();
        assert_eq!(
            runs_interval(&[&a], &[&b], 0.5, Contrast::Ratio, BOOTSTRAP_SEED),
            ratio_interval(&a, &b, 0.5, BOOTSTRAP_SEED)
        );
        assert_eq!(
            runs_interval(&[&a], &[&b], 0.9, Contrast::Difference, BOOTSTRAP_SEED),
            difference_interval(&a, &b, 0.9, BOOTSTRAP_SEED)
        );
        assert!(runs_interval(&[], &[&b], 0.5, Contrast::Ratio, BOOTSTRAP_SEED).is_none());
    }

    #[test]
    fn run_to_run_variation_widens_the_interval() {
        // Three runs per side whose levels differ from run to run by about 5%.
        let run = |level: f64, seed: u64| -> Vec<f64> {
            let mut rng = Rng::new(seed);
            (0..300).map(|_| level * (1.0 + ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * 0.02)).collect()
        };
        let a = [run(10.0, 1), run(10.5, 2), run(9.5, 3)];
        let b = [run(9.0, 4), run(9.45, 5), run(8.55, 6)];
        let refs_a: Vec<&[f64]> = a.iter().map(Vec::as_slice).collect();
        let refs_b: Vec<&[f64]> = b.iter().map(Vec::as_slice).collect();
        let several = runs_interval(&refs_a, &refs_b, 0.5, Contrast::Ratio, BOOTSTRAP_SEED).unwrap();
        let single = ratio_interval(&a[0], &b[0], 0.5, BOOTSTRAP_SEED).unwrap();
        assert!(several.high - several.low > 3.0 * (single.high - single.low), "{several:?} {single:?}");
        assert!(several.low <= 0.9 && 0.9 <= several.high, "{several:?}");
    }

    #[test]
    fn the_binomial_tail_and_the_anomaly_test() {
        assert!((binomial_upper_tail(10, 0.5, 10) - 0.5_f64.powi(10)).abs() < 1e-12);
        assert!((binomial_upper_tail(2, 0.5, 1) - 0.75).abs() < 1e-12);
        assert_eq!(binomial_upper_tail(5, 0.3, 0), 1.0);
        assert_eq!(binomial_upper_tail(5, 0.3, 6), 0.0);
        // Equal halves, 12 anomalies against 1: B's excess is far beyond chance.
        assert!(more_anomalies(1, 1000, 12, 1000).unwrap() < 0.01);
        // 2 against 1 is not.
        assert!(more_anomalies(1, 1000, 2, 1000).unwrap() > 0.05);
        assert!(more_anomalies(3, 1000, 3, 1000).is_none());
    }

    #[test]
    fn the_band_is_the_p10_to_p90_of_a_centred_window() {
        let values: Vec<Option<f64>> = (0..20).map(|i| Some(i as f64)).collect();
        let band = quantile_band(&values);
        let middle = band[10].unwrap();
        // Window 5..=15: p10 is the 2nd smallest (6), p90 the 10th (14).
        assert_eq!((middle.low, middle.high), (6.0, 14.0));
        assert!(band[0].is_some(), "the edge has six neighbours");
        let sparse = vec![Some(1.0), None, None, None, Some(2.0)];
        assert!(quantile_band(&sparse).iter().all(Option::is_none));
    }

    #[test]
    fn mad_marks_a_spike_and_leaves_a_flat_run_alone() {
        let mut values: Vec<Option<f64>> = (0..50).map(|i| Some(10.0 + (i % 5) as f64 * 0.1)).collect();
        values[20] = Some(40.0);
        assert_eq!(mad_indices(&values), vec![20]);
        assert!(mad_indices(&[Some(3.0); 10]).is_empty());
        assert!(mad_indices(&[]).is_empty());
        // More than half on the median: the MAD is 0 and the mean absolute deviation scales.
        let mostly_flat: Vec<Option<f64>> = [24.0; 14].iter().chain(&[40.0, 34.0, 30.0, 28.0, 26.0, 25.0]).map(|v| Some(*v)).collect();
        assert_eq!(mad_indices(&mostly_flat), vec![14]);
    }
}
