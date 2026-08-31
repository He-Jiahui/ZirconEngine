use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiValue};

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260826ce_showcase_state_hash_cache_preserves_latest_values() {
    let mut showcase = UiComponentShowcaseDemoState::default();
    showcase.states.insert(
        "SliderDemo".to_owned(),
        UiComponentState::new().with_value("value", UiValue::Float(12.0)),
    );
    showcase.states.insert(
        "CheckboxDemo".to_owned(),
        UiComponentState::new().with_value("value", UiValue::Bool(true)),
    );
    showcase.states.insert(
        "SliderDemo".to_owned(),
        UiComponentState::new().with_value("value", UiValue::Float(42.0)),
    );

    assert_eq!(showcase.states.len(), 2);
    assert_eq!(
        showcase
            .states
            .get("SliderDemo")
            .and_then(|state| state.value("value")),
        Some(&UiValue::Float(42.0))
    );
    assert_eq!(
        showcase
            .states
            .get("CheckboxDemo")
            .and_then(|state| state.value("value")),
        Some(&UiValue::Bool(true))
    );
}

#[test]
fn optimization_batch_20260826ce_showcase_state_hash_cache_has_no_order_contract() {
    let source = include_str!("../showcase_demo_state.rs");

    assert!(source.contains("states: HashMap<String, UiComponentState>"));
    assert!(source.contains("use std::collections::{BTreeMap, HashMap, VecDeque};"));
    assert!(!source.contains("self.states.iter()"));
    assert!(!source.contains("self.states.into_iter()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ce_showcase_state_hash_cache_p95() {
    const CONTROLS: usize = 16_384;
    let controls = (0..CONTROLS)
        .map(|index| format!("UiComponentShowcase.SharedControlPrefix.{index:05}"))
        .collect::<Vec<_>>();
    let legacy = controls
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, control)| (control, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = controls
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, control)| (control, index))
        .collect::<HashMap<_, _>>();
    let target = controls
        .last()
        .expect("benchmark control set must not be empty");

    let mut legacy_lookup = || repeated_lookup(&legacy, target);
    let mut optimized_lookup = || repeated_lookup(&optimized, target);
    assert_eq!(black_box(legacy_lookup()), black_box(optimized_lookup()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_lookup));
            optimized_ns.push(measure_ns(&mut optimized_lookup));
        } else {
            optimized_ns.push(measure_ns(&mut optimized_lookup));
            legacy_ns.push(measure_ns(&mut legacy_lookup));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "showcase state hash lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR52_SHOWCASE_STATE_HASH_CACHE_BENCH_V1 controls={CONTROLS} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn repeated_lookup<M>(map: &M, key: &str) -> usize
where
    M: LookupMap,
{
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(map.lookup(black_box(key))));
    }
    black_box(checksum)
}

trait LookupMap {
    fn lookup(&self, key: &str) -> usize;
}

impl LookupMap for BTreeMap<String, usize> {
    fn lookup(&self, key: &str) -> usize {
        *self.get(key).expect("legacy benchmark control must exist")
    }
}

impl LookupMap for HashMap<String, usize> {
    fn lookup(&self, key: &str) -> usize {
        *self
            .get(key)
            .expect("optimized benchmark control must exist")
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
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
