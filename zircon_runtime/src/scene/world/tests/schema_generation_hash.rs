//! 模式代数散列索引的行为、源码布局与忽略性能夹具。
//!
//! 描述符迭代仍按有序目录输出；散列表用于模式代数的逐键查询。

use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::ComponentTypeRegistry;
use crate::core::framework::scene::ComponentTypeDescriptor;

// 512 个键、每样本 4096 次查询、17 对交错样本；这些预算只约束本微基准。
const ENTRY_COUNT: usize = 512;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

/// 分别核对登记代数递增、未知键为零，以及描述符仍按类型键排序。
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

/// 只核对目录源码的三项容器声明；排序行为另由上一个测试实际调用目录验证。
#[test]
fn runtime63_batch_schema_generation_hash_index_keeps_descriptor_ordered() {
    let source = include_str!("../component_type_registry.rs");

    assert!(source.contains("use std::collections::{BTreeMap, HashMap};"));
    assert!(source.contains("descriptors: BTreeMap<String, ComponentTypeDescriptor>"));
    assert!(source.contains("schema_generations: HashMap<String, u64>"));
}

/// 对同一组末键查询比较两种容器的交错计时；这是忽略微基准，不代表完整字段写入耗时。
/// 输出中的查询次数与分配字段来自固定预算和标签，不是独立的分配观测。
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

/// 供基准统一调用两种容器的复制值查询，不参与 Runtime 目录发布。
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

/// 仅计时一次查询批次，并检查校验和非零；返回纳秒值。
fn measure_ns(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
    started.elapsed().as_nanos()
}

/// 复制并排序非空样本，返回上整秩的样本值；调用处固定使用 50 或 95 百分位。
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
