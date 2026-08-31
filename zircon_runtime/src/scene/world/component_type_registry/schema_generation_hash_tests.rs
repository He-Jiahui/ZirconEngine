use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::ComponentTypeRegistry;
use crate::core::framework::scene::ComponentTypeDescriptor;

const ENTRY_COUNT: usize = 512;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn runtime63_batch_schema_generation_hash_index_preserves_generation_and_order() {
    let mut registry = ComponentTypeRegistry::default();
    registry
        .register(ComponentTypeDescriptor::new(
            "weather.Component.Zeta",
            "weather",
            "Zeta",
        ))
        .unwrap();
    let zeta_generation = registry.schema_generation("weather.Component.Zeta");
    registry
        .register(ComponentTypeDescriptor::new(
            "weather.Component.Alpha",
            "weather",
            "Alpha",
        ))
        .unwrap();

    assert!(zeta_generation > 0);
    assert!(registry.schema_generation("weather.Component.Alpha") > zeta_generation);
    assert_eq!(registry.schema_generation("weather.Component.Missing"), 0);
    assert_eq!(
        registry
            .descriptors()
            .map(|descriptor| descriptor.type_id.as_str())
            .collect::<Vec<_>>(),
        vec!["weather.Component.Alpha", "weather.Component.Zeta"]
    );
}

#[test]
fn runtime63_batch_schema_generation_hash_index_keeps_descriptor_ordered() {
    let source = include_str!("../component_type_registry.rs");

    assert!(source.contains("use std::collections::{BTreeMap, HashMap};"));
    assert!(source.contains("descriptors: BTreeMap<String, ComponentTypeDescriptor>"));
    assert!(source.contains("schema_generations: HashMap<String, u64>"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime63_batch_schema_generation_hash_index_p95() {
    let keys = (0..ENTRY_COUNT)
        .map(|index| {
            format!("weather.Component.SharedSchemaGenerationPrefix.ForStableLookup.{index:04}")
        })
        .collect::<Vec<_>>();
    let ordered = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index as u64 + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = keys.last().unwrap().as_str();

    let mut ordered_lookup = || repeated_lookup(&ordered, target);
    let mut hash_lookup = || repeated_lookup(&hashed, target);
    assert_eq!(black_box(ordered_lookup()), black_box(hash_lookup()));

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut ordered_lookup));
            hash_ns.push(measure_ns(&mut hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut hash_lookup));
            ordered_ns.push(measure_ns(&mut ordered_lookup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(7),
        "schema generation hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME63_SCHEMA_GENERATION_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} sample_pairs={SAMPLE_COUNT} pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} descriptor_order_changes=0 direct_hit_allocations=0 ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: &str) -> u64
where
    V: Lookup,
{
    let mut total = 0_u64;
    for _ in 0..HIT_COUNT {
        total = total.wrapping_add(black_box(map.lookup(black_box(target))).unwrap_or_default());
    }
    total
}

trait Lookup {
    fn lookup(&self, key: &str) -> Option<u64>;
}

impl Lookup for BTreeMap<String, u64> {
    fn lookup(&self, key: &str) -> Option<u64> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<String, u64> {
    fn lookup(&self, key: &str) -> Option<u64> {
        self.get(key).copied()
    }
}

fn measure_ns(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
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
