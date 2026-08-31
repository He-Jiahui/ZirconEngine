use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use glyphon::SwashContent;

use crate::text::atlas::{GlyphAtlasFormat, GlyphRasterKey, GlyphSmoothingMode};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkerPool, TextRasterWorkerRequestError,
};
use crate::text::raster::{GlyphBitmap, SwashRasterRequest};
use crate::text::{FontFaceId, InstancedFaceId, VariationCoords};

mod lru;
mod report;
mod worker;

use lru::{NativeBitmapAtlasSourceCacheEntry, NativeBitmapAtlasSourceLru};
pub(crate) use report::NativeBitmapAtlasSourceCacheFrameReport;

const DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_CAPACITY: usize = 2048;
const DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;
const FIRST_NATIVE_BITMAP_ATLAS_WORK_ID: u64 = 1;
const FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID: u64 = 1;
pub(crate) const NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NativeBitmapAtlasReadinessGeneration(u64);

impl NativeBitmapAtlasReadinessGeneration {
    fn next(current: Self) -> Self {
        Self(current.0.saturating_add(1).max(1))
    }

    pub(crate) fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeBitmapAtlasReadinessChangeReceipt {
    generation: NativeBitmapAtlasReadinessGeneration,
    full_invalidation: bool,
    changed_keys: HashSet<GlyphRasterKey>,
}

impl NativeBitmapAtlasReadinessChangeReceipt {
    pub(crate) fn generation(&self) -> NativeBitmapAtlasReadinessGeneration {
        self.generation
    }

    pub(crate) fn full_invalidation(&self) -> bool {
        self.full_invalidation
    }

    pub(crate) fn changed_keys(&self) -> &HashSet<GlyphRasterKey> {
        &self.changed_keys
    }

    pub(crate) fn changed_key_count(&self) -> usize {
        self.changed_keys().len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasWorkerRequestStatus {
    Submitted(TextRasterWorkId),
    Pending,
    DeferredByFrameBudget,
    DeferredByWorkerBackpressure,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeBitmapAtlasCachedGlyphImage {
    pub(crate) content: SwashContent,
    pub(crate) top: i16,
    pub(crate) left: i16,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) bytes: Arc<[u8]>,
}

#[derive(Debug)]
pub(crate) struct NativeBitmapAtlasSourceCache {
    capacity: usize,
    max_byte_count: usize,
    resident_byte_count: usize,
    face_epoch: u64,
    readiness_generation: NativeBitmapAtlasReadinessGeneration,
    pending_readiness_full_invalidation: bool,
    pending_readiness_changed_keys: HashSet<GlyphRasterKey>,
    // The text-owned glyph identity, rather than a renderer layout cache key, owns residency.
    entries: HashMap<GlyphRasterKey, NativeBitmapAtlasSourceCacheEntry>,
    lru: NativeBitmapAtlasSourceLru,
    // A raster can select a color bitmap after the request was issued as an alpha glyph. Keep a
    // direct reverse lookup from its actual persistent atlas key to the text-owned request key.
    cache_keys_by_raster_key: HashMap<GlyphRasterKey, GlyphRasterKey>,
    budget_evicted_raster_keys: Vec<GlyphRasterKey>,
    pending_worker_cache_keys: HashMap<TextRasterWorkId, GlyphRasterKey>,
    pending_worker_work_ids: HashMap<GlyphRasterKey, TextRasterWorkId>,
    // Font bytes are shared by all variation instances of the same face for this epoch.
    raster_font_bytes_by_face: HashMap<FontFaceId, Arc<[u8]>>,
    raster_variations_by_instance: HashMap<InstancedFaceId, Arc<VariationCoords>>,
    raster_font_resident_byte_count: usize,
    // A worker-local Swash context can safely reuse parsed face state for a stable source face.
    raster_font_identity_by_face: HashMap<FontFaceId, u64>,
    next_raster_font_id: u64,
    next_worker_id: u64,
    pending_face_invalidated_count: usize,
    pending_worker_cancelled_count: usize,
    pending_linked_raster_invalidation_count: usize,
    frame_report: NativeBitmapAtlasSourceCacheFrameReport,
}

impl Default for NativeBitmapAtlasSourceCache {
    fn default() -> Self {
        Self::with_limits(
            DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_CAPACITY,
            DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_MAX_BYTES,
        )
    }
}

impl NativeBitmapAtlasSourceCache {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_limits(capacity, DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_MAX_BYTES)
    }

    pub(crate) fn with_limits(capacity: usize, max_byte_count: usize) -> Self {
        Self {
            capacity,
            max_byte_count,
            resident_byte_count: 0,
            face_epoch: 0,
            readiness_generation: NativeBitmapAtlasReadinessGeneration::default(),
            pending_readiness_full_invalidation: false,
            pending_readiness_changed_keys: HashSet::new(),
            entries: HashMap::new(),
            lru: NativeBitmapAtlasSourceLru::default(),
            cache_keys_by_raster_key: HashMap::new(),
            budget_evicted_raster_keys: Vec::new(),
            pending_worker_cache_keys: HashMap::new(),
            pending_worker_work_ids: HashMap::new(),
            raster_font_bytes_by_face: HashMap::new(),
            raster_variations_by_instance: HashMap::new(),
            raster_font_resident_byte_count: 0,
            raster_font_identity_by_face: HashMap::new(),
            next_raster_font_id: FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID,
            next_worker_id: FIRST_NATIVE_BITMAP_ATLAS_WORK_ID,
            pending_face_invalidated_count: 0,
            pending_worker_cancelled_count: 0,
            pending_linked_raster_invalidation_count: 0,
            frame_report: NativeBitmapAtlasSourceCacheFrameReport::default(),
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        let invalidated_count = std::mem::take(&mut self.pending_face_invalidated_count);
        let cancelled_count = std::mem::take(&mut self.pending_worker_cancelled_count);
        let linked_raster_invalidation_count =
            std::mem::take(&mut self.pending_linked_raster_invalidation_count);
        self.frame_report = NativeBitmapAtlasSourceCacheFrameReport {
            capacity: self.capacity,
            max_byte_count: self.max_byte_count,
            resident_byte_count: self.resident_byte_count,
            evicted_count: invalidated_count,
            invalidated_count,
            worker_request_cancelled_count: cancelled_count,
            linked_raster_invalidation_count,
            entry_count: self.entries.len(),
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        };
    }

    pub(crate) fn frame_report(&self) -> NativeBitmapAtlasSourceCacheFrameReport {
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: self.capacity,
            max_byte_count: self.max_byte_count,
            resident_byte_count: self.resident_byte_count,
            entry_count: self.entries.len(),
            persistent_raster_key_count: self.cache_keys_by_raster_key.len(),
            pending_worker_count: self.pending_worker_cache_keys.len(),
            worker_raster_font_resident_byte_count: self.raster_font_resident_byte_count,
            worker_raster_font_entry_count: self.raster_font_bytes_by_face.len(),
            ..self.frame_report
        }
    }

    pub(crate) fn face_epoch(&self) -> u64 {
        self.face_epoch
    }

    pub(crate) fn readiness_generation(&self) -> NativeBitmapAtlasReadinessGeneration {
        self.readiness_generation
    }

    pub(crate) fn take_readiness_changes(&mut self) -> NativeBitmapAtlasReadinessChangeReceipt {
        NativeBitmapAtlasReadinessChangeReceipt {
            generation: self.readiness_generation(),
            full_invalidation: std::mem::take(&mut self.pending_readiness_full_invalidation),
            changed_keys: std::mem::take(&mut self.pending_readiness_changed_keys),
        }
    }

    pub(crate) fn idle_frame_report(&mut self) -> NativeBitmapAtlasSourceCacheFrameReport {
        self.begin_frame();
        self.frame_report()
    }

    pub(crate) fn discard_all_for_face_invalidation(&mut self) {
        self.discard_all_for_face_invalidation_with_worker_pool(None);
    }

    pub(crate) fn discard_all_for_face_invalidation_with_worker_pool(
        &mut self,
        worker_pool: Option<&TextRasterWorkerPool>,
    ) {
        let invalidated_count = self.entries.len();
        self.entries.clear();
        self.resident_byte_count = 0;
        self.lru.clear();
        self.cache_keys_by_raster_key.clear();
        self.budget_evicted_raster_keys.clear();
        self.cancel_pending_worker_requests(worker_pool);
        self.raster_font_bytes_by_face.clear();
        self.raster_variations_by_instance.clear();
        self.raster_font_resident_byte_count = 0;
        self.raster_font_identity_by_face.clear();
        self.next_raster_font_id = FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID;
        self.face_epoch = self.face_epoch.saturating_add(1);
        self.record_full_readiness_invalidation();
        self.pending_face_invalidated_count = self
            .pending_face_invalidated_count
            .saturating_add(invalidated_count);
    }

    pub(crate) fn register_worker_request(
        &mut self,
        work_id: TextRasterWorkId,
        cache_key: GlyphRasterKey,
    ) {
        if let Some(previous_cache_key) = self.pending_worker_cache_keys.insert(work_id, cache_key)
        {
            self.pending_worker_work_ids.remove(&previous_cache_key);
        }
        if let Some(previous_work_id) = self.pending_worker_work_ids.insert(cache_key, work_id) {
            self.pending_worker_cache_keys.remove(&previous_work_id);
        }
    }

    pub(crate) fn cached_image(
        &mut self,
        cache_key: GlyphRasterKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        if let Some(image) = self.touch_entry(cache_key) {
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return Some(image);
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        None
    }

    pub(crate) fn bind_persistent_raster_key(
        &mut self,
        cache_key: GlyphRasterKey,
        raster_key: GlyphRasterKey,
    ) -> bool {
        let Some(previous_raster_key) = self.entries.get(&cache_key).map(|entry| entry.raster_key)
        else {
            return false;
        };
        if previous_raster_key == Some(raster_key) {
            return true;
        }
        if let Some(previous_raster_key) = previous_raster_key {
            self.cache_keys_by_raster_key.remove(&previous_raster_key);
        }
        let displaced_cache_key = self
            .cache_keys_by_raster_key
            .insert(raster_key, cache_key)
            .filter(|previous_cache_key| *previous_cache_key != cache_key);
        if let Some(previous_cache_key) = displaced_cache_key {
            if let Some(previous_entry) = self.entries.get_mut(&previous_cache_key) {
                previous_entry.raster_key = None;
            }
        }
        if let Some(entry) = self.entries.get_mut(&cache_key) {
            entry.raster_key = Some(raster_key);
        }
        self.record_readiness_change(cache_key);
        if let Some(displaced_cache_key) = displaced_cache_key {
            self.record_readiness_change(displaced_cache_key);
        }
        true
    }

    pub(crate) fn take_budget_evicted_raster_keys(&mut self) -> Vec<GlyphRasterKey> {
        std::mem::take(&mut self.budget_evicted_raster_keys)
    }

    pub(crate) fn invalidate_raster_keys<I>(&mut self, raster_keys: I)
    where
        I: IntoIterator<Item = GlyphRasterKey>,
    {
        let invalidated_count = self.invalidate_raster_keys_inner(raster_keys);
        self.frame_report.linked_raster_invalidation_count = self
            .frame_report
            .linked_raster_invalidation_count
            .saturating_add(invalidated_count);
    }

    pub(crate) fn invalidate_raster_keys_for_next_frame<I>(&mut self, raster_keys: I)
    where
        I: IntoIterator<Item = GlyphRasterKey>,
    {
        let invalidated_count = self.invalidate_raster_keys_inner(raster_keys);
        self.pending_linked_raster_invalidation_count = self
            .pending_linked_raster_invalidation_count
            .saturating_add(invalidated_count);
    }

    pub(crate) fn approximate_cached_image(
        &mut self,
        cache_key: GlyphRasterKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        for vertical_subpixel_bin in
            approximate_vertical_bin_candidates(cache_key.vertical_subpixel_bin)
        {
            let candidate_key = GlyphRasterKey {
                vertical_subpixel_bin,
                ..cache_key
            };
            self.frame_report.approximate_probe_count =
                self.frame_report.approximate_probe_count.saturating_add(1);
            if let Some(image) = self.touch_entry(candidate_key) {
                self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
                self.frame_report.approximate_hit_count =
                    self.frame_report.approximate_hit_count.saturating_add(1);
                return Some(image);
            }
        }
        None
    }

    fn insert(
        &mut self,
        cache_key: GlyphRasterKey,
        image: NativeBitmapAtlasCachedGlyphImage,
    ) -> bool {
        let image_byte_count = image.bytes.len();
        if self.capacity == 0 || self.max_byte_count == 0 || image_byte_count > self.max_byte_count
        {
            self.frame_report.rejected_byte_budget_count = self
                .frame_report
                .rejected_byte_budget_count
                .saturating_add(1);
            return false;
        }
        let (existing, repaired) = self.lru.remove(&mut self.entries, cache_key);
        self.record_lru_repair(repaired);
        if let Some(existing) = existing {
            self.remove_raster_key_binding(cache_key, existing.raster_key);
            self.resident_byte_count = self
                .resident_byte_count
                .saturating_sub(existing.image.bytes.len());
        }
        while self.entries.len() >= self.capacity
            || self.resident_byte_count.saturating_add(image_byte_count) > self.max_byte_count
        {
            if !self.evict_least_recently_used() {
                self.frame_report.rejected_byte_budget_count = self
                    .frame_report
                    .rejected_byte_budget_count
                    .saturating_add(1);
                return false;
            }
        }
        let repaired = self
            .lru
            .insert_most_recent(&mut self.entries, cache_key, image);
        self.record_lru_repair(repaired);
        self.resident_byte_count = self.resident_byte_count.saturating_add(image_byte_count);
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.record_readiness_change(cache_key);
        true
    }

    fn evict_least_recently_used(&mut self) -> bool {
        let (entry, repaired) = self.lru.pop_least_recent(&mut self.entries);
        self.record_lru_repair(repaired);
        let Some((cache_key, entry)) = entry else {
            return false;
        };
        if let Some(raster_key) = entry.raster_key {
            self.cache_keys_by_raster_key.remove(&raster_key);
            self.budget_evicted_raster_keys.push(raster_key);
            self.frame_report.budget_linked_eviction_count = self
                .frame_report
                .budget_linked_eviction_count
                .saturating_add(1);
        }
        let byte_count = entry.image.bytes.len();
        self.resident_byte_count = self.resident_byte_count.saturating_sub(byte_count);
        self.frame_report.evicted_count = self.frame_report.evicted_count.saturating_add(1);
        self.frame_report.evicted_byte_count = self
            .frame_report
            .evicted_byte_count
            .saturating_add(byte_count);
        self.record_readiness_change(cache_key);
        true
    }

    fn touch_entry(
        &mut self,
        cache_key: GlyphRasterKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let (image, repaired) = self.lru.touch(&mut self.entries, cache_key);
        self.record_lru_repair(repaired);
        if image.is_some() {
            self.frame_report.lru_touch_count = self.frame_report.lru_touch_count.saturating_add(1);
        }
        image
    }

    fn invalidate_raster_keys_inner<I>(&mut self, raster_keys: I) -> usize
    where
        I: IntoIterator<Item = GlyphRasterKey>,
    {
        let mut invalidated_count = 0_usize;
        for raster_key in raster_keys {
            let Some(cache_key) = self.cache_keys_by_raster_key.remove(&raster_key) else {
                continue;
            };
            let (entry, repaired) = self.lru.remove(&mut self.entries, cache_key);
            self.record_lru_repair(repaired);
            let Some(entry) = entry else {
                continue;
            };
            self.resident_byte_count = self
                .resident_byte_count
                .saturating_sub(entry.image.bytes.len());
            self.frame_report.evicted_count = self.frame_report.evicted_count.saturating_add(1);
            self.frame_report.evicted_byte_count = self
                .frame_report
                .evicted_byte_count
                .saturating_add(entry.image.bytes.len());
            self.record_readiness_change(cache_key);
            invalidated_count = invalidated_count.saturating_add(1);
        }
        invalidated_count
    }

    fn remove_raster_key_binding(
        &mut self,
        cache_key: GlyphRasterKey,
        raster_key: Option<GlyphRasterKey>,
    ) {
        if let Some(raster_key) = raster_key {
            if self.cache_keys_by_raster_key.get(&raster_key) == Some(&cache_key) {
                self.cache_keys_by_raster_key.remove(&raster_key);
            }
        }
    }

    fn record_lru_repair(&mut self, repaired: bool) {
        if repaired {
            self.frame_report.lru_repair_count =
                self.frame_report.lru_repair_count.saturating_add(1);
        }
    }

    fn advance_readiness_generation(&mut self) {
        self.readiness_generation =
            NativeBitmapAtlasReadinessGeneration::next(self.readiness_generation);
    }

    fn record_readiness_change(&mut self, cache_key: GlyphRasterKey) {
        self.advance_readiness_generation();
        if self.pending_readiness_full_invalidation {
            return;
        }
        self.pending_readiness_changed_keys.extend([
            GlyphRasterKey {
                vertical_subpixel_bin: 0,
                ..cache_key
            },
            GlyphRasterKey {
                vertical_subpixel_bin: 1,
                ..cache_key
            },
            GlyphRasterKey {
                vertical_subpixel_bin: 2,
                ..cache_key
            },
            GlyphRasterKey {
                vertical_subpixel_bin: 3,
                ..cache_key
            },
        ]);
    }

    fn record_full_readiness_invalidation(&mut self) {
        self.advance_readiness_generation();
        self.pending_readiness_full_invalidation = true;
        self.pending_readiness_changed_keys.clear();
    }
}

pub(super) fn native_bitmap_atlas_raster_key_for_content(
    request_key: GlyphRasterKey,
    content: SwashContent,
) -> Option<GlyphRasterKey> {
    let format = match content {
        SwashContent::Mask => GlyphAtlasFormat::AlphaMask,
        SwashContent::Color => GlyphAtlasFormat::Color,
        SwashContent::SubpixelMask => GlyphAtlasFormat::SubpixelMask,
    };
    let smoothing = match format {
        GlyphAtlasFormat::AlphaMask => GlyphSmoothingMode::Grayscale,
        GlyphAtlasFormat::Color => GlyphSmoothingMode::None,
        GlyphAtlasFormat::SubpixelMask => GlyphSmoothingMode::Subpixel,
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => return None,
    };
    Some(GlyphRasterKey {
        format,
        smoothing,
        ..request_key
    })
}

fn cached_glyph_image_from_worker_bitmap(
    bitmap: GlyphBitmap,
) -> Option<NativeBitmapAtlasCachedGlyphImage> {
    let content = swash_content_for_atlas_format(bitmap.atlas_format()?)?;
    Some(NativeBitmapAtlasCachedGlyphImage {
        content,
        top: worker_bitmap_bearing_to_i16(bitmap.bearing.y)?,
        left: worker_bitmap_bearing_to_i16(bitmap.bearing.x)?,
        width: u16::try_from(bitmap.size.x).ok()?,
        height: u16::try_from(bitmap.size.y).ok()?,
        bytes: Arc::from(bitmap.data),
    })
}

fn swash_content_for_atlas_format(format: GlyphAtlasFormat) -> Option<SwashContent> {
    match format {
        GlyphAtlasFormat::AlphaMask => Some(SwashContent::Mask),
        GlyphAtlasFormat::SubpixelMask => Some(SwashContent::SubpixelMask),
        GlyphAtlasFormat::Color => Some(SwashContent::Color),
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => None,
    }
}

fn approximate_vertical_bin_candidates(requested: u8) -> [u8; 3] {
    match requested.min(3) {
        0 => [1, 2, 3],
        1 => [0, 2, 3],
        2 => [1, 3, 0],
        _ => [2, 1, 0],
    }
}

fn worker_bitmap_bearing_to_i16(value: f32) -> Option<i16> {
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    if rounded < i16::MIN as f32 || rounded > i16::MAX as f32 {
        return None;
    }
    Some(rounded as i16)
}

#[cfg(test)]
impl NativeBitmapAtlasSourceCache {
    pub(crate) fn corrupt_lru_tail_for_test(&mut self, cache_key: GlyphRasterKey) {
        self.lru.corrupt_tail_for_test(cache_key);
    }

    pub(crate) fn insert_test_image(
        &mut self,
        cache_key: GlyphRasterKey,
        image: NativeBitmapAtlasCachedGlyphImage,
    ) {
        self.insert(cache_key, image);
    }

    pub(crate) fn cached_test_image(
        &mut self,
        cache_key: GlyphRasterKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        self.touch_entry(cache_key)
    }
}
