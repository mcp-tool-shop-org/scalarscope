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
fn delta_tc_uses_the_milestone_and_is_withheld_without_one() {
    let later = pair(&marked("a", vec![10.0; 8], Some(2)), &marked("b", vec![10.0; 8], Some(5))).unwrap();
    let Pair::Inference(review) = later else { panic!("inference") };
    assert_eq!(review.fired, vec!["ΔTc".to_string()]);
    assert!(review.verdict.contains("Stabilizes 3 steps later"));

    let earlier = pair(&marked("a", vec![10.0; 8], Some(5)), &marked("b", vec![10.0; 8], Some(2))).unwrap();
    let Pair::Inference(review) = earlier else { panic!("inference") };
    assert!(review.verdict.contains("Stabilizes 3 steps earlier"));

    let left = "step,latency_ms\n0,10\n1,10\n2,10\n3,10\n";
    let right = "step,latency_ms\n0,10\n1,10\n2,10\n3,10\n4,10\n5,10\n6,10\n7,10\n";
    let Pair::Inference(review) = pair(&open_text(left, "short").unwrap(), &open_text(right, "longer").unwrap()).unwrap() else {
        panic!("inference");
    };
    assert!(!review.fired.iter().any(|symbol| symbol == "ΔTc"));
    assert!(review.verdict.contains("not a stabilization time"));
}

#[test]
fn delta_f_fires_only_when_the_right_side_has_more_outliers() {
    let calm = vec![10.0; 20];
    let mut spiked = vec![10.0; 19];
    spiked.push(100.0);
    let Pair::Inference(review) = pair(&marked("a", calm.clone(), None), &marked("b", spiked.clone(), None)).unwrap() else {
        panic!("inference");
    };
    assert_eq!(review.fired, vec!["ΔF".to_string(), "ΔO".to_string()]);
    assert!(review.verdict.contains("Introduced 1 new runtime anomalies"));

    let Pair::Inference(review) = pair(&marked("a", spiked, None), &marked("b", calm, None)).unwrap() else {
        panic!("inference");
    };
    assert_eq!(review.fired, vec!["ΔO".to_string()]);
    assert!(review.verdict.contains("Reduced runtime variability"));
}

#[test]
fn delta_o_ignores_a_change_inside_one_percent_of_the_larger_spread() {
    let Pair::Inference(review) = pair(&marked("a", vec![1.0, -1.0], None), &marked("b", vec![1.005, -1.005], None)).unwrap() else {
        panic!("inference");
    };
    assert!(!review.fired.iter().any(|symbol| symbol == "ΔO"));

    let Pair::Inference(review) = pair(&marked("a", vec![1.0, -1.0], None), &marked("b", vec![1.02, -1.02], None)).unwrap() else {
        panic!("inference");
    };
    assert_eq!(review.fired, vec!["ΔO".to_string()]);
    assert!(review.verdict.contains("Increased runtime variability"));
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
    })
}

fn finite(values: &[Option<f64>]) -> Vec<f64> {
    values.iter().copied().flatten().collect()
}
