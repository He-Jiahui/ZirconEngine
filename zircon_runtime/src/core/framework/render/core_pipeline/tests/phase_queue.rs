use std::hint::black_box;
use std::time::Instant;

use super::sort_phase_queue_items;
use crate::core::framework::render::{
    RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseSortKey,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn phase_iteration_limits_work_to_the_sorted_phase_order_span() {
    let source = include_str!("../phase_queue.rs");

    assert!(source.contains(concat!("partition", "_point")));
    assert!(source.contains(concat!("let phase_order", " = phase.queue_order();")));
    assert!(!source.contains(concat!(
        "self.items.iter().",
        "filter(move |item| item.phase == phase)"
    )));
}

#[test]
fn optimization_batch_dc_phase_queue_indirect_sort_preserves_stable_ties() {
    let mut actual = vec![
        item(3, 7, 30),
        item(1, 2, 10),
        item(3, 7, 31),
        item(2, 4, 20),
        item(3, 7, 32),
    ];
    let mut expected = actual.clone();
    expected.sort_by_key(RenderPhaseItem::ordering_key);

    sort_phase_queue_items(&mut actual);

    assert_eq!(actual, expected);
    assert_eq!(
        actual
            .iter()
            .filter(|item| item.entity == 3)
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::MeshIndex(30),
            RenderPhaseMeshSource::MeshIndex(31),
            RenderPhaseMeshSource::MeshIndex(32),
        ]
    );
}

#[test]
fn optimization_batch_dc_phase_queue_uses_compact_indirect_sort() {
    let production = include_str!("../phase_queue.rs")
        .split_once("#[cfg(test)]")
        .expect("production source and tests must remain separated")
        .0;

    assert!(production.contains("Vec::with_capacity(items.len())"));
    assert!(production.contains("ordered.sort_unstable();"));
    assert!(production.contains("destination_for_source"));
    assert!(!production.contains("items.sort_by_key(RenderPhaseItem::ordering_key)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_dc_runtime409_phase_queue_indirect_sort_p95() {
    const ITEM_COUNT: usize = 65_536;
    let template = (0..ITEM_COUNT)
        .map(|index| {
            let mixed = (index as u64)
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .rotate_left(17);
            item(mixed & 0x3fff, mixed >> 9, index)
        })
        .collect::<Vec<_>>();

    for _ in 0..3 {
        black_box(measure_sort(&template, false));
        black_box(measure_sort(&template, true));
    }

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure_sort(&template, false));
            optimized.push(measure_sort(&template, true));
        } else {
            optimized.push(measure_sort(&template, true));
            legacy.push(measure_sort(&template, false));
        }
    }

    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME409_PHASE_QUEUE_INDIRECT_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} item_count={ITEM_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn item(entity: u64, raw_sort_key: u64, mesh_index: usize) -> RenderPhaseItem {
    RenderPhaseItem {
        entity,
        phase: RenderPhase::Opaque3d,
        sort_key: RenderPhaseSortKey::new(raw_sort_key),
        mesh_source: RenderPhaseMeshSource::MeshIndex(mesh_index),
    }
}

fn measure_sort(template: &[RenderPhaseItem], optimized: bool) -> u128 {
    let mut items = template.to_vec();
    let started = Instant::now();
    if optimized {
        sort_phase_queue_items(&mut items);
    } else {
        items.sort_by_key(RenderPhaseItem::ordering_key);
    }
    black_box(items);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
