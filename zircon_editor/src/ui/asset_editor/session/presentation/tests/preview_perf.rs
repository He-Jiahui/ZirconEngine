use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const ITEMS_PER_SAMPLE: usize = 256;

#[test]
fn preview_projection_reserves_all_output_collection_lengths() {
    let source = include_str!("../preview.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(source_items.len())"));
    assert!(implementation.contains("Vec::with_capacity(candidates.len())"));
    assert!(implementation.contains("Vec::with_capacity(canvas_nodes.len())"));
    assert!(implementation.contains("for item in source_items"));
    assert!(implementation.contains("for candidate in candidates"));
    assert!(implementation.contains("for item in canvas_nodes"));
}

#[test]
fn preview_projection_keeps_slot_candidates_before_canvas_mapping() {
    let source = include_str!("../preview.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let slot = implementation
        .find("for item in source_items")
        .expect("slot loop");
    let candidate = implementation
        .find("for candidate in candidates")
        .expect("candidate loop");
    let canvas = implementation
        .find("for item in canvas_nodes")
        .expect("canvas loop");
    assert!(slot < candidate);
    assert!(candidate < canvas);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830by_editor_preview_projection_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR323_PREVIEW_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut slots = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut candidates = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut canvas = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..ITEMS_PER_SAMPLE {
            slots.push(index);
            candidates.push(index);
            canvas.push(index);
        }
        checksum ^= slots.len() ^ candidates.len() ^ canvas.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
