use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const FRAMES_PER_SAMPLE: usize = 64;
const SEGMENTS_PER_FRAME: usize = 4_096;

#[test]
fn optimization_batch_20260826fu_runtime216_capacity_covers_unique_execution_pages() {
    let mut seen_pages = HashSet::with_capacity(SEGMENTS_PER_FRAME);
    let mut all_pages = HashSet::with_capacity(SEGMENTS_PER_FRAME);
    for page in 0..SEGMENTS_PER_FRAME as u32 {
        assert!(seen_pages.insert(page));
        assert!(all_pages.insert(page));
    }

    assert_eq!(seen_pages.len(), SEGMENTS_PER_FRAME);
    assert_eq!(all_pages.len(), SEGMENTS_PER_FRAME);
    assert!(seen_pages.capacity() >= SEGMENTS_PER_FRAME);
    assert!(all_pages.capacity() >= SEGMENTS_PER_FRAME);
}

#[test]
fn optimization_batch_20260826fu_runtime216_page_sets_reserve_draw_segment_count() {
    let source = include_str!("../virtual_geometry_stats.rs");
    assert!(source.contains("let draw_segments = &context"));
    assert_eq!(
        source
            .matches("HashSet::with_capacity(draw_segments.len())")
            .count(),
        2
    );
    assert!(source.contains("for segment in draw_segments"));
    assert!(!source.contains("let mut seen_pages = HashSet::new();"));
    assert!(!source.contains("let mut all_pages = HashSet::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fu_runtime216_virtual_geometry_page_set_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME216_VIRTUAL_GEOMETRY_PAGE_SET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
frames_per_sample={FRAMES_PER_SAMPLE} segments_per_frame={SEGMENTS_PER_FRAME} \
legacy_preallocated_sets_per_frame=0 optimized_preallocated_sets_per_frame=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for frame in 0..FRAMES_PER_SAMPLE {
        let mut seen_pages = if reserve {
            HashSet::with_capacity(SEGMENTS_PER_FRAME)
        } else {
            HashSet::new()
        };
        let mut all_pages = if reserve {
            HashSet::with_capacity(SEGMENTS_PER_FRAME)
        } else {
            HashSet::new()
        };
        for segment in 0..SEGMENTS_PER_FRAME {
            let page = black_box((frame ^ segment) as u32);
            seen_pages.insert(page);
            all_pages.insert(page);
        }
        checksum ^= black_box(
            seen_pages.len() ^ seen_pages.capacity() ^ all_pages.len() ^ all_pages.capacity(),
        );
        black_box((&seen_pages, &all_pages));
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
