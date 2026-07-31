use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::Hash;
use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::text::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript, SubFontRange};

use super::composite_resolve::CompositeFontIndex;
use super::fallback::FallbackResolution;
use super::matching::FontFamilyIdentity;

const FALLBACK_CACHE_HASH_DOMAIN: &[u8] = b"zircon-font-fallback-cache-v1";
const FAMILY_CACHE_CAPACITY: usize = 256;
const COMPOSITE_CACHE_CAPACITY: usize = 64;
const CANDIDATE_CACHE_CAPACITY: usize = 1_024;
const RESOLUTION_CACHE_CAPACITY: usize = 1_024;
const FALLBACK_CACHE_MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FamilyCandidateCacheKey([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct CompositeFontIdentity([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FallbackQueryIdentity([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FallbackCandidateCacheKey([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FallbackResolutionCacheKey([u8; 16]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FallbackCacheReport {
    pub(crate) normalization_allocation_count: u64,
    pub(crate) family_hits: u64,
    pub(crate) family_misses: u64,
    pub(crate) candidate_hits: u64,
    pub(crate) candidate_misses: u64,
    pub(crate) resolution_hits: u64,
    pub(crate) resolution_misses: u64,
    pub(crate) composite_hits: u64,
    pub(crate) composite_misses: u64,
    pub(crate) composite_compile_count: u64,
    pub(crate) family_sort_count: u64,
    pub(crate) family_visit_count: u64,
    pub(crate) face_visit_count: u64,
    pub(crate) coverage_probe_count: u64,
    pub(crate) eviction_count: u64,
    pub(crate) family_entry_count: usize,
    pub(crate) candidate_entry_count: usize,
    pub(crate) resolution_entry_count: usize,
    pub(crate) composite_entry_count: usize,
    pub(crate) approximate_bytes: usize,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    last_used: u64,
    approximate_bytes: usize,
}

struct BoundedCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    // One entry per live cache item: eviction reads the oldest tick without scanning `entries`.
    lru: BTreeMap<u64, K>,
    tick: u64,
    capacity: usize,
    max_bytes: usize,
    approximate_bytes: usize,
    eviction_count: u64,
}

impl<K, V> BoundedCache<K, V>
where
    K: Copy + Eq + Hash,
    V: Clone,
{
    fn new(capacity: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru: BTreeMap::new(),
            tick: 0,
            capacity,
            max_bytes,
            approximate_bytes: 0,
            eviction_count: 0,
        }
    }

    fn get(&mut self, key: K) -> Option<V> {
        let tick = self.next_tick();
        let (value, previous_tick) = {
            let entry = self.entries.get_mut(&key)?;
            let previous_tick = entry.last_used;
            entry.last_used = tick;
            (entry.value.clone(), previous_tick)
        };
        self.lru.remove(&previous_tick);
        self.lru.insert(tick, key);
        Some(value)
    }

    fn insert(&mut self, key: K, value: V, approximate_bytes: usize) {
        let tick = self.next_tick();
        if let Some(existing) = self.entries.remove(&key) {
            self.approximate_bytes = self
                .approximate_bytes
                .saturating_sub(existing.approximate_bytes);
            self.lru.remove(&existing.last_used);
        }
        if self.capacity == 0 || approximate_bytes > self.max_bytes {
            return;
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= self.capacity
                || self.approximate_bytes.saturating_add(approximate_bytes) > self.max_bytes)
        {
            let Some((_, oldest)) = self.lru.pop_first() else {
                break;
            };
            if let Some(evicted) = self.entries.remove(&oldest) {
                self.approximate_bytes = self
                    .approximate_bytes
                    .saturating_sub(evicted.approximate_bytes);
                self.eviction_count = self.eviction_count.saturating_add(1);
            }
        }
        self.approximate_bytes = self.approximate_bytes.saturating_add(approximate_bytes);
        self.entries.insert(
            key,
            CacheEntry {
                value,
                last_used: tick,
                approximate_bytes,
            },
        );
        self.lru.insert(tick, key);
    }

    fn next_tick(&mut self) -> u64 {
        if self.tick == u64::MAX {
            self.rebase_lru_ticks();
        }
        self.tick += 1;
        self.tick
    }

    fn rebase_lru_ticks(&mut self) {
        let lru = std::mem::take(&mut self.lru);
        self.tick = 0;
        for (_, key) in lru {
            self.tick += 1;
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.last_used = self.tick;
                self.lru.insert(self.tick, key);
            }
        }
    }
}

struct FallbackCacheState {
    families: BoundedCache<FamilyCandidateCacheKey, Arc<[FontFaceId]>>,
    composites: BoundedCache<CompositeFontIdentity, Arc<CompositeFontIndex>>,
    candidates: BoundedCache<FallbackCandidateCacheKey, Arc<[FontFaceId]>>,
    resolutions: BoundedCache<FallbackResolutionCacheKey, FallbackResolution>,
}

impl Default for FallbackCacheState {
    fn default() -> Self {
        Self {
            families: BoundedCache::new(FAMILY_CACHE_CAPACITY, FALLBACK_CACHE_MAX_BYTES / 8),
            composites: BoundedCache::new(COMPOSITE_CACHE_CAPACITY, FALLBACK_CACHE_MAX_BYTES / 8),
            candidates: BoundedCache::new(CANDIDATE_CACHE_CAPACITY, FALLBACK_CACHE_MAX_BYTES / 2),
            resolutions: BoundedCache::new(RESOLUTION_CACHE_CAPACITY, FALLBACK_CACHE_MAX_BYTES / 4),
        }
    }
}

#[derive(Default)]
struct FallbackCacheStats {
    normalization_allocation_count: AtomicU64,
    family_hits: AtomicU64,
    family_misses: AtomicU64,
    candidate_hits: AtomicU64,
    candidate_misses: AtomicU64,
    resolution_hits: AtomicU64,
    resolution_misses: AtomicU64,
    composite_hits: AtomicU64,
    composite_misses: AtomicU64,
    composite_compile_count: AtomicU64,
    family_sort_count: AtomicU64,
    family_visit_count: AtomicU64,
    face_visit_count: AtomicU64,
    coverage_probe_count: AtomicU64,
}

#[derive(Clone, Default)]
pub(super) struct FallbackCaches {
    state: Arc<Mutex<FallbackCacheState>>,
    stats: Arc<FallbackCacheStats>,
}

impl fmt::Debug for FallbackCaches {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FallbackCaches")
            .field("report", &self.report())
            .finish()
    }
}

impl FallbackCaches {
    pub(super) fn composite_index(
        &self,
        descriptor: &CompositeFontDescriptor,
    ) -> (CompositeFontIdentity, Arc<CompositeFontIndex>) {
        let identity = composite_font_identity(descriptor);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(index) = state.composites.get(identity) {
            self.stats.composite_hits.fetch_add(1, Ordering::Relaxed);
            return (identity, index);
        }
        self.stats.composite_misses.fetch_add(1, Ordering::Relaxed);
        let index = Arc::new(CompositeFontIndex::compile(descriptor));
        let bytes = size_of::<CompositeFontIdentity>().saturating_add(index.approximate_bytes());
        state.composites.insert(identity, Arc::clone(&index), bytes);
        self.stats
            .composite_compile_count
            .fetch_add(1, Ordering::Relaxed);
        (identity, index)
    }

    pub(super) fn family_candidates(
        &self,
        key: FamilyCandidateCacheKey,
    ) -> Option<Arc<[FontFaceId]>> {
        let value = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .families
            .get(key);
        self.record_hit(
            value.is_some(),
            &self.stats.family_hits,
            &self.stats.family_misses,
        );
        value
    }

    pub(super) fn insert_family_candidates(
        &self,
        key: FamilyCandidateCacheKey,
        candidates: Arc<[FontFaceId]>,
    ) {
        let bytes = size_of::<FamilyCandidateCacheKey>()
            .saturating_add(candidates.len().saturating_mul(size_of::<FontFaceId>()));
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .families
            .insert(key, candidates, bytes);
        self.stats.family_sort_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn candidates(&self, key: FallbackCandidateCacheKey) -> Option<Arc<[FontFaceId]>> {
        let value = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .candidates
            .get(key);
        self.record_hit(
            value.is_some(),
            &self.stats.candidate_hits,
            &self.stats.candidate_misses,
        );
        value
    }

    pub(super) fn insert_candidates(
        &self,
        key: FallbackCandidateCacheKey,
        candidates: Arc<[FontFaceId]>,
    ) {
        let bytes = size_of::<FallbackCandidateCacheKey>()
            .saturating_add(candidates.len().saturating_mul(size_of::<FontFaceId>()));
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .candidates
            .insert(key, candidates, bytes);
    }

    pub(super) fn resolution(&self, key: FallbackResolutionCacheKey) -> Option<FallbackResolution> {
        let value = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolutions
            .get(key);
        self.record_hit(
            value.is_some(),
            &self.stats.resolution_hits,
            &self.stats.resolution_misses,
        );
        value
    }

    pub(super) fn insert_resolution(
        &self,
        key: FallbackResolutionCacheKey,
        resolution: FallbackResolution,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolutions
            .insert(
                key,
                resolution,
                size_of::<FallbackResolutionCacheKey>() + size_of::<FallbackResolution>(),
            );
    }

    pub(super) fn record_family_visits(&self, count: usize) {
        self.stats
            .family_visit_count
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub(super) fn record_face_visits(&self, count: usize) {
        self.stats
            .face_visit_count
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub(super) fn record_coverage_probe(&self) {
        self.stats
            .coverage_probe_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn report(&self) -> FallbackCacheReport {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        FallbackCacheReport {
            normalization_allocation_count: self
                .stats
                .normalization_allocation_count
                .load(Ordering::Relaxed),
            family_hits: self.stats.family_hits.load(Ordering::Relaxed),
            family_misses: self.stats.family_misses.load(Ordering::Relaxed),
            candidate_hits: self.stats.candidate_hits.load(Ordering::Relaxed),
            candidate_misses: self.stats.candidate_misses.load(Ordering::Relaxed),
            resolution_hits: self.stats.resolution_hits.load(Ordering::Relaxed),
            resolution_misses: self.stats.resolution_misses.load(Ordering::Relaxed),
            composite_hits: self.stats.composite_hits.load(Ordering::Relaxed),
            composite_misses: self.stats.composite_misses.load(Ordering::Relaxed),
            composite_compile_count: self.stats.composite_compile_count.load(Ordering::Relaxed),
            family_sort_count: self.stats.family_sort_count.load(Ordering::Relaxed),
            family_visit_count: self.stats.family_visit_count.load(Ordering::Relaxed),
            face_visit_count: self.stats.face_visit_count.load(Ordering::Relaxed),
            coverage_probe_count: self.stats.coverage_probe_count.load(Ordering::Relaxed),
            eviction_count: state
                .families
                .eviction_count
                .saturating_add(state.composites.eviction_count)
                .saturating_add(state.candidates.eviction_count)
                .saturating_add(state.resolutions.eviction_count),
            family_entry_count: state.families.entries.len(),
            candidate_entry_count: state.candidates.entries.len(),
            resolution_entry_count: state.resolutions.entries.len(),
            composite_entry_count: state.composites.entries.len(),
            approximate_bytes: state
                .families
                .approximate_bytes
                .saturating_add(state.composites.approximate_bytes)
                .saturating_add(state.candidates.approximate_bytes)
                .saturating_add(state.resolutions.approximate_bytes),
        }
    }

    fn record_hit(&self, hit: bool, hits: &AtomicU64, misses: &AtomicU64) {
        if hit {
            hits.fetch_add(1, Ordering::Relaxed);
        } else {
            misses.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub(super) fn family_candidate_cache_key(
    family: FontFamilyIdentity,
    query: &FontQuery,
) -> FamilyCandidateCacheKey {
    let mut hasher = cache_hasher(b"family");
    hasher.update(family.as_bytes());
    update_query(&mut hasher, query);
    FamilyCandidateCacheKey(finish_key(hasher))
}

pub(super) fn fallback_candidate_cache_key(
    query: FallbackQueryIdentity,
    script: FontScript,
    codepoints: &[char],
) -> FallbackCandidateCacheKey {
    let mut hasher = cache_hasher(b"candidate");
    hasher.update(&query.0);
    update_script(&mut hasher, script);
    update_codepoints(&mut hasher, codepoints);
    FallbackCandidateCacheKey(finish_key(hasher))
}

pub(super) fn fallback_query_identity(
    query: &FontQuery,
    composite: Option<CompositeFontIdentity>,
    language: Option<&str>,
) -> FallbackQueryIdentity {
    let mut hasher = cache_hasher(b"query");
    update_query(&mut hasher, query);
    match composite {
        Some(composite) => {
            hasher.update(&[1]);
            hasher.update(&composite.0);
        }
        None => {
            hasher.update(&[0]);
        }
    };
    update_normalized_text(&mut hasher, language.unwrap_or_default());
    FallbackQueryIdentity(finish_key(hasher))
}

fn composite_font_identity(composite: &CompositeFontDescriptor) -> CompositeFontIdentity {
    let mut hasher = cache_hasher(b"composite");
    update_composite(&mut hasher, composite);
    CompositeFontIdentity(finish_key(hasher))
}

pub(super) fn fallback_resolution_cache_key(
    primary: FontFaceId,
    candidate_key: FallbackCandidateCacheKey,
    max_depth: u8,
) -> FallbackResolutionCacheKey {
    let mut hasher = cache_hasher(b"resolution");
    hasher.update(&primary.0.to_le_bytes());
    hasher.update(&candidate_key.0);
    hasher.update(&[max_depth]);
    FallbackResolutionCacheKey(finish_key(hasher))
}

fn cache_hasher(kind: &[u8]) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FALLBACK_CACHE_HASH_DOMAIN);
    hasher.update(kind);
    hasher
}

fn finish_key(hasher: blake3::Hasher) -> [u8; 16] {
    let mut key = [0_u8; 16];
    key.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    key
}

fn update_query(hasher: &mut blake3::Hasher, query: &FontQuery) {
    hasher.update(&(query.families.len() as u64).to_le_bytes());
    for family in &query.families {
        update_normalized_text(hasher, family.as_str());
    }
    hasher.update(&query.weight.0.to_le_bytes());
    match query.style {
        crate::text::FontStyle::Normal => hasher.update(&[0]),
        crate::text::FontStyle::Italic => hasher.update(&[1]),
        crate::text::FontStyle::Oblique(angle) => {
            hasher.update(&[2]);
            hasher.update(&angle.to_bits().to_le_bytes())
        }
    };
    hasher.update(&query.stretch.0.to_le_bytes());
}

fn update_composite(hasher: &mut blake3::Hasher, composite: &CompositeFontDescriptor) {
    update_normalized_text(hasher, composite.default_family.as_str());
    hasher.update(&(composite.sub_fonts.len() as u64).to_le_bytes());
    for sub_font in &composite.sub_fonts {
        update_sub_font(hasher, sub_font);
    }
}

fn update_sub_font(hasher: &mut blake3::Hasher, sub_font: &SubFontRange) {
    update_normalized_text(hasher, sub_font.family.as_str());
    hasher.update(&(sub_font.scripts.len() as u64).to_le_bytes());
    for script in &sub_font.scripts {
        update_script(hasher, *script);
    }
    hasher.update(&(sub_font.ranges.len() as u64).to_le_bytes());
    for (start, end) in &sub_font.ranges {
        hasher.update(&start.to_le_bytes());
        hasher.update(&end.to_le_bytes());
    }
    hasher.update(&(sub_font.cultures.len() as u64).to_le_bytes());
    for culture in &sub_font.cultures {
        update_normalized_text(hasher, culture.as_str());
    }
}

fn update_script(hasher: &mut blake3::Hasher, script: FontScript) {
    let (kind, detail) = match script {
        FontScript::Latin => (0, 0),
        FontScript::Cyrillic => (1, 0),
        FontScript::Greek => (2, 0),
        FontScript::Han => (3, 0),
        FontScript::Hiragana => (4, 0),
        FontScript::Katakana => (5, 0),
        FontScript::Hangul => (6, 0),
        FontScript::Arabic => (7, 0),
        FontScript::Hebrew => (8, 0),
        FontScript::Devanagari => (9, 0),
        FontScript::Other(detail) => (10, detail),
    };
    hasher.update(&[kind]);
    hasher.update(&detail.to_le_bytes());
}

fn update_codepoints(hasher: &mut blake3::Hasher, codepoints: &[char]) {
    hasher.update(&(codepoints.len() as u64).to_le_bytes());
    for codepoint in codepoints {
        hasher.update(&(*codepoint as u32).to_le_bytes());
    }
}

fn update_normalized_text(hasher: &mut blake3::Hasher, value: &str) {
    let value = value.trim();
    hasher.update(&(value.len() as u64).to_le_bytes());
    for byte in value.bytes() {
        hasher.update(&[byte.to_ascii_lowercase()]);
    }
}

#[cfg(test)]
#[path = "fallback_cache/tests.rs"]
mod tests;
