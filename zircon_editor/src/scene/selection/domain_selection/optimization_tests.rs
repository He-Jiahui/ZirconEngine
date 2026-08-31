use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::scene::EntityId;

use super::DomainSelection;

const PERF_MARKER: &str = "EDITOR306_SELECTION_EXTEND_RESERVE_BENCH_V1";

fn ids(count: usize) -> Vec<EntityId> {
    (0..count)
        .map(|index| EntityId::from(index as u64))
        .collect()
}

fn legacy_extend(
    selection: &mut DomainSelection,
    items: impl IntoIterator<Item = EntityId>,
) -> bool {
    let mut changed = false;
    for entity in items {
        if selection.items.insert(entity) {
            selection.primary = Some(entity);
            changed = true;
        }
    }
    if changed {
        selection.bump_generation();
    }
    changed
}

#[test]
fn optimization_batch_20260830bi_editor_selection_extend_preserves_order_and_primary() {
    let mut selection = DomainSelection::default();
    assert!(selection.extend(ids(64)));
    assert_eq!(selection.items().len(), 64);
    assert_eq!(selection.primary(), Some(63_u64));
    assert!(!selection.extend(ids(64)));
    assert_eq!(selection.generation(), 1);
}

#[test]
fn optimization_batch_20260830bi_editor_selection_extend_source_contract() {
    let source = include_str!("../domain_selection.rs");
    assert!(source.contains("let (lower_bound, _) = items.size_hint()"));
    assert!(source.contains("self.items.reserve(lower_bound)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bi_editor_selection_extend_p95() {
    const ITEMS: usize = 512;
    const REPETITIONS: usize = 2_000;
    const SAMPLES: usize = 17;
    let values = black_box(ids(ITEMS));
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..REPETITIONS {
                let mut selection = DomainSelection::default();
                if pass == 0 {
                    legacy_extend(&mut selection, values.iter().copied());
                } else {
                    selection.extend(values.iter().copied());
                }
                checksum += selection.items.len();
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} items={ITEMS} repetitions={REPETITIONS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
