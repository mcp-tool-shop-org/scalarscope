use scalarscope::milestones::{detect_steady_start, detect_warmup_end};
use scalarscope::open::{open_text, InferenceRun, Side};
use scalarscope::readings::{deviation_band, percentile, steady_index, three_sigma_indices};
use scalarscope::review::{pair, Pair};

#[test]
fn deviation_band_is_centered_population_spread() {
    let values = [Some(0.0), Some(0.0), Some(0.0), Some(0.0), Some(10.0), Some(0.0), Some(0.0), Some(0.0)];
    let band = deviation_band(&values);
    let point = band[4].expect("the spike has a band");
    assert!((point.low - -2.0).abs() < 1e-9);
    assert!((point.high - 6.0).abs() < 1e-9);
}

#[test]
fn three_sigma_marks_only_the_point_beyond_the_spread() {
    let mut values = vec![Some(10.0); 19];
    values.push(Some(100.0));
    assert_eq!(three_sigma_indices(&values), vec![19]);
}

#[test]
fn steady_index_follows_the_milestone() {
    let steps: Vec<i64> = (0..8).collect();
    assert_eq!(steady_index(&steps, 0, 8, None), None);
    assert_eq!(steady_index(&steps, 0, 8, Some(5)), Some(5));
    assert_eq!(steady_index(&steps, 2, 6, Some(5)), Some(3));
}

#[test]
fn nearest_rank_p99_is_the_last_of_eight() {
    let sorted = [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 7.0];
    assert_eq!(percentile(&sorted, 0.99), Some(7.0));
}

#[test]
fn latency_csv_round_trips_and_keeps_throughput_beside_it() {
    let left = "step,latency_ms,throughput\n0,10,100\n1,10,110\n2,12,90\n3,11,100\n4,10,100\n5,10,100\n6,13,80\n7,10,100\n";
    let right = "step,latency_ms,throughput\n0,5,200\n1,5,200\n2,6,180\n3,5,200\n4,5,200\n5,5,200\n6,7,160\n7,5,200\n";
    let Pair::Inference(review) = pair(&open_text(left, "baseline").unwrap(), &open_text(right, "optimized").unwrap()).unwrap() else {
        panic!("two latency files are an inference review");
    };
    assert_eq!(review.signal, "latency_ms");
    assert_eq!(finite(&review.left), vec![10.0, 10.0, 12.0, 11.0, 10.0, 10.0, 13.0, 10.0]);
    assert_eq!(review.left_throughput, vec![100.0, 110.0, 90.0, 100.0, 100.0, 100.0, 80.0, 100.0]);
    assert_eq!(review.left_p99, Some(13.0));
    assert_eq!(review.right_p99, Some(7.0));
    assert!(review.left_steady.is_none());
    assert!(review.caption.contains("not a confidence interval"));
    assert!(!review.caption.contains("vertical line"));
    assert_eq!(review.left_cdf.last().map(|point| point.1), Some(1.0));
}

#[test]
fn profiler_and_benchmark_keep_their_latency() {
    let trace = r#"{"traceEvents":[
        {"name":"TensorRT inference","ph":"X","dur":2500,"ts":0},
        {"name":"aten::linear","ph":"X","dur":9000,"ts":1},
        {"name":"inference","ph":"X","dur":4000,"ts":3000}
    ]}"#;
    let Side::Inference(run) = open_text(trace, "capture").unwrap() else {
        panic!("a chrome trace is inference");
    };
    assert_eq!(run.latency_ms, vec![2.5, 4.0]);

    let benchmark = r#"{"results":[{"latency_ms":1.25},{"iteration":1,"latency_ms":1.75}]}"#;
    let Side::Inference(run) = open_text(benchmark, "after").unwrap() else {
        panic!("a benchmark is inference");
    };
    assert_eq!(run.latency_ms, vec![1.25, 1.75]);
}

#[test]
fn a_profiler_step_is_one_inference_and_nested_ops_are_not() {
    let trace = r#"{"traceEvents":[
        {"name":"ProfilerStep#0","ph":"X","dur":5000,"ts":0},
        {"name":"aten::linear","ph":"X","dur":9000,"ts":1},
        {"name":"cudaLaunchKernel","ph":"X","dur":100,"ts":2},
        {"name":"TensorRT inference","ph":"X","dur":2500,"ts":3},
        {"name":"ProfilerStep#1","ph":"X","ts":10000},
        {"name":"ProfilerStep#2","ph":"B","dur":8000,"ts":20000},
        {"name":"ProfilerStep#3","ph":"X","dur":6000,"ts":30000}
    ]}"#;
    let Side::Inference(run) = open_text(trace, "pytorch").unwrap() else {
        panic!("a profiler step trace is inference");
    };
    assert_eq!(run.latency_ms, vec![5.0, 6.0]);
}

#[test]
fn an_incomplete_profiler_step_does_not_hide_a_named_inference_event() {
    let trace = r#"{"traceEvents":[
        {"name":"ProfilerStep#0","ph":"B","dur":5000},
        {"name":"inference","ph":"X","dur":4000}
    ]}"#;
    let Side::Inference(run) = open_text(trace, "capture").unwrap() else {
        panic!("a named inference event is still a sample");
    };
    assert_eq!(run.latency_ms, vec![4.0]);
}

#[test]
fn a_trace_whose_names_are_not_inference_is_refused() {
    let trace = r#"{"traceEvents":[{"name":"aten::linear","dur":1000}]}"#;
    let error = open_text(trace, "pytorch").unwrap_err();
    assert!(error.contains("TensorRT or inference"));
}

#[test]
fn a_word_in_the_latency_column_is_refused() {
    let error = open_text("step,latency_ms\n0,fast\n", "empty").unwrap_err();
    assert!(error.contains("no numeric latency"));
}

#[test]
fn training_loss_stays_loss_and_final_loss_is_not_appended() {
    let history = r#"[
        {"run_id":"early","status":"completed","model_name":"old","loss_history":[2.0,1.5],"final_loss":1.4},
        {"run_id":"running","status":"running","loss_history":[9.0]},
        {"run_id":"later","status":"completed","model_name":"qwen","steps":40,"loss_history":[1.0,0.5],"final_loss":0.4,
         "eval":{"held_out_loss":0.55,"perplexity":1.73,"eval_n":120,"task_metrics":{"exact_match":0.42},"metric_ci":{"exact_match":0.03}}}
    ]"#;
    let other = r#"{"run_id":"baseline","status":"completed","loss_history":[1.2,0.8],"final_loss":0.7}"#;
    let Pair::Training(review) = pair(&open_text(history, "after").unwrap(), &open_text(other, "before").unwrap()).unwrap() else {
        panic!("two histories are a training review");
    };
    assert_eq!(review.left.run_id, "later");
    assert_eq!(review.left.loss, vec![1.0, 0.5]);
    assert_eq!(review.left.final_loss, Some(0.4));
    assert_eq!(review.left.held_out_loss, Some(0.55));
    assert_eq!(review.left.perplexity, Some(1.73));
    assert_eq!(review.left.eval_n, Some(120));
    assert_eq!(review.left.task_metrics, vec![("exact_match".to_string(), 0.42)]);
    assert!(review.left_text.contains("exact_match 0.42 ± 0.03"));
    assert!(review.caption.contains("Training loss"));
    assert!(review.caption.contains("not an inference review"));
    assert!(!review.left.loss.contains(&0.4));
}

#[test]
fn an_empty_loss_history_does_not_invent_a_point_from_final_loss() {
    let entry = r#"{"run_id":"bare","status":"completed","loss_history":[],"final_loss":0.2}"#;
    let Side::Training(parsed) = open_text(entry, "bare").unwrap() else {
        panic!("a run entry is training");
    };
    assert!(parsed.loss.is_empty());
    assert_eq!(parsed.final_loss, Some(0.2));
}

#[test]
fn geometry_is_not_a_training_history_and_a_csv_is_not_one_either() {
    let geometry = r#"{"schema_version":"1.0","trajectory":{"timesteps":[{"t":0,"state_2d":[1,2]}]}}"#;
    assert!(open_text(geometry, "path").unwrap_err().contains("geometry"));
    let Side::Inference(_) = open_text("step,latency_ms\n0,1\n", "trace").unwrap() else {
        panic!("a latency csv is inference");
    };
}

#[test]
fn a_training_file_beside_a_trace_is_refused() {
    let history = r#"{"run_id":"one","status":"completed","loss_history":[1.0],"final_loss":1.0}"#;
    let error = pair(
        &open_text(history, "train").unwrap(),
        &open_text("step,latency_ms\n0,1\n", "trace").unwrap(),
    )
    .unwrap_err();
    assert!(error.contains("same kind"));
}

#[test]
fn the_steady_line_is_named_only_when_both_milestones_exist() {
    let flat = |count: usize| {
        let mut csv = String::from("step,latency_ms\n");
        for index in 0..count {
            csv.push_str(&format!("{index},10\n"));
        }
        csv
    };
    let Pair::Inference(review) = pair(&open_text(&flat(20), "a").unwrap(), &open_text(&flat(20), "b").unwrap()).unwrap() else {
        panic!("two flat series are an inference review");
    };
    assert!(review.left_steady.is_some());
    assert!(review.right_steady.is_some());
    assert!(review.caption.contains("The vertical line is the steady-state milestone."));

    let Pair::Inference(short) = pair(&open_text(&flat(4), "a").unwrap(), &open_text(&flat(4), "b").unwrap()).unwrap() else {
        panic!("two short series are an inference review");
    };
    assert!(short.left_steady.is_none());
    assert!(!short.caption.contains("vertical line"));
}

#[test]
fn a_flat_series_of_twenty_finds_steady_state_at_the_first_stable_window() {
    let values = vec![10.0; 20];
    assert_eq!(detect_warmup_end(&values), Some(3));
    assert_eq!(detect_steady_start(&values, 3), Some(3));
    assert_eq!(detect_warmup_end(&[10.0; 8]), None);
}

#[test]
fn delta_tc_set_by_hand_on_a_short_run_is_withheld() {
    // A steady step given without a file to state it is not used; eight samples are too short
    // to find one. The stated-step rule is covered by the golden fixtures in runtrace_tests.
    let review = pair(&marked("a", vec![10.0; 8], Some(2)), &marked("b", vec![10.0; 8], Some(5))).unwrap();
    let Pair::Inference(review) = review else { panic!("inference") };
    assert!(review.fired.is_empty());
    assert!(review.verdict.contains("ΔTc is withheld") && review.verdict.contains("too short to tell"), "{}", review.verdict);
}

fn spikes(count: usize) -> Vec<f64> {
    let mut values = vec![10.0; 200];
    for index in 0..count {
        values[20 + index * 22] = 100.0;
    }
    values
}

#[test]
fn delta_f_fires_only_for_an_excess_beyond_chance() {
    let Pair::Inference(review) = pair(&marked("a", vec![10.0; 200], None), &marked("b", spikes(8), None)).unwrap() else {
        panic!("inference");
    };
    assert_eq!(review.fired, vec!["ΔF".to_string()]);
    assert!(review.verdict.contains("Introduced 8 new runtime anomalies"), "{}", review.verdict);

    // Fewer anomalies on B is not a ΔF, and one more is not beyond chance.
    let Pair::Inference(review) = pair(&marked("a", spikes(8), None), &marked("b", vec![10.0; 200], None)).unwrap() else {
        panic!("inference");
    };
    assert!(review.fired.is_empty(), "{:?}", review.fired);
    let Pair::Inference(review) = pair(&marked("a", vec![10.0; 200], None), &marked("b", spikes(1), None)).unwrap() else {
        panic!("inference");
    };
    assert!(review.fired.is_empty(), "{:?}", review.fired);
}

fn spread(width: f64, seed: u64) -> Vec<f64> {
    let mut rng = scalarscope::stats::Rng::new(seed);
    (0..300).map(|_| 10.0 + ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * width).collect()
}

#[test]
fn delta_o_fires_only_when_the_spread_interval_excludes_one() {
    let Pair::Inference(review) = pair(&marked("a", spread(1.0, 1), None), &marked("b", spread(1.005, 2), None)).unwrap() else {
        panic!("inference");
    };
    assert!(!review.fired.iter().any(|symbol| symbol == "ΔO"), "{}", review.verdict);

    let Pair::Inference(review) = pair(&marked("a", spread(1.0, 3), None), &marked("b", spread(2.0, 4), None)).unwrap() else {
        panic!("inference");
    };
    assert_eq!(review.fired, vec!["ΔO".to_string()]);
    assert!(review.verdict.contains("Increased runtime variability: relative spread (p10–p90 / p50) × 2"), "{}", review.verdict);

    // Two samples cannot carry an interval.
    let Pair::Inference(review) = pair(&marked("a", vec![1.0, -1.0], None), &marked("b", vec![1.5, -1.5], None)).unwrap() else {
        panic!("inference");
    };
    assert!(review.fired.is_empty());
}

fn marked(label: &str, latency: Vec<f64>, steady: Option<i64>) -> Side {
    let steps = (0..latency.len() as i64).collect();
    Side::Inference(InferenceRun {
        label: label.to_string(),
        steps,
        latency_ms: latency,
        throughput: Vec::new(),
        warmup_end: None,
        steady_step: steady,
        memory_mb: Vec::new(),
        trace: None,
    })
}

fn finite(values: &[Option<f64>]) -> Vec<f64> {
    values.iter().copied().flatten().collect()
}

#[test]
fn delta_tc_stays_quiet_below_the_three_step_resolution() {
    for (left_step, right_step) in [(4, 5), (5, 3), (2, 2)] {
        let quiet = pair(&marked("a", vec![10.0; 8], Some(left_step)), &marked("b", vec![10.0; 8], Some(right_step))).unwrap();
        let Pair::Inference(review) = quiet else { panic!("inference") };
        assert!(!review.fired.iter().any(|symbol| symbol == "ΔTc"), "{left_step} vs {right_step}");
        assert!(!review.verdict.contains("Stabilizes"), "{left_step} vs {right_step}");
    }
}

fn decay(amplitude: f64, base: f64, tau: f64, seed: u64) -> Side {
    let mut rng = scalarscope::stats::Rng::new(seed);
    let rows: String = (0..400)
        .map(|step| {
            let noise = ((rng.next_u64() % 1000) as f64 / 1000.0 - 0.5) * 0.06;
            format!("{step},{}\n", base * (1.0 + noise) + amplitude * (-(step as f64) / tau).exp())
        })
        .collect();
    open_text(&format!("step,latency_ms\n{rows}"), "decay").unwrap()
}

#[test]
fn delta_tc_fires_when_the_settling_ranges_are_apart() {
    // A slow warmup (tau 40) against a fast one (tau 5): settled near step 200 against step 25.
    let Pair::Inference(review) = pair(&decay(30.0, 10.0, 40.0, 1), &decay(30.0, 10.0, 5.0, 2)).unwrap() else { panic!("inference") };
    assert!(review.fired.contains(&"ΔTc".to_string()), "{}", review.verdict);
    assert!(review.verdict.contains("earlier"), "{}", review.verdict);
    assert!(review.left_text.contains("warmup, settles"), "{}", review.left_text);
}

#[test]
fn delta_tc_stays_quiet_when_the_ranges_overlap() {
    // The same decay constant: both settle at about the same step.
    let Pair::Inference(review) = pair(&decay(30.0, 12.0, 12.0, 3), &decay(22.0, 8.0, 12.0, 4)).unwrap() else { panic!("inference") };
    assert!(!review.fired.contains(&"ΔTc".to_string()), "{}", review.verdict);
}

#[test]
fn the_headline_is_a_ratio_with_an_interval_and_names_short_percentiles() {
    let Pair::Inference(review) = pair(&decay(0.0, 12.0, 1.0, 5), &decay(0.0, 9.0, 1.0, 6)).unwrap() else { panic!("inference") };
    assert!(review.headline.starts_with("B/A p50 0.7"), "{}", review.headline);
    assert!(review.headline.contains("p99"), "{}", review.headline);
    let short = |n: usize| open_text(&format!("step,latency_ms\n{}", (0..n).map(|s| format!("{s},10\n")).collect::<String>()), "short").unwrap();
    let Pair::Inference(review) = pair(&short(60), &short(60)).unwrap() else { panic!("inference") };
    assert!(review.headline.contains("p99 needs 368 steady samples per side"), "{}", review.headline);
}
