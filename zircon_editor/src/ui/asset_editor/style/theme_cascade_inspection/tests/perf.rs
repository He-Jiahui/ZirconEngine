use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const CASCADES_PER_SAMPLE: usize = 4_096;
const TOKENS_PER_LAYER: usize = 256;
const RULES_PER_LAYER: usize = 128;
const LAYERS: usize = 8;

#[test]
fn optimization_batch_jm_editor652_cascade_outputs_reserve_input_bounds() {
    let source = include_str!("../../theme_cascade_inspection.rs");
    let layer_fn = source
        .split("fn cascade_layer_items(")
        .nth(1)
        .expect("cascade layer item projection");
    let token_fn = source
        .split("fn cascade_token_items<'a>(")
        .nth(1)
        .expect("cascade token item projection");
    let rule_fn = source
        .split("fn cascade_rule_items<'a>(")
        .nth(1)
        .expect("cascade rule item projection");

    assert!(source.contains("saturating_add(usize::from(local_layer))"));
    assert!(layer_fn.contains("Vec::with_capacity(layers.len())"));
    assert!(token_fn.contains("Vec::with_capacity(total_token_definitions)"));
    assert!(rule_fn.contains("let total_rule_items = layers"));
    assert!(rule_fn.contains("saturating_mul(2)"));
    assert!(rule_fn.contains("Vec::with_capacity(total_rule_items)"));
    assert!(!layer_fn.contains("let mut items = Vec::new();"));
    assert!(!token_fn.contains("let mut items = Vec::new();"));
    assert!(!rule_fn.contains("let mut items = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jm_editor652_cascade_output_capacity_bench() {
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure(false));
            reserved_samples.push(measure(true));
        } else {
            reserved_samples.push(measure(true));
            unreserved_samples.push(measure(false));
        }
    }

    let unreserved_p50_ns = percentile(&unreserved_samples, 50);
    let reserved_p50_ns = percentile(&reserved_samples, 50);
    let unreserved_p95_ns = percentile(&unreserved_samples, 95);
    let reserved_p95_ns = percentile(&reserved_samples, 95);
    println!(
        "EDITOR652_CASCADE_OUTPUT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
cascades_per_sample={CASCADES_PER_SAMPLE} layers={LAYERS} tokens_per_layer={TOKENS_PER_LAYER} \
rules_per_layer={RULES_PER_LAYER} unreserved_p50_ns={unreserved_p50_ns} \
reserved_p50_ns={reserved_p50_ns} unreserved_p95_ns={unreserved_p95_ns} \
reserved_p95_ns={reserved_p95_ns} unreserved_raw_ns={} reserved_raw_ns={}",
        sample_csv(&unreserved_samples),
        sample_csv(&reserved_samples),
    );

    assert!(reserved_p95_ns <= unreserved_p95_ns * 80 / 100);
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for cascade in 0..CASCADES_PER_SAMPLE {
        let output_bound = LAYERS * (TOKENS_PER_LAYER + RULES_PER_LAYER);
        let mut items = if reserve {
            Vec::with_capacity(output_bound)
        } else {
            Vec::new()
        };
        for layer in 0..LAYERS {
            for token in 0..TOKENS_PER_LAYER {
                items.push(black_box(cascade ^ layer ^ token));
            }
            for rule in 0..RULES_PER_LAYER {
                items.push(black_box(cascade ^ layer ^ rule));
            }
        }
        checksum ^= items.len() ^ items[items.len() - 1];
        black_box(&items);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
