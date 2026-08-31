use std::collections::HashMap;
use std::fmt;
use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::text::{FontFaceId, InstancedFaceId, VariationCoords};

const FONT_INSTANCE_HASH_DOMAIN: &[u8] = b"zircon-font-instance-v1";
const OPEN_TYPE_NORMALIZED_COORDINATE_SCALE: f32 = 16_384.0;
const EFFECTIVE_INSTANCE_CACHE_CAPACITY: usize = 256;
const EFFECTIVE_INSTANCE_CACHE_MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FontInstance {
    pub(crate) face: FontFaceId,
    pub(crate) variations: VariationCoords,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FontInstanceRegistry {
    instances: HashMap<InstancedFaceId, FontInstance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct EffectiveInstanceCacheKey {
    pub(super) face: FontFaceId,
    pub(super) instance: InstancedFaceId,
    pub(super) font_weight: u16,
}

#[derive(Clone, Debug)]
pub(super) struct EffectiveInstanceCacheValue {
    pub(super) id: InstancedFaceId,
    pub(super) variations: Arc<VariationCoords>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EffectiveInstanceCacheReport {
    pub(crate) hits: u64,
    pub(crate) misses: u64,
    pub(crate) eviction_count: u64,
    pub(crate) entry_count: usize,
    pub(crate) approximate_bytes: usize,
}

#[derive(Clone, Debug)]
struct EffectiveInstanceCacheEntry {
    value: EffectiveInstanceCacheValue,
    approximate_bytes: usize,
    lru_links: EffectiveInstanceCacheLruLinks,
}

#[derive(Clone, Copy, Debug, Default)]
struct EffectiveInstanceCacheLruLinks {
    previous: Option<EffectiveInstanceCacheKey>,
    next: Option<EffectiveInstanceCacheKey>,
}

#[derive(Debug, Default)]
struct EffectiveInstanceCacheState {
    entries: HashMap<EffectiveInstanceCacheKey, EffectiveInstanceCacheEntry>,
    lru_head: Option<EffectiveInstanceCacheKey>,
    lru_tail: Option<EffectiveInstanceCacheKey>,
    eviction_count: u64,
    approximate_bytes: usize,
}

#[derive(Debug, Default)]
struct EffectiveInstanceCacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
}

#[derive(Clone)]
pub(super) struct EffectiveInstanceCache {
    state: Arc<Mutex<EffectiveInstanceCacheState>>,
    stats: Arc<EffectiveInstanceCacheStats>,
}

impl fmt::Debug for EffectiveInstanceCache {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EffectiveInstanceCache")
            .field("report", &self.report())
            .finish()
    }
}

impl Default for EffectiveInstanceCache {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(EffectiveInstanceCacheState::default())),
            stats: Arc::new(EffectiveInstanceCacheStats::default()),
        }
    }
}

impl EffectiveInstanceCache {
    pub(super) fn get(
        &self,
        key: EffectiveInstanceCacheKey,
    ) -> Option<EffectiveInstanceCacheValue> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = state.get(key);
        drop(state);
        if value.is_some() {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
        }
        value
    }

    pub(super) fn insert(
        &self,
        key: EffectiveInstanceCacheKey,
        value: EffectiveInstanceCacheValue,
    ) {
        let approximate_bytes = effective_instance_entry_bytes(&value);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(existing) = state.remove(key) {
            state.approximate_bytes = state
                .approximate_bytes
                .saturating_sub(existing.approximate_bytes);
        }
        if approximate_bytes > EFFECTIVE_INSTANCE_CACHE_MAX_BYTES {
            return;
        }
        while !state.entries.is_empty()
            && (state.entries.len() >= EFFECTIVE_INSTANCE_CACHE_CAPACITY
                || state.approximate_bytes.saturating_add(approximate_bytes)
                    > EFFECTIVE_INSTANCE_CACHE_MAX_BYTES)
        {
            let Some(oldest) = state.lru_head else {
                break;
            };
            let Some(evicted) = state.remove(oldest) else {
                break;
            };
            state.approximate_bytes = state
                .approximate_bytes
                .saturating_sub(evicted.approximate_bytes);
            state.eviction_count = state.eviction_count.saturating_add(1);
        }
        state.approximate_bytes = state.approximate_bytes.saturating_add(approximate_bytes);
        state.entries.insert(
            key,
            EffectiveInstanceCacheEntry {
                value,
                approximate_bytes,
                lru_links: EffectiveInstanceCacheLruLinks::default(),
            },
        );
        state.attach_most_recent(key);
    }

    pub(super) fn report(&self) -> EffectiveInstanceCacheReport {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        EffectiveInstanceCacheReport {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            eviction_count: state.eviction_count,
            entry_count: state.entries.len(),
            approximate_bytes: state.approximate_bytes,
        }
    }
}

impl EffectiveInstanceCacheState {
    fn get(&mut self, key: EffectiveInstanceCacheKey) -> Option<EffectiveInstanceCacheValue> {
        self.entries.get(&key)?;
        self.touch(key);
        self.entries.get(&key).map(|entry| entry.value.clone())
    }

    fn remove(&mut self, key: EffectiveInstanceCacheKey) -> Option<EffectiveInstanceCacheEntry> {
        let links = self.entries.get(&key)?.lru_links;
        self.detach(key, links);
        self.entries.remove(&key)
    }

    fn touch(&mut self, key: EffectiveInstanceCacheKey) {
        let Some(links) = self.entries.get(&key).map(|entry| entry.lru_links) else {
            return;
        };
        self.detach(key, links);
        self.attach_most_recent(key);
    }

    fn attach_most_recent(&mut self, key: EffectiveInstanceCacheKey) {
        let previous = self
            .lru_tail
            .filter(|candidate| self.entries.contains_key(candidate));
        if let Some(previous) = previous {
            if let Some(entry) = self.entries.get_mut(&previous) {
                entry.lru_links.next = Some(key);
            }
        } else {
            self.lru_head = Some(key);
        }
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.lru_links = EffectiveInstanceCacheLruLinks {
                previous,
                next: None,
            };
        }
        self.lru_tail = Some(key);
    }

    fn detach(&mut self, key: EffectiveInstanceCacheKey, links: EffectiveInstanceCacheLruLinks) {
        let previous = links
            .previous
            .filter(|candidate| self.entries.contains_key(candidate));
        let next = links
            .next
            .filter(|candidate| self.entries.contains_key(candidate));
        if let Some(previous) = previous {
            if let Some(entry) = self.entries.get_mut(&previous) {
                entry.lru_links.next = next;
            }
        } else {
            self.lru_head = next;
        }
        if let Some(next) = next {
            if let Some(entry) = self.entries.get_mut(&next) {
                entry.lru_links.previous = previous;
            }
        } else {
            self.lru_tail = previous;
        }
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.lru_links = EffectiveInstanceCacheLruLinks::default();
        }
    }
}

fn effective_instance_entry_bytes(value: &EffectiveInstanceCacheValue) -> usize {
    size_of::<EffectiveInstanceCacheKey>()
        .saturating_add(size_of::<EffectiveInstanceCacheValue>())
        .saturating_add(
            value
                .variations
                .0
                .len()
                .saturating_mul(size_of::<(u32, f32)>()),
        )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum FontInstanceError {
    #[error("font variation coordinate {tag:#010x} is not finite")]
    NonFiniteCoordinate { tag: u32 },
    #[error("font instance identity collision for {id:?}")]
    IdentityCollision { id: InstancedFaceId },
}

impl FontInstanceRegistry {
    pub(crate) fn resolve_or_insert(
        &mut self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontInstanceError> {
        let variations = canonical_variation_coords(variations)?;
        let id = font_instance_identity(face, &variations)?;
        let instance = FontInstance { face, variations };
        match self.instances.get(&id) {
            Some(existing) if existing == &instance => Ok(id),
            Some(_) => Err(FontInstanceError::IdentityCollision { id }),
            None => {
                self.instances.insert(id, instance);
                Ok(id)
            }
        }
    }

    pub(crate) fn get(&self, id: InstancedFaceId) -> Option<&FontInstance> {
        self.instances.get(&id)
    }

    pub(super) fn remove_face(&mut self, face: FontFaceId) {
        self.instances.retain(|_, instance| instance.face != face);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.instances.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

pub(super) fn font_instance_identity(
    face: FontFaceId,
    variations: &VariationCoords,
) -> Result<InstancedFaceId, FontInstanceError> {
    let variations = canonical_variation_coords(variations)?;
    Ok(font_instance_id(face, &variations))
}

pub(super) fn canonical_variation_coords(
    variations: &VariationCoords,
) -> Result<VariationCoords, FontInstanceError> {
    let mut coordinates = Vec::with_capacity(variations.0.len());
    for &(tag, value) in &variations.0 {
        if !value.is_finite() {
            return Err(FontInstanceError::NonFiniteCoordinate { tag });
        }
        coordinates.push((tag, if value == 0.0 { 0.0 } else { value }));
    }
    coordinates.sort_by_key(|(tag, _)| *tag);
    let mut unique_count = 0;
    for read_index in 0..coordinates.len() {
        if unique_count > 0 && coordinates[unique_count - 1].0 == coordinates[read_index].0 {
            coordinates[unique_count - 1].1 = coordinates[read_index].1;
        } else {
            coordinates[unique_count] = coordinates[read_index];
            unique_count += 1;
        }
    }
    coordinates.truncate(unique_count);
    Ok(VariationCoords(coordinates))
}

pub(super) fn quantized_axis_value(
    value: f32,
    min_value: f32,
    default_value: f32,
    max_value: f32,
) -> f32 {
    let value = value.clamp(min_value, max_value);
    if value == default_value {
        return default_value;
    }
    let span = if value < default_value {
        default_value - min_value
    } else {
        max_value - default_value
    };
    if span <= f32::EPSILON {
        return default_value;
    }
    let normalized = (value - default_value) / span;
    let normalized = (normalized.clamp(-1.0, 1.0) * OPEN_TYPE_NORMALIZED_COORDINATE_SCALE).trunc()
        / OPEN_TYPE_NORMALIZED_COORDINATE_SCALE;
    if normalized < 0.0 {
        default_value + normalized * (default_value - min_value)
    } else {
        default_value + normalized * (max_value - default_value)
    }
}

fn font_instance_id(face: FontFaceId, variations: &VariationCoords) -> InstancedFaceId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FONT_INSTANCE_HASH_DOMAIN);
    hasher.update(&face.0.to_le_bytes());
    for (tag, value) in &variations.0 {
        hasher.update(&tag.to_be_bytes());
        hasher.update(&value.to_bits().to_le_bytes());
    }
    let mut id_bytes = [0_u8; size_of::<u64>()];
    id_bytes.copy_from_slice(&hasher.finalize().as_bytes()[..size_of::<u64>()]);
    InstancedFaceId(u64::from_le_bytes(id_bytes))
}

#[cfg(test)]
#[path = "instance/tests.rs"]
mod tests;

#[cfg(test)]
mod optimization_batch_dd_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{FontInstanceError, canonical_variation_coords};
    use crate::text::VariationCoords;

    const SAMPLE_PAIRS: usize = 17;
    const AXES_PER_SAMPLE: usize = 16_384;

    #[test]
    fn font_variation_canonicalization_preserves_last_duplicate_and_sorted_order() {
        let input = VariationCoords(vec![(3, 1.0), (1, 2.0), (3, 4.0), (2, -0.0)]);
        assert_eq!(
            canonical_variation_coords(&input).unwrap(),
            VariationCoords(vec![(1, 2.0), (2, 0.0), (3, 4.0)])
        );
    }

    #[test]
    fn optimization_batch_dd_font_variation_canonicalization_matches_tree_baseline() {
        let input = VariationCoords(
            (0..4_096)
                .map(|index| {
                    let tag = (index as u32).wrapping_mul(17) % 257;
                    (tag, if index % 31 == 0 { -0.0 } else { index as f32 })
                })
                .collect(),
        );
        assert_eq!(
            canonical_variation_coords(&input),
            legacy_canonical_variation_coords(&input.0)
        );
    }

    #[test]
    fn optimization_batch_dd_font_variation_canonicalization_uses_sorted_vector() {
        let production = include_str!("instance.rs")
            .split_once("#[cfg(test)]")
            .expect("production source and tests must remain separated")
            .0;
        assert!(production.contains("Vec::with_capacity(variations.0.len())"));
        assert!(production.contains("coordinates.sort_by_key(|(tag, _)| *tag)"));
        assert!(production.contains("coordinates.truncate(unique_count)"));
        assert!(!production.contains("let mut coordinates = BTreeMap::new()"));
    }

    #[test]
    fn optimization_batch_dd_font_variation_canonicalization_rejects_non_finite_values() {
        let error = canonical_variation_coords(&VariationCoords(vec![(7, f32::NAN)])).unwrap_err();
        assert_eq!(error, FontInstanceError::NonFiniteCoordinate { tag: 7 });
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_dd_runtime411_font_variation_vector_sort_p95() {
        let input = VariationCoords(
            (0..AXES_PER_SAMPLE)
                .map(|index| {
                    let tag = (index as u32).wrapping_mul(0x9e37_79b9);
                    (tag, (index % 997) as f32 + 0.5)
                })
                .collect::<Vec<_>>(),
        );

        for _ in 0..3 {
            black_box(legacy_canonical_variation_coords(black_box(&input.0)));
            black_box(canonical_variation_coords(black_box(&input)));
        }

        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(|| legacy_canonical_variation_coords(&input.0)));
                optimized.push(measure(|| canonical_variation_coords(&input)));
            } else {
                optimized.push(measure(|| canonical_variation_coords(&input)));
                legacy.push(measure(|| legacy_canonical_variation_coords(&input.0)));
            }
        }

        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME411_FONT_VARIATION_VECTOR_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} axes_per_sample={AXES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn legacy_canonical_variation_coords(
        values: &[(u32, f32)],
    ) -> Result<VariationCoords, FontInstanceError> {
        let mut coordinates = BTreeMap::new();
        for &(tag, value) in values {
            if !value.is_finite() {
                return Err(FontInstanceError::NonFiniteCoordinate { tag });
            }
            coordinates.insert(tag, if value == 0.0 { 0.0 } else { value });
        }
        Ok(VariationCoords(coordinates.into_iter().collect()))
    }

    fn measure(run: impl FnOnce() -> Result<VariationCoords, FontInstanceError>) -> u128 {
        let started = Instant::now();
        black_box(run());
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
}
