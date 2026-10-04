use std::{hint::black_box, time::Instant};

use crate::ui::event_ui::UiNodeId;

use super::{UiRenderCacheInvalidationReason, UiRenderCachePaintEntry, UiRenderCacheStatus};

const SAMPLE_COUNT: usize = 11;

fn scanned_entries(generations: &[Option<u64>]) -> (Vec<UiRenderCachePaintEntry>, usize) {
    let entries = generations
        .iter()
        .enumerate()
        .map(|(paint_index, &generation)| UiRenderCachePaintEntry {
            node_id: UiNodeId::new(paint_index as u64),
            paint_index,
            cache_generation: generation,
            status: UiRenderCacheStatus::from_generation(
                generation,
                UiRenderCacheInvalidationReason::Unchanged,
            ),
            reason: UiRenderCacheInvalidationReason::Unchanged,
        })
        .collect::<Vec<_>>();
    let reused = entries
        .iter()
        .filter(|entry| entry.status == UiRenderCacheStatus::Reused)
        .count();
    (entries, reused)
}

fn fused_entries(generations: &[Option<u64>]) -> (Vec<UiRenderCachePaintEntry>, usize) {
    let mut reused = 0;
    let entries = generations
        .iter()
        .enumerate()
        .map(|(paint_index, &generation)| {
            let status = UiRenderCacheStatus::from_generation(
                generation,
                UiRenderCacheInvalidationReason::Unchanged,
            );
            reused += usize::from(status == UiRenderCacheStatus::Reused);
            UiRenderCachePaintEntry {
                node_id: UiNodeId::new(paint_index as u64),
                paint_index,
                cache_generation: generation,
                status,
                reason: UiRenderCacheInvalidationReason::Unchanged,
            }
        })
        .collect::<Vec<_>>();
    (entries, reused)
}

#[test]
fn runtime_interface03_batch8_fused_cache_stats_preserve_entries_and_count() {
    let generations = [Some(1), None, Some(3), Some(4), None];

    assert_eq!(fused_entries(&generations), scanned_entries(&generations));
}

#[test]
#[ignore = "release-only fused render-cache stats benchmark"]
fn runtime_interface03_batch8_fused_render_cache_stats_release_benchmark() {
    const ENTRY_COUNT: usize = 131_072;
    let generations = (0..ENTRY_COUNT)
        .map(|index| (index % 4 != 0).then_some(index as u64))
        .collect::<Vec<_>>();
    let mut scanned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_scanned = || {
            let started = Instant::now();
            black_box(scanned_entries(black_box(&generations)));
            started.elapsed().as_nanos()
        };
        let measure_fused = || {
            let started = Instant::now();
            black_box(fused_entries(black_box(&generations)));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            scanned_samples.push(measure_scanned());
            fused_samples.push(measure_fused());
        } else {
            fused_samples.push(measure_fused());
            scanned_samples.push(measure_scanned());
        }
    }

    scanned_samples.sort_unstable();
    fused_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_FUSED_RENDER_CACHE_STATS_BENCH_V1 entries={ENTRY_COUNT} samples={SAMPLE_COUNT} scanned_p50_ns={} fused_p50_ns={} scanned_p95_ns={} fused_p95_ns={}",
        scanned_samples[p50],
        fused_samples[p50],
        scanned_samples[p95],
        fused_samples[p95],
    );
    assert!(
        fused_samples[p95].saturating_mul(10) <= scanned_samples[p95].saturating_mul(9),
        "fused cache stats must improve P95 by at least 10%: scanned={}ns fused={}ns",
        scanned_samples[p95],
        fused_samples[p95],
    );
}
