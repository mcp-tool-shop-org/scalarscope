//! A view as an SVG file: the same data the window draws, as vectors, for a paper or a slide.

use std::fmt::Write;

use crate::review::InferenceReview;
use crate::views;

const WIDTH: f64 = 960.0;
const HEIGHT: f64 = 540.0;
const LEFT: f64 = 70.0;
const RIGHT: f64 = 30.0;
const TOP: f64 = 78.0;
const BOTTOM: f64 = 70.0;

/// Colors as hex digits without `#`, from the page's theme.
#[derive(Clone, Copy, Debug)]
pub struct Colors {
    pub left: [u8; 3],
    pub right: [u8; 3],
    pub mark: [u8; 3],
    pub note: [u8; 3],
    pub text: [u8; 3],
    pub background: [u8; 3],
}

fn hex(color: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2])
}

/// Text for an SVG text node or attribute.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// `text` in lines of at most `width` characters, broken at spaces.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// Tick positions on `low..=high`: about five steps of 1, 2 or 5 times a power of ten.
pub fn ticks(low: f64, high: f64) -> Vec<f64> {
    if !(low.is_finite() && high.is_finite()) || high <= low {
        return vec![low];
    }
    let rough = (high - low) / 5.0;
    let power = 10f64.powf(rough.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0].iter().map(|factor| factor * power).find(|step| *step >= rough).unwrap_or(10.0 * power);
    let mut value = (low / step).ceil() * step;
    let mut marks = Vec::new();
    while value <= high + step * 1e-9 {
        marks.push(if value.abs() < step * 1e-9 { 0.0 } else { value });
        value += step;
    }
    marks
}

fn tick_text(value: f64) -> String {
    let text = format!("{value:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A chart's coordinate frame.
struct Frame {
    x: (f64, f64),
    y: (f64, f64),
}

impl Frame {
    fn new(points: impl Iterator<Item = [f64; 2]>) -> Option<Frame> {
        let (mut x, mut y) = ((f64::INFINITY, f64::NEG_INFINITY), (f64::INFINITY, f64::NEG_INFINITY));
        for [px, py] in points.filter(|[px, py]| px.is_finite() && py.is_finite()) {
            x = (x.0.min(px), x.1.max(px));
            y = (y.0.min(py), y.1.max(py));
        }
        if !x.0.is_finite() {
            return None;
        }
        let pad = |(low, high): (f64, f64)| if high > low { (low, high) } else { (low - 1.0, high + 1.0) };
        let (y_low, y_high) = pad(y);
        let margin = (y_high - y_low) * 0.05;
        Some(Frame { x: pad(x), y: (y_low - margin, y_high + margin) })
    }

    fn px(&self, x: f64) -> f64 {
        LEFT + (x - self.x.0) / (self.x.1 - self.x.0) * (WIDTH - LEFT - RIGHT)
    }

    fn py(&self, y: f64) -> f64 {
        HEIGHT - BOTTOM - (y - self.y.0) / (self.y.1 - self.y.0) * (HEIGHT - TOP - BOTTOM)
    }

    fn path(&self, points: &[[f64; 2]]) -> String {
        points
            .iter()
            .filter(|[x, y]| x.is_finite() && y.is_finite())
            .map(|[x, y]| format!("{:.1},{:.1}", self.px(*x), self.py(*y)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

struct Chart {
    body: String,
    colors: Colors,
}

impl Chart {
    fn new(colors: Colors, title: &str, subtitle: &str) -> Chart {
        let mut body = String::new();
        let _ = write!(
            body,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" font-family="Segoe UI, Arial, sans-serif"><rect width="100%" height="100%" fill="{}"/><text x="{LEFT}" y="26" font-size="17" fill="{}">{}</text>"#,
            hex(colors.background),
            hex(colors.text),
            escape(title),
        );
        for (row, line) in wrap(subtitle, 130).iter().take(2).enumerate() {
            let _ = write!(body, r#"<text x="{LEFT}" y="{}" font-size="12" fill="{}">{}</text>"#, 44.0 + 15.0 * row as f64, hex(colors.note), escape(line));
        }
        Chart { body, colors }
    }

    fn axes(&mut self, frame: &Frame, x_label: &str, y_label: &str, x_ticks: &[(f64, String)]) {
        let note = hex(self.colors.note);
        let (x0, x1, y0, y1) = (LEFT, WIDTH - RIGHT, TOP, HEIGHT - BOTTOM);
        let _ = write!(self.body, r#"<g stroke="{note}" stroke-opacity="0.35" stroke-width="1">"#);
        for y in ticks(frame.y.0, frame.y.1) {
            let _ = write!(self.body, r#"<line x1="{x0}" x2="{x1}" y1="{0:.1}" y2="{0:.1}"/>"#, frame.py(y));
        }
        for (x, _) in x_ticks {
            let _ = write!(self.body, r#"<line y1="{y0}" y2="{y1}" x1="{0:.1}" x2="{0:.1}"/>"#, frame.px(*x));
        }
        let _ = write!(self.body, "</g>");
        let _ = write!(self.body, r#"<g font-size="11" fill="{note}">"#);
        for y in ticks(frame.y.0, frame.y.1) {
            let _ = write!(self.body, r#"<text x="{}" y="{:.1}" text-anchor="end">{}</text>"#, x0 - 6.0, frame.py(y) + 4.0, tick_text(y));
        }
        for (x, label) in x_ticks {
            let _ = write!(self.body, r#"<text x="{:.1}" y="{}" text-anchor="middle">{}</text>"#, frame.px(*x), y1 + 16.0, escape(label));
        }
        let _ = write!(
            self.body,
            r#"<text x="{:.1}" y="{}" text-anchor="middle">{}</text><text transform="translate(18,{:.1}) rotate(-90)" text-anchor="middle">{}</text></g>"#,
            (x0 + x1) / 2.0,
            y1 + 36.0,
            escape(x_label),
            (y0 + y1) / 2.0,
            escape(y_label)
        );
    }

    fn line(&mut self, frame: &Frame, points: &[[f64; 2]], color: [u8; 3], width: f64, dashed: bool) {
        let dash = if dashed { r#" stroke-dasharray="6 4""# } else { "" };
        let _ = write!(self.body, r#"<polyline fill="none" stroke="{}" stroke-width="{width}"{dash} points="{}"/>"#, hex(color), frame.path(points));
    }

    fn polygon(&mut self, frame: &Frame, points: &[[f64; 2]], color: [u8; 3], opacity: f64) {
        let _ = write!(self.body, r#"<polygon fill="{}" fill-opacity="{opacity:.2}" points="{}"/>"#, hex(color), frame.path(points));
    }

    fn dots(&mut self, frame: &Frame, points: &[[f64; 2]], color: [u8; 3], radius: f64) {
        for [x, y] in points.iter().filter(|[x, y]| x.is_finite() && y.is_finite()) {
            let _ = write!(self.body, r#"<circle cx="{:.1}" cy="{:.1}" r="{radius}" fill="{}"/>"#, frame.px(*x), frame.py(*y), hex(color));
        }
    }

    fn legend(&mut self, entries: &[(&str, [u8; 3])]) {
        let mut x = WIDTH - RIGHT - 10.0;
        for (name, color) in entries.iter().rev() {
            let width = 18.0 + 7.0 * name.chars().count() as f64;
            x -= width;
            let _ = write!(
                self.body,
                r#"<rect x="{x:.1}" y="{}" width="10" height="10" fill="{}"/><text x="{:.1}" y="{}" font-size="12" fill="{}">{}</text>"#,
                TOP - 18.0,
                hex(*color),
                x + 14.0,
                TOP - 9.0,
                hex(self.colors.text),
                escape(name)
            );
        }
    }

    fn finish(mut self, caption: &str) -> String {
        let _ = write!(self.body, r#"<text x="{LEFT}" y="{}" font-size="10" fill="{}">{}</text></svg>"#, HEIGHT - 12.0, hex(self.colors.note), escape(caption));
        self.body
    }
}

fn indexed(values: &[Option<f64>]) -> Vec<[f64; 2]> {
    values.iter().enumerate().filter_map(|(index, value)| value.map(|value| [index as f64, value])).collect()
}

fn step_ticks(frame: &Frame) -> Vec<(f64, String)> {
    ticks(frame.x.0, frame.x.1).into_iter().map(|x| (x, tick_text(x))).collect()
}

fn percentile_ticks(frame: &Frame) -> Vec<(f64, String)> {
    [0.0, 0.5, 0.75, 0.9, 0.95, 0.99, 0.999]
        .into_iter()
        .map(|probability| (views::nines(probability), probability))
        .filter(|(x, _)| *x >= frame.x.0 - 1e-9 && *x <= frame.x.1 + 1e-9)
        .map(|(x, probability)| (x, format!("p{}", tick_text(probability * 100.0))))
        .collect()
}

/// The series view: latency by step, the p10–p90 band, the anomaly marks.
pub fn series(review: &InferenceReview, colors: Colors) -> String {
    let (left, right) = (indexed(&review.left), indexed(&review.right));
    let Some(frame) = Frame::new(left.iter().chain(&right).copied()) else {
        return empty(colors, "No samples to draw.");
    };
    let mut chart = Chart::new(colors, &format!("{} vs {}", review.left_label, review.right_label), &review.headline);
    chart.axes(&frame, "step", "latency, ms", &step_ticks(&frame));
    for (band, color) in [(&review.left_band, colors.left), (&review.right_band, colors.right)] {
        for (index, pair) in band.windows(2).enumerate() {
            if let (Some(here), Some(next)) = (pair[0], pair[1]) {
                let (x, x1) = (index as f64, (index + 1) as f64);
                chart.polygon(&frame, &[[x, here.low], [x1, next.low], [x1, next.high], [x, here.high]], color, 0.25);
            }
        }
    }
    chart.line(&frame, &left, colors.left, 1.5, false);
    chart.line(&frame, &right, colors.right, 1.5, false);
    for (values, marks) in [(&review.left, &review.left_marks), (&review.right, &review.right_marks)] {
        let points: Vec<[f64; 2]> = marks.iter().filter_map(|index| values.get(*index).copied().flatten().map(|value| [*index as f64, value])).collect();
        chart.dots(&frame, &points, colors.mark, 3.5);
    }
    chart.legend(&[(&review.left_label, colors.left), (&review.right_label, colors.right), ("anomaly", colors.mark)]);
    chart.finish("The band is the p10–p90 of an 11-sample window, a spread of the samples, not a confidence interval.")
}

/// Each run from its first sample, with where it settles shaded.
pub fn warmup(review: &InferenceReview, colors: Colors) -> String {
    let full = |lead: &[f64], window: &[Option<f64>]| -> Vec<Option<f64>> { lead.iter().map(|value| Some(*value)).chain(window.iter().copied()).collect() };
    let (left, right) = (indexed(&full(&review.left_lead, &review.left)), indexed(&full(&review.right_lead, &review.right)));
    let Some(frame) = Frame::new(left.iter().chain(&right).copied()) else {
        return empty(colors, "No samples to draw.");
    };
    let mut chart = Chart::new(colors, &format!("Warmup: {} vs {}", review.left_label, review.right_label), "Shaded: where each run settles");
    chart.axes(&frame, "sample from the run's first", "latency, ms", &step_ticks(&frame));
    for (settle, color) in [(review.left_settle, colors.left), (review.right_settle, colors.right)] {
        if let Some((low, high)) = settle {
            let (low, high) = (low as f64, high.max(low) as f64 + 0.5);
            chart.polygon(&frame, &[[low, frame.y.0], [high, frame.y.0], [high, frame.y.1], [low, frame.y.1]], color, 0.18);
        }
    }
    chart.line(&frame, &left, colors.left, 1.5, false);
    chart.line(&frame, &right, colors.right, 1.5, false);
    chart.legend(&[(&review.left_label, colors.left), (&review.right_label, colors.right)]);
    chart.finish("A range when the steady start is detected; one step when the file states it.")
}

/// The cumulative distribution, with a threshold.
pub fn distribution(review: &InferenceReview, colors: Colors, threshold: Option<f64>) -> String {
    let as_points = |cdf: &[(f64, f64)]| cdf.iter().map(|(x, p)| [*x, *p]).collect::<Vec<_>>();
    let (left, right) = (as_points(&review.left_cdf), as_points(&review.right_cdf));
    let Some(frame) = Frame::new(left.iter().chain(&right).copied()) else {
        return empty(colors, "No samples to draw.");
    };
    let mut chart = Chart::new(colors, &format!("Distribution: {} vs {}", review.left_label, review.right_label), &review.headline);
    chart.axes(&frame, "latency, ms", "share of samples at or below", &step_ticks(&frame));
    chart.line(&frame, &left, colors.left, 1.8, false);
    chart.line(&frame, &right, colors.right, 1.8, false);
    let mut caption = "The empirical cumulative distribution of the samples in the window.".to_string();
    if let Some(x) = threshold.filter(|x| x.is_finite()) {
        chart.line(&frame, &[[x, frame.y.0], [x, frame.y.1]], colors.mark, 1.2, true);
        let share = |values: &[Option<f64>]| views::exceedance(values, x).map_or("none".to_string(), |share| format!("{:.1}%", share * 100.0));
        caption = format!("P(latency > {x:.3} ms): {} {}, {} {}.", review.left_label, share(&review.left), review.right_label, share(&review.right));
    }
    chart.legend(&[(&review.left_label, colors.left), (&review.right_label, colors.right)]);
    chart.finish(&caption)
}

/// B − A by percentile with intervals.
pub fn difference(review: &InferenceReview, colors: Colors) -> String {
    let points: Vec<[f64; 2]> = review
        .difference
        .iter()
        .flat_map(|point| [[views::nines(point.probability), point.low], [views::nines(point.probability), point.high]])
        .chain(std::iter::once([0.25, 0.0]))
        .collect();
    let Some(frame) = Frame::new(points.into_iter()) else {
        return empty(colors, "No percentile has enough samples on both sides.");
    };
    let mut chart = Chart::new(colors, &format!("{} − {}, by percentile", review.right_label, review.left_label), &review.headline);
    chart.axes(&frame, "percentile", "difference, ms", &percentile_ticks(&frame));
    chart.line(&frame, &[[frame.x.0, 0.0], [frame.x.1, 0.0]], colors.note, 1.0, true);
    for point in &review.difference {
        let x = views::nines(point.probability);
        chart.line(&frame, &[[x, point.low], [x, point.high]], colors.right, 3.0, false);
    }
    let estimates: Vec<[f64; 2]> = review.difference.iter().map(|point| [views::nines(point.probability), point.estimate]).collect();
    chart.dots(&frame, &estimates, colors.mark, 4.5);
    chart.finish("Bars are 95% block-bootstrap intervals of the steady samples; below zero, B is faster.")
}

/// Latency by percentile on a log tail axis.
pub fn spectrum(review: &InferenceReview, colors: Colors) -> String {
    let (left, right) = (views::spectrum(&review.left), views::spectrum(&review.right));
    let Some(frame) = Frame::new(left.iter().chain(&right).copied()) else {
        return empty(colors, "No samples to draw.");
    };
    let mut chart = Chart::new(colors, &format!("Latency spectrum: {} vs {}", review.left_label, review.right_label), &review.headline);
    chart.axes(&frame, "percentile", "latency, ms", &percentile_ticks(&frame));
    chart.line(&frame, &left, colors.left, 1.8, false);
    chart.line(&frame, &right, colors.right, 1.8, false);
    chart.legend(&[(&review.left_label, colors.left), (&review.right_label, colors.right)]);
    chart.finish("Each line stops at the highest percentile its sample count bounds.")
}

fn empty(colors: Colors, message: &str) -> String {
    Chart::new(colors, "ScalarScope", message).finish("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_round_steps() {
        assert_eq!(ticks(0.0, 10.0), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(ticks(7.2, 13.4), vec![8.0, 10.0, 12.0]);
        assert_eq!(ticks(5.0, 5.0), vec![5.0]);
    }

    #[test]
    fn long_text_wraps_at_spaces() {
        assert_eq!(wrap("aa bb cc", 5), vec!["aa bb", "cc"]);
        assert!(wrap("", 5).is_empty());
    }

    #[test]
    fn text_is_escaped() {
        assert_eq!(escape("a<b & \"c\">"), "a&lt;b &amp; &quot;c&quot;&gt;");
    }
}
