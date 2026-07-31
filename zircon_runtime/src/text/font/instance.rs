use std::collections::{BTreeMap, HashMap};
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
    last_used: u64,
    approximate_bytes: usize,
}

#[derive(Debug, Default)]
struct EffectiveInstanceCacheState {
    entries: HashMap<EffectiveInstanceCacheKey, EffectiveInstanceCacheEntry>,
    tick: u64,
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
        state.tick = state.tick.wrapping_add(1);
        let tick = state.tick;
        let value = state.entries.get_mut(&key).map(|entry| {
            entry.last_used = tick;
            entry.value.clone()
        });
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
        state.tick = state.tick.wrapping_add(1);
        if let Some(existing) = state.entries.remove(&key) {
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
            let Some(oldest) = state
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            if let Some(evicted) = state.entries.remove(&oldest) {
                state.approximate_bytes = state
                    .approximate_bytes
                    .saturating_sub(evicted.approximate_bytes);
                state.eviction_count = state.eviction_count.saturating_add(1);
            }
        }
        state.approximate_bytes = state.approximate_bytes.saturating_add(approximate_bytes);
        let last_used = state.tick;
        state.entries.insert(
            key,
            EffectiveInstanceCacheEntry {
                value,
                last_used,
                approximate_bytes,
            },
        );
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
    let mut coordinates = BTreeMap::new();
    for &(tag, value) in &variations.0 {
        if !value.is_finite() {
            return Err(FontInstanceError::NonFiniteCoordinate { tag });
        }
        coordinates.insert(tag, if value == 0.0 { 0.0 } else { value });
    }
    Ok(VariationCoords(coordinates.into_iter().collect()))
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
