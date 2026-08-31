use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime200_pointer_hover_path_reuses_retained_buffer() {
    let pointer_id = UiPointerId::new(7);
    let mut table = UiActivePointerTable::default();
    table.upsert(pointer_id, UiPointerSource::Mouse, true);
    let route = (1..=256).map(UiNodeId::new).collect::<Vec<_>>();

    table.set_hovered_path_iter(pointer_id, route.iter().copied());
    let entry = table.entry(pointer_id).expect("active pointer");
    let initial_ptr = entry.hovered.as_ptr();
    let initial_capacity = entry.hovered.capacity();

    table.set_hovered_path_iter(pointer_id, route.iter().copied());
    let unchanged = table.entry(pointer_id).expect("unchanged pointer");
    assert_eq!(unchanged.hovered, route);
    assert_eq!(unchanged.hovered.as_ptr(), initial_ptr);
    assert_eq!(unchanged.hovered.capacity(), initial_capacity);

    let mut changed_route = route.clone();
    changed_route[255] = UiNodeId::new(999);
    table.set_hovered_path_iter(pointer_id, changed_route.iter().copied());
    let changed = table.entry(pointer_id).expect("changed pointer");
    assert_eq!(changed.hovered, changed_route);
    assert_eq!(changed.hovered.as_ptr(), initial_ptr);
    assert_eq!(changed.hovered.capacity(), initial_capacity);
}

#[test]
fn runtime200_pointer_hover_path_borrows_an_iterator_without_collecting() {
    let pointer_source = include_str!("../pointer_table.rs");
    let slice_adapter = bounded_function(
        pointer_source,
        "pub fn set_hovered_path(",
        "pub fn set_hovered_path_iter(",
    );
    let iterator_setter = bounded_function(
        pointer_source,
        "pub fn set_hovered_path_iter(",
        "pub fn press_button",
    );

    assert!(
        slice_adapter.contains("impl AsRef<[UiNodeId]>") || slice_adapter.contains("&[UiNodeId]")
    );
    assert!(slice_adapter.contains("hovered.iter().copied()"));
    assert!(!slice_adapter.contains("to_vec()"));
    assert!(iterator_setter.contains("Iterator<Item = UiNodeId> + Clone"));
    assert!(iterator_setter.contains("entry.hovered.iter().copied().eq(hovered.clone())"));
    assert!(iterator_setter.contains("entry.hovered.clear()"));
    assert!(iterator_setter.contains("entry.hovered.extend(hovered)"));
    assert!(!iterator_setter.contains("entry.hovered = hovered"));
    assert!(!iterator_setter.contains(".collect()"));
    assert!(!iterator_setter.contains("to_vec()"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime200_pointer_hover_path_reuse_p95() {
    const ROUTE_DEPTH: usize = 512;
    const EVENTS: usize = 16_384;
    let pointer_id = UiPointerId::new(11);
    let route = (1..=ROUTE_DEPTH as u64)
        .map(UiNodeId::new)
        .collect::<Vec<_>>();
    let mut legacy_table = initialized_table(pointer_id, &route);
    let mut optimized_table = initialized_table(pointer_id, &route);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(EVENTS, || {
                legacy_set_hovered_path(&mut legacy_table, pointer_id, &route)
            }));
            optimized_ns.push(measure_ns(EVENTS, || {
                optimized_table.set_hovered_path_iter(pointer_id, route.iter().copied());
                ROUTE_DEPTH
            }));
        } else {
            optimized_ns.push(measure_ns(EVENTS, || {
                optimized_table.set_hovered_path_iter(pointer_id, route.iter().copied());
                ROUTE_DEPTH
            }));
            legacy_ns.push(measure_ns(EVENTS, || {
                legacy_set_hovered_path(&mut legacy_table, pointer_id, &route)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "retained hover-path reuse P95 must be at least 25% below repeated route clones: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME200_POINTER_HOVER_RETAINED_PATH_BENCH_V1 route_depth={ROUTE_DEPTH} events_per_sample={EVENTS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_route_clones_per_sample={EVENTS} optimized_route_clones_per_sample=0 legacy_node_copies_per_sample={} optimized_node_copies_per_sample=0 optimized_node_comparisons_per_sample={} legacy_vec_allocations_lower_bound_per_sample={EVENTS} optimized_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        ROUTE_DEPTH * EVENTS,
        ROUTE_DEPTH * EVENTS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn initialized_table(pointer_id: UiPointerId, route: &[UiNodeId]) -> UiActivePointerTable {
    let mut table = UiActivePointerTable::default();
    table.upsert(pointer_id, UiPointerSource::Mouse, true);
    legacy_set_hovered_path(&mut table, pointer_id, route);
    table
}

fn legacy_set_hovered_path(
    table: &mut UiActivePointerTable,
    pointer_id: UiPointerId,
    hovered: &[UiNodeId],
) -> usize {
    if let Some(entry) = table.entry_mut(pointer_id) {
        entry.hovered = hovered.to_vec();
        return entry.hovered.len();
    }
    0
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_function<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("function start")
        .split(end)
        .next()
        .expect("function end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
