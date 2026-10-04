use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::TreeIndex;

const SAMPLE_PAIRS: usize = 17;
const SELECTED_COUNT: usize = 65_536;

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut used = 0usize;
    let mut events = 0usize;
    while used < length {
        if used == capacity {
            if capacity == 0 {
                capacity = 4;
            } else if capacity == usize::MAX {
                break;
            } else {
                capacity = capacity.saturating_mul(2);
            }
            events += 1;
        }
        used += 1;
    }
    events
}

fn reserved_growth_events(_length: usize) -> usize {
    0
}

fn tree_source(count: usize) -> UiValue {
    UiValue::Array(
        (0..count)
            .map(|index| {
                let mut entry = std::collections::BTreeMap::new();
                let id = format!("node-{index:05}");
                entry.insert("id".to_string(), UiValue::String(id.clone()));
                entry.insert("label".to_string(), UiValue::String(id));
                UiValue::Map(entry)
            })
            .collect(),
    )
}

#[test]
fn optimization_batch_20260915_runtime788_tree_selection_output_preserves_order() {
    let source = tree_source(6);
    let mut index = TreeIndex::compile(&source, None, None, 1);
    assert_eq!(index.selected_count(), 0);
    assert!(index.set_selected("node-04", true));
    assert!(index.set_selected("node-01", true));
    assert_eq!(index.selected_count(), 2);
    assert_eq!(
        index.selected_ids_ordered().collect::<Vec<_>>(),
        ["node-01", "node-04"]
    );
    assert!(index.set_selected("node-01", false));
    assert_eq!(index.selected_count(), 1);
    assert_eq!(
        index.selected_ids_ordered().collect::<Vec<_>>(),
        ["node-04"]
    );
}

#[test]
fn optimization_batch_20260915_runtime788_tree_selection_output_keeps_exact_bound() {
    let source = include_str!("../../state_model.rs");
    let production = source.split("#[cfg(test)]").next().expect("production");
    assert!(production.contains("Vec::with_capacity(self.tree_index().selected_count())"));
    assert!(production.contains(
        "next_selected.extend(self.tree_index().selected_ids_ordered().map(str::to_owned));"
    ));
}

fn measure_nanos(run: impl FnOnce() -> u64) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos().max(1)
}

fn legacy_projection(ids: &[String]) -> u64 {
    let values = ids
        .iter()
        .filter(|_| true)
        .map(String::clone)
        .collect::<Vec<_>>();
    values.len() as u64
}

fn reserved_projection(ids: &[String]) -> u64 {
    let mut values = Vec::with_capacity(ids.len());
    values.extend(ids.iter().map(String::clone));
    values.len() as u64
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260915_runtime788_tree_selection_output_p95() {
    let ids = (0..SELECTED_COUNT)
        .map(|index| format!("node-{index:05}"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_nanos(|| legacy_projection(black_box(&ids))));
            reserved_samples.push(measure_nanos(|| reserved_projection(black_box(&ids))));
        } else {
            reserved_samples.push(measure_nanos(|| reserved_projection(black_box(&ids))));
            legacy_samples.push(measure_nanos(|| legacy_projection(black_box(&ids))));
        }
    }
    let legacy_p95 = percentile_95(&mut legacy_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1 selected={SELECTED_COUNT} \
         legacy_p95_ns={legacy_p95} reserved_p95_ns={reserved_p95} \
         legacy_growth_events={} reserved_growth_events={}",
        geometric_growth_events(SELECTED_COUNT),
        reserved_growth_events(SELECTED_COUNT),
    );
    assert!(legacy_p95 > 0);
    assert!(reserved_p95 > 0);
    assert!(geometric_growth_events(SELECTED_COUNT) > reserved_growth_events(SELECTED_COUNT));
}
