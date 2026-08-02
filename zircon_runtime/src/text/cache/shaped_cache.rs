use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

use crate::core::framework::text::TextDirection;
use crate::text::font::shared_font_database_generation;
use crate::text::{
    BackendShapeRequest, OpenTypeFeature, ShapedGlyphRun, TextOrientation, VerticalMode,
};
use crate::text::{TextRange, TextStyle};

mod memory;

use memory::estimated_entry_bytes;

pub(crate) const DEFAULT_SHAPED_RUN_CACHE_CAPACITY: usize = 1024;
pub(crate) const DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheKey {
    pub(crate) text_hash: u64,
    pub(crate) source_range: TextRange,
    pub(crate) font_family: Option<String>,
    pub(crate) font_weight: u16,
    pub(crate) font_size_bits: u32,
    pub(crate) line_height_bits: u32,
    pub(crate) tab_size_bits: u32,
    pub(crate) base_direction: TextDirection,
    pub(crate) orientation: TextOrientation,
    pub(crate) vertical_mode: VerticalMode,
    include_kerning: bool,
    pub(crate) features_hash: u64,
    features: Arc<[OpenTypeFeature]>,
    pub(crate) language: Option<String>,
    pub(crate) font_database_generation: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ShapedRunCacheLookupKey<'a> {
    text_hash: u64,
    source_range: TextRange,
    font_family: Option<&'a str>,
    font_weight: u16,
    font_size_bits: u32,
    line_height_bits: u32,
    tab_size_bits: u32,
    base_direction: TextDirection,
    orientation: TextOrientation,
    vertical_mode: VerticalMode,
    include_kerning: bool,
    features_hash: u64,
    features: &'a [OpenTypeFeature],
    language: Option<&'a str>,
    font_database_generation: u64,
}

impl ShapedRunCacheKey {
    pub(crate) fn from_request(request: &BackendShapeRequest<'_>) -> Self {
        Self::from_lookup(&ShapedRunCacheLookupKey::from_request(request))
    }

    fn from_lookup(lookup: &ShapedRunCacheLookupKey<'_>) -> Self {
        Self {
            text_hash: lookup.text_hash,
            source_range: lookup.source_range,
            font_family: lookup.font_family.map(ToOwned::to_owned),
            font_weight: lookup.font_weight,
            font_size_bits: lookup.font_size_bits,
            line_height_bits: lookup.line_height_bits,
            tab_size_bits: lookup.tab_size_bits,
            base_direction: lookup.base_direction,
            orientation: lookup.orientation,
            vertical_mode: lookup.vertical_mode,
            include_kerning: lookup.include_kerning,
            features_hash: lookup.features_hash,
            features: Arc::from(lookup.features),
            language: owned_normalized_language_tag(lookup.language),
            font_database_generation: lookup.font_database_generation,
        }
    }

    fn lookup(&self) -> ShapedRunCacheLookupKey<'_> {
        ShapedRunCacheLookupKey {
            text_hash: self.text_hash,
            source_range: self.source_range,
            font_family: self.font_family.as_deref(),
            font_weight: self.font_weight,
            font_size_bits: self.font_size_bits,
            line_height_bits: self.line_height_bits,
            tab_size_bits: self.tab_size_bits,
            base_direction: self.base_direction,
            orientation: self.orientation,
            vertical_mode: self.vertical_mode,
            include_kerning: self.include_kerning,
            features_hash: self.features_hash,
            features: self.features.as_ref(),
            language: self.language.as_deref(),
            font_database_generation: self.font_database_generation,
        }
    }

    pub(crate) fn matches_lookup(&self, lookup: &ShapedRunCacheLookupKey<'_>) -> bool {
        self.text_hash == lookup.text_hash
            && self.source_range == lookup.source_range
            && self.font_family.as_deref() == lookup.font_family
            && self.font_weight == lookup.font_weight
            && self.font_size_bits == lookup.font_size_bits
            && self.line_height_bits == lookup.line_height_bits
            && self.tab_size_bits == lookup.tab_size_bits
            && self.base_direction == lookup.base_direction
            && self.orientation == lookup.orientation
            && self.vertical_mode == lookup.vertical_mode
            && self.include_kerning == lookup.include_kerning
            && self.features_hash == lookup.features_hash
            && self.features.as_ref() == lookup.features
            && normalized_language_matches(self.language.as_deref(), lookup.language)
            && self.font_database_generation == lookup.font_database_generation
    }

    pub(crate) const fn font_database_generation(&self) -> u64 {
        self.font_database_generation
    }
}

impl<'a> ShapedRunCacheLookupKey<'a> {
    pub(crate) fn from_request(request: &BackendShapeRequest<'a>) -> Self {
        debug_assert!(request.features_are_normalized());
        let font_size = request.style.font_size.max(1.0);
        let line_height = request.style.line_height.max(font_size);

        Self {
            text_hash: hash_text(request.text),
            source_range: request.source_range,
            font_family: cache_font_family_ref(request.style),
            font_weight: TextStyle::normalized_font_weight(request.style.font_weight),
            font_size_bits: normalized_f32_bits(font_size),
            line_height_bits: normalized_f32_bits(line_height),
            tab_size_bits: normalized_f32_bits(request.style.tab_size),
            base_direction: request.base_direction,
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            include_kerning: request.include_kerning,
            features_hash: shaping_features_hash(request),
            features: request.features(),
            language: cache_language_tag(request.language),
            font_database_generation: shared_font_database_generation(),
        }
    }

    pub(crate) fn exact_fingerprint(&self) -> u64 {
        self.full_fingerprint()
    }

    pub(crate) const fn font_database_generation(&self) -> u64 {
        self.font_database_generation
    }

    fn full_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.text_hash.hash(&mut hasher);
        self.source_range.start.hash(&mut hasher);
        self.source_range.end.hash(&mut hasher);
        self.font_family.map(hash_text).hash(&mut hasher);
        self.font_weight.hash(&mut hasher);
        self.font_size_bits.hash(&mut hasher);
        self.line_height_bits.hash(&mut hasher);
        self.tab_size_bits.hash(&mut hasher);
        std::mem::discriminant(&self.base_direction).hash(&mut hasher);
        std::mem::discriminant(&self.orientation).hash(&mut hasher);
        std::mem::discriminant(&self.vertical_mode).hash(&mut hasher);
        self.include_kerning.hash(&mut hasher);
        self.features_hash.hash(&mut hasher);
        normalized_language_hash(self.language).hash(&mut hasher);
        self.font_database_generation.hash(&mut hasher);
        hasher.finish()
    }

    fn direction_alias_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.text_hash.hash(&mut hasher);
        self.source_range.start.hash(&mut hasher);
        self.source_range.end.hash(&mut hasher);
        self.font_family.map(hash_text).hash(&mut hasher);
        self.font_weight.hash(&mut hasher);
        self.font_size_bits.hash(&mut hasher);
        self.line_height_bits.hash(&mut hasher);
        self.tab_size_bits.hash(&mut hasher);
        std::mem::discriminant(&self.orientation).hash(&mut hasher);
        std::mem::discriminant(&self.vertical_mode).hash(&mut hasher);
        self.include_kerning.hash(&mut hasher);
        self.features_hash.hash(&mut hasher);
        normalized_language_hash(self.language).hash(&mut hasher);
        self.font_database_generation.hash(&mut hasher);
        hasher.finish()
    }

    fn owned_key_byte_len(&self) -> usize {
        self.font_family
            .map_or(0, str::len)
            .saturating_add(normalized_language_len(self.language))
            .saturating_add(
                self.features
                    .len()
                    .saturating_mul(size_of::<OpenTypeFeature>()),
            )
    }
}

impl Hash for ShapedRunCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text_hash.hash(state);
        self.source_range.start.hash(state);
        self.source_range.end.hash(state);
        self.font_family.hash(state);
        self.font_weight.hash(state);
        self.font_size_bits.hash(state);
        self.line_height_bits.hash(state);
        self.tab_size_bits.hash(state);
        std::mem::discriminant(&self.base_direction).hash(state);
        std::mem::discriminant(&self.orientation).hash(state);
        std::mem::discriminant(&self.vertical_mode).hash(state);
        self.include_kerning.hash(state);
        self.features_hash.hash(state);
        self.features.hash(state);
        self.language.hash(state);
        self.font_database_generation.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) max_bytes: usize,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) owned_key_allocation_bytes: u64,
    pub(crate) eviction_scan_count: u64,
    pub(crate) entry_move_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ShapedRunCacheEntry {
    key: ShapedRunCacheKey,
    run: Arc<ShapedGlyphRun>,
    estimated_bytes: usize,
}

impl IndexedTextCacheEntry<ShapedRunCacheKey> for ShapedRunCacheEntry {
    fn cache_key(&self) -> &ShapedRunCacheKey {
        &self.key
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedRunCache {
    index: IndexedTextCache<ShapedRunCacheKey, ShapedRunCacheEntry>,
    lookup_buckets: HashMap<u64, Vec<TextCacheSlot>>,
    lookup_bucket_positions: HashMap<TextCacheSlot, usize>,
    direction_alias_buckets: HashMap<u64, Vec<TextCacheSlot>>,
    direction_alias_bucket_positions: HashMap<TextCacheSlot, usize>,
    capacity: usize,
    max_bytes: usize,
    estimated_bytes: usize,
    frame_report: ShapedRunCacheReport,
}

impl Default for ShapedRunCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapedRunCache {
    pub(crate) fn new() -> Self {
        Self::with_limits(
            DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
            DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
        )
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_limits(capacity, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES)
    }

    pub(crate) fn with_limits(capacity: usize, max_bytes: usize) -> Self {
        let mut cache = Self {
            index: IndexedTextCache::new(),
            lookup_buckets: HashMap::new(),
            lookup_bucket_positions: HashMap::new(),
            direction_alias_buckets: HashMap::new(),
            direction_alias_bucket_positions: HashMap::new(),
            capacity,
            max_bytes,
            estimated_bytes: 0,
            frame_report: ShapedRunCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache.frame_report.max_bytes = max_bytes;
        cache
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.frame_report = ShapedRunCacheReport {
            frame_index,
            capacity: self.capacity,
            max_bytes: self.max_bytes,
            entry_count: self.index.len(),
            estimated_bytes: self.estimated_bytes,
            ..ShapedRunCacheReport::default()
        };
    }

    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_limits();
    }

    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.index.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.index.clear();
        self.lookup_buckets.clear();
        self.lookup_bucket_positions.clear();
        self.direction_alias_buckets.clear();
        self.direction_alias_bucket_positions.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    pub(crate) fn report(&self) -> ShapedRunCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    pub(crate) fn contains_exact(&self, key: &ShapedRunCacheKey, text: &str) -> bool {
        self.index
            .find_slot(key, |entry| entry.run.source_text.as_ref() == text)
            .slot
            .is_some()
    }

    pub(crate) fn get(
        &mut self,
        key: &ShapedRunCacheKey,
        text: &str,
    ) -> Option<Arc<ShapedGlyphRun>> {
        self.get_with_lookup(&key.lookup(), text)
    }

    pub(crate) fn get_with_lookup(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
        text: &str,
    ) -> Option<Arc<ShapedGlyphRun>> {
        let mut collision_seen = false;
        let exact_lookup = self
            .lookup_buckets
            .get(&lookup.full_fingerprint())
            .map(|candidates| {
                self.index.find_in_slots(candidates, |entry| {
                    if !entry.key.matches_lookup(lookup) {
                        return false;
                    }
                    if entry.run.source_text.as_ref() == text {
                        true
                    } else {
                        collision_seen = true;
                        false
                    }
                })
            })
            .unwrap_or_default();
        self.record_lookup(exact_lookup.candidate_count);
        let hit_slot = if exact_lookup.slot.is_some() || !is_single_text_paragraph(text) {
            exact_lookup.slot
        } else {
            let alias_lookup = self
                .direction_alias_buckets
                .get(&lookup.direction_alias_fingerprint())
                .map(|candidates| {
                    self.index.find_in_slots(candidates, |entry| {
                        entry
                            .key
                            .can_reuse_resolved_direction_for_lookup(lookup, &entry.run)
                            && entry.run.source_text.as_ref() == text
                    })
                })
                .unwrap_or_default();
            self.record_lookup(alias_lookup.candidate_count);
            alias_lookup.slot
        };

        let Some(slot) = hit_slot else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            return None;
        };

        let run = Arc::clone(&self.index.entry(slot)?.run);
        self.index.touch(slot);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(run)
    }

    pub(crate) fn own_lookup_key(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
    ) -> ShapedRunCacheKey {
        self.frame_report.owned_key_allocation_bytes = self
            .frame_report
            .owned_key_allocation_bytes
            .saturating_add(lookup.owned_key_byte_len() as u64);
        ShapedRunCacheKey::from_lookup(lookup)
    }

    pub(crate) fn insert(
        &mut self,
        key: ShapedRunCacheKey,
        run: ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        let run = Arc::new(run);
        let estimated_bytes = estimated_entry_bytes(&key, run.as_ref());

        let lookup = self
            .index
            .find_slot(&key, |entry| entry.run.source_text == run.source_text);
        if let Some(slot) = lookup.slot {
            let updated = if let Some(entry) = self.index.entry_mut(slot) {
                self.estimated_bytes = self
                    .estimated_bytes
                    .saturating_sub(entry.estimated_bytes)
                    .saturating_add(estimated_bytes);
                entry.run = Arc::clone(&run);
                entry.estimated_bytes = estimated_bytes;
                true
            } else {
                false
            };
            if updated {
                self.index.touch(slot);
                self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
                self.trim_to_limits();
                return run;
            }
        }

        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        let lookup = key.lookup();
        let lookup_fingerprint = lookup.full_fingerprint();
        let alias_fingerprint = lookup.direction_alias_fingerprint();
        let slot = self.index.insert(ShapedRunCacheEntry {
            key,
            run: Arc::clone(&run),
            estimated_bytes,
        });
        Self::insert_bucket_slot(
            &mut self.lookup_buckets,
            &mut self.lookup_bucket_positions,
            lookup_fingerprint,
            slot,
        );
        Self::insert_bucket_slot(
            &mut self.direction_alias_buckets,
            &mut self.direction_alias_bucket_positions,
            alias_fingerprint,
            slot,
        );
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.trim_to_limits();
        run
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        key: ShapedRunCacheKey,
        text: &str,
        shape: impl FnOnce() -> ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        if let Some(run) = self.get(&key, text) {
            return run;
        }
        self.insert(key, shape())
    }

    fn trim_to_limits(&mut self) {
        let mut evicted = 0_u64;
        while self.over_limits() {
            let Some((slot, removed)) = self.index.pop_oldest_with_slot() else {
                break;
            };
            let lookup = removed.key.lookup();
            self.remove_lookup_slot(lookup.full_fingerprint(), slot);
            self.remove_direction_alias_slot(lookup.direction_alias_fingerprint(), slot);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(removed.estimated_bytes);
            evicted = evicted.saturating_add(1);
        }

        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
    }

    fn over_limits(&self) -> bool {
        self.index.len() > self.capacity || self.estimated_bytes > self.max_bytes
    }

    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    fn remove_lookup_slot(&mut self, fingerprint: u64, slot: TextCacheSlot) {
        let remove_bucket = if let Some(candidates) = self.lookup_buckets.get_mut(&fingerprint) {
            Self::remove_bucket_slot(candidates, &mut self.lookup_bucket_positions, slot);
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.lookup_buckets.remove(&fingerprint);
        }
    }

    fn remove_direction_alias_slot(&mut self, fingerprint: u64, slot: TextCacheSlot) {
        let remove_bucket = if let Some(candidates) =
            self.direction_alias_buckets.get_mut(&fingerprint)
        {
            Self::remove_bucket_slot(candidates, &mut self.direction_alias_bucket_positions, slot);
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.direction_alias_buckets.remove(&fingerprint);
        }
    }

    fn insert_bucket_slot(
        buckets: &mut HashMap<u64, Vec<TextCacheSlot>>,
        positions: &mut HashMap<TextCacheSlot, usize>,
        fingerprint: u64,
        slot: TextCacheSlot,
    ) {
        let candidates = buckets.entry(fingerprint).or_default();
        let position = candidates.len();
        candidates.push(slot);
        positions.insert(slot, position);
    }

    fn remove_bucket_slot(
        candidates: &mut Vec<TextCacheSlot>,
        positions: &mut HashMap<TextCacheSlot, usize>,
        slot: TextCacheSlot,
    ) {
        let Some(position) = positions.remove(&slot) else {
            return;
        };
        let removed = candidates.swap_remove(position);
        debug_assert_eq!(removed, slot);
        if let Some(&moved_slot) = candidates.get(position) {
            positions.insert(moved_slot, position);
        }
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
        self.frame_report.estimated_bytes = self.estimated_bytes;
    }
}

impl ShapedRunCacheKey {
    fn can_reuse_resolved_direction_for_lookup(
        &self,
        requested: &ShapedRunCacheLookupKey<'_>,
        cached_run: &ShapedGlyphRun,
    ) -> bool {
        matches!(
            self.base_direction,
            TextDirection::Auto | TextDirection::Mixed
        ) && matches!(
            requested.base_direction,
            TextDirection::LeftToRight | TextDirection::RightToLeft
        ) && cached_run.direction == requested.base_direction
            && self.matches_lookup_except_base_direction(requested)
    }

    fn matches_lookup_except_base_direction(&self, lookup: &ShapedRunCacheLookupKey<'_>) -> bool {
        self.text_hash == lookup.text_hash
            && self.source_range == lookup.source_range
            && self.font_family.as_deref() == lookup.font_family
            && self.font_weight == lookup.font_weight
            && self.font_size_bits == lookup.font_size_bits
            && self.line_height_bits == lookup.line_height_bits
            && self.tab_size_bits == lookup.tab_size_bits
            && self.orientation == lookup.orientation
            && self.vertical_mode == lookup.vertical_mode
            && self.include_kerning == lookup.include_kerning
            && self.features_hash == lookup.features_hash
            && self.features.as_ref() == lookup.features
            && normalized_language_matches(self.language.as_deref(), lookup.language)
            && self.font_database_generation == lookup.font_database_generation
    }
}

fn is_single_text_paragraph(text: &str) -> bool {
    !text.chars().any(|ch| {
        matches!(
            ch,
            '\n' | '\r'
                | '\u{001c}'
                | '\u{001d}'
                | '\u{001e}'
                | '\u{0085}'
                | '\u{2028}'
                | '\u{2029}'
        )
    })
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn shaping_features_hash(request: &BackendShapeRequest<'_>) -> u64 {
    let mut hasher = DefaultHasher::new();
    b"shaped-run-features-v2".hash(&mut hasher);
    request.include_kerning.hash(&mut hasher);
    request.features().hash(&mut hasher);
    hasher.finish()
}

fn cache_font_family_ref(style: &TextStyle) -> Option<&str> {
    style
        .font_family
        .as_deref()
        .or(style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty())
}

fn cache_language_tag(language: Option<&str>) -> Option<&str> {
    language
        .map(str::trim)
        .filter(|language| !language.is_empty())
}

fn normalized_language_hash(language: Option<&str>) -> Option<u64> {
    let language = cache_language_tag(language)?;
    let mut hasher = DefaultHasher::new();
    for byte in language.bytes() {
        let normalized = if byte == b'_' {
            b'-'
        } else {
            byte.to_ascii_lowercase()
        };
        normalized.hash(&mut hasher);
    }
    Some(hasher.finish())
}

fn owned_normalized_language_tag(language: Option<&str>) -> Option<String> {
    let language = cache_language_tag(language)?;
    let mut normalized = String::with_capacity(language.len());
    for character in language.chars() {
        normalized.push(if character == '_' {
            '-'
        } else {
            character.to_ascii_lowercase()
        });
    }
    Some(normalized)
}

fn normalized_language_len(language: Option<&str>) -> usize {
    cache_language_tag(language).map_or(0, str::len)
}

fn normalized_language_matches(normalized: Option<&str>, requested: Option<&str>) -> bool {
    let requested = cache_language_tag(requested);
    match (normalized, requested) {
        (None, None) => true,
        (Some(normalized), Some(requested)) if normalized.len() == requested.len() => normalized
            .bytes()
            .zip(requested.bytes())
            .all(|(stored, requested)| {
                stored
                    == if requested == b'_' {
                        b'-'
                    } else {
                        requested.to_ascii_lowercase()
                    }
            }),
        _ => false,
    }
}

fn normalized_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

#[cfg(test)]
mod tests;
