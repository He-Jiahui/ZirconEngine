use super::*;

fn legacy_prefixes(value: &str) -> (String, String) {
    let normalized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_ascii_lowercase();
    let node_prefix = if normalized.is_empty() {
        "node".to_string()
    } else {
        normalized
    };

    let control = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    let control_prefix = if control.is_empty() {
        "Node".to_string()
    } else {
        control
    };
    (node_prefix, control_prefix)
}

#[test]
fn single_pass_prefixes_preserve_legacy_normalization() {
    for value in [
        "Button",
        "Fancy-Button",
        "__Leading__And__Trailing__",
        "9Patch[Image]",
        "汉字",
        "---",
        "Button 42",
        "A.B:C#D",
    ] {
        let expected = legacy_prefixes(value);
        let template = UiDefaultNodeTemplate::native(value);
        assert_eq!(
            (template.node_id_prefix, template.control_id_prefix.unwrap()),
            expected
        );
    }
}

#[test]
#[ignore = "release-only single-pass default node prefix benchmark"]
fn runtime_interface03_batch53_single_pass_default_node_prefix_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ITERATIONS: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let widget_type = "__Fancy-Button.Panel42__";
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_prefixes(black_box(widget_type)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(native_prefixes(black_box(widget_type)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_PASS_DEFAULT_NODE_PREFIX_BENCH_V1 iterations={ITERATIONS} samples={SAMPLE_COUNT} legacy_p95_ns={} optimized_p95_ns={}",
        legacy_samples[p95], optimized_samples[p95],
    );
    assert!(
        optimized_samples[p95].saturating_mul(5) <= legacy_samples[p95].saturating_mul(4),
        "single-pass default node prefixes must improve P95 by at least 20%: legacy={}ns optimized={}ns",
        legacy_samples[p95],
        optimized_samples[p95],
    );
}
