use std::collections::{btree_map::Entry, BTreeMap};
use std::fmt;

use zircon_runtime::core::framework::ai::{AiBlackboardSchemaDescriptor, AiBlackboardValueType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A stable key location inside one schema-compiled blackboard layout.
pub struct BlackboardSlot {
    value_type: AiBlackboardValueType,
    offset: u32,
    generation_index: u32,
}

impl BlackboardSlot {
    /// Returns the value type owned by this slot.
    pub const fn value_type(self) -> AiBlackboardValueType {
        self.value_type
    }

    /// Returns the offset within the slot's type partition.
    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// Returns the index used by generation and observer arrays.
    pub const fn generation_index(self) -> u32 {
        self.generation_index
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Immutable key-to-slot mapping compiled from a validated schema descriptor.
pub struct BlackboardLayout {
    schema_id: String,
    slots: BTreeMap<String, BlackboardSlot>,
    keys: Box<[String]>,
    counts: [u32; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Error returned when a schema cannot be compiled into a dense layout.
pub enum BlackboardLayoutError {
    /// The schema declares the same key more than once.
    DuplicateKey { key: String },
    /// The schema declares a value type unsupported by the runtime store.
    UnknownValueType { key: String, value_type: String },
}

impl fmt::Display for BlackboardLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey { key } => write!(formatter, "blackboard key `{key}` is duplicated"),
            Self::UnknownValueType { key, value_type } => write!(
                formatter,
                "blackboard key `{key}` uses unknown value type `{value_type}`"
            ),
        }
    }
}

impl std::error::Error for BlackboardLayoutError {}

impl BlackboardLayout {
    /// Compiles a schema into stable, per-type dense partitions.
    pub fn from_schema(
        descriptor: &AiBlackboardSchemaDescriptor,
    ) -> Result<Self, BlackboardLayoutError> {
        let mut slots = BTreeMap::new();
        let mut keys = Vec::with_capacity(descriptor.keys.len());
        let mut counts = [0_u32; 6];
        for key in &descriptor.keys {
            let vacant = match slots.entry(key.key.clone()) {
                Entry::Occupied(_) => {
                    return Err(BlackboardLayoutError::DuplicateKey {
                        key: key.key.clone(),
                    });
                }
                Entry::Vacant(vacant) => vacant,
            };
            let value_type = key.expected_value_type().ok_or_else(|| {
                BlackboardLayoutError::UnknownValueType {
                    key: key.key.clone(),
                    value_type: key.value_type.clone(),
                }
            })?;
            let type_index = value_type_index(value_type);
            let slot = BlackboardSlot {
                value_type,
                offset: counts[type_index],
                generation_index: keys.len() as u32,
            };
            counts[type_index] += 1;
            vacant.insert(slot);
            keys.push(key.key.clone());
        }
        Ok(Self {
            schema_id: descriptor.id.clone(),
            slots,
            keys: keys.into_boxed_slice(),
            counts,
        })
    }

    /// Returns the source schema id.
    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }

    /// Resolves a schema key once into its dense runtime slot.
    pub fn resolve(&self, key: &str) -> Option<BlackboardSlot> {
        self.slots.get(key).copied()
    }

    /// Returns the schema key associated with a slot.
    pub fn key_for_slot(&self, slot: BlackboardSlot) -> Option<&str> {
        self.keys
            .get(slot.generation_index as usize)
            .map(String::as_str)
    }

    /// Returns the number of compiled keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub(crate) fn slots(&self) -> impl Iterator<Item = (&str, BlackboardSlot)> {
        self.slots.iter().map(|(key, slot)| (key.as_str(), *slot))
    }

    pub(crate) const fn count(&self, value_type: AiBlackboardValueType) -> usize {
        self.counts[value_type_index(value_type)] as usize
    }
}

const fn value_type_index(value_type: AiBlackboardValueType) -> usize {
    match value_type {
        AiBlackboardValueType::Bool => 0,
        AiBlackboardValueType::Integer => 1,
        AiBlackboardValueType::Scalar => 2,
        AiBlackboardValueType::String => 3,
        AiBlackboardValueType::Vec3 => 4,
        AiBlackboardValueType::Entity => 5,
    }
}

#[cfg(test)]
mod entry_performance_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiBlackboardSchemaDescriptor, AiBlackboardValueType,
    };

    use super::{value_type_index, BlackboardLayout, BlackboardLayoutError, BlackboardSlot};

    const BENCHMARK_KEY_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn entry_layout_preserves_duplicate_precedence_and_dense_offsets() {
        let duplicate_with_invalid_second_type =
            AiBlackboardSchemaDescriptor::new("duplicate", "Duplicate")
                .with_key("target", "entity", false)
                .with_key("target", "quaternion", false);
        assert_eq!(
            BlackboardLayout::from_schema(&duplicate_with_invalid_second_type),
            Err(BlackboardLayoutError::DuplicateKey {
                key: "target".to_string(),
            })
        );

        let descriptor = AiBlackboardSchemaDescriptor::new("mixed", "Mixed")
            .with_key("z_bool", "bool", false)
            .with_key("a_entity", "entity", false)
            .with_key("m_bool", "bool", false);
        let layout = BlackboardLayout::from_schema(&descriptor).expect("valid layout");
        let first = layout.resolve("z_bool").expect("first bool");
        let entity = layout.resolve("a_entity").expect("entity");
        let second = layout.resolve("m_bool").expect("second bool");

        assert_eq!(first.generation_index(), 0);
        assert_eq!(entity.generation_index(), 1);
        assert_eq!(second.generation_index(), 2);
        assert_eq!(first.offset(), 0);
        assert_eq!(entity.offset(), 0);
        assert_eq!(second.offset(), 1);
    }

    #[test]
    fn layout_compilation_uses_one_btree_entry_search_per_key() {
        let source = include_str!("layout.rs");
        let from_schema = source
            .split("pub fn from_schema(")
            .nth(1)
            .and_then(|body| body.split("pub fn schema_id").next())
            .expect("from_schema body");

        assert!(from_schema.contains("slots.entry("));
        assert!(!from_schema.contains("slots.contains_key("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn single_search_blackboard_layout_release_benchmark_evidence() {
        let descriptor = benchmark_descriptor(BENCHMARK_KEY_COUNT);
        assert_eq!(
            legacy_layout(black_box(&descriptor))
                .expect("legacy layout")
                .key_count(),
            BlackboardLayout::from_schema(black_box(&descriptor))
                .expect("optimized layout")
                .key_count()
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || black_box(legacy_layout(black_box(&descriptor)).expect("legacy layout")).key_count(),
            || {
                black_box(
                    BlackboardLayout::from_schema(black_box(&descriptor))
                        .expect("optimized layout"),
                )
                .key_count()
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_single_search_blackboard_layout keys={BENCHMARK_KEY_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_btree_searches_per_sample={} optimized_btree_searches_per_sample={BENCHMARK_KEY_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_KEY_COUNT * 2
        );
        assert!(
            optimized_p95 * 4 <= legacy_p95 * 3,
            "optimized P95 {optimized_p95}ns must be no more than 75% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_descriptor(key_count: usize) -> AiBlackboardSchemaDescriptor {
        let mut descriptor = AiBlackboardSchemaDescriptor::new("benchmark", "Benchmark");
        for index in 0..key_count {
            descriptor = descriptor.with_key(format!("key_{index:04}"), "integer", false);
        }
        descriptor
    }

    fn legacy_layout(
        descriptor: &AiBlackboardSchemaDescriptor,
    ) -> Result<BlackboardLayout, BlackboardLayoutError> {
        let mut slots = BTreeMap::new();
        let mut keys = Vec::with_capacity(descriptor.keys.len());
        let mut counts = [0_u32; 6];
        for key in &descriptor.keys {
            if slots.contains_key(key.key.as_str()) {
                return Err(BlackboardLayoutError::DuplicateKey {
                    key: key.key.clone(),
                });
            }
            let value_type = key.expected_value_type().ok_or_else(|| {
                BlackboardLayoutError::UnknownValueType {
                    key: key.key.clone(),
                    value_type: key.value_type.clone(),
                }
            })?;
            let type_index = value_type_index(value_type);
            let slot = BlackboardSlot {
                value_type,
                offset: counts[type_index],
                generation_index: keys.len() as u32,
            };
            counts[type_index] += 1;
            slots.insert(key.key.clone(), slot);
            keys.push(key.key.clone());
        }
        Ok(BlackboardLayout {
            schema_id: descriptor.id.clone(),
            slots,
            keys: keys.into_boxed_slice(),
            counts,
        })
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        black_box(operation());
        started.elapsed().as_nanos()
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
