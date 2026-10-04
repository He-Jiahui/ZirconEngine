use std::hint::black_box;
use std::time::Instant;

const IMAGE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn runtime868_gltf_image_output_capacity_preserves_empty_capacity() {
    let empty = optimized_projection(&[]);
    assert!(empty.is_empty());
    assert_eq!(empty.capacity(), 0);

    let source = include_str!("../../images.rs");
    let decode_images = source
        .split("pub(super) fn decode_images")
        .nth(1)
        .expect("glTF image decoder owner must exist")
        .split("pub(super) fn decode_external_image")
        .next()
        .expect("glTF image output owner must stay bounded");
    assert!(decode_images.contains("let image_count = document.images().len();"));
    assert!(decode_images.contains("let mut images = Vec::with_capacity(image_count);"));
    assert!(decode_images.contains("images.push(decoded);"));
    assert!(!decode_images.contains(".collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime868_gltf_image_output_capacity_benchmark() {
    let values = (0..IMAGE_COUNT).collect::<Vec<_>>();
    assert_eq!(legacy_projection(&values), values);
    assert_eq!(optimized_projection(&values), values);

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&values)));
            optimized.push(measure(|| optimized_projection(&values)));
        } else {
            optimized.push(measure(|| optimized_projection(&values)));
            legacy.push(measure(|| legacy_projection(&values)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "RUNTIME868_GLTF_IMAGE_OUTPUT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} images={IMAGE_COUNT} legacy_growth_events=11 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(growth_events(IMAGE_COUNT, 0), 11);
    assert_eq!(growth_events(IMAGE_COUNT, IMAGE_COUNT), 0);
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "preallocated output P95 {optimized_p95}ns must stay within 10% of Result-collect P95 {legacy_p95}ns"
    );
}

fn legacy_projection(values: &[usize]) -> Vec<usize> {
    values
        .iter()
        .copied()
        .map(Ok::<usize, ()>)
        .collect::<Result<Vec<_>, _>>()
        .expect("the benchmark fixture is infallible")
}

fn optimized_projection(values: &[usize]) -> Vec<usize> {
    let mut projected = Vec::with_capacity(values.len());
    for value in values {
        projected.push(*value);
    }
    projected
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
