use std::collections::HashMap;
use std::sync::Arc;

use glyphon::cosmic_text::SubpixelBin;
use glyphon::{CacheKey, FontSystem, SwashContent};

use crate::text::atlas::GlyphAtlasFormat;
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkerPool, TextRasterWorkerRequestError,
};
use crate::text::raster::{GlyphBitmap, SwashRasterRequest};

mod lru;

use lru::{NativeBitmapAtlasSourceCacheEntry, NativeBitmapAtlasSourceLru};

const DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_CAPACITY: usize = 2048;
const DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;
const FIRST_NATIVE_BITMAP_ATLAS_WORK_ID: u64 = 1;
const FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID: u64 = 1;
pub(crate) const NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeBitmapAtlasSourceCacheFrameReport {
    pub(crate) capacity: usize,
    pub(crate) max_byte_count: usize,
    pub(crate) resident_byte_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) approximate_hit_count: usize,
    pub(crate) approximate_probe_count: usize,
    pub(crate) miss_count: usize,
    pub(crate) insert_count: usize,
    pub(crate) worker_request_submitted_count: usize,
    pub(crate) worker_request_pending_count: usize,
    pub(crate) worker_request_deferred_count: usize,
    pub(crate) worker_request_failed_count: usize,
    pub(crate) worker_request_backpressured_count: usize,
    pub(crate) worker_request_font_missing_count: usize,
    pub(crate) worker_request_font_copied_byte_count: usize,
    pub(crate) worker_request_unavailable_count: usize,
    pub(crate) worker_request_cancelled_count: usize,
    pub(crate) worker_completion_insert_count: usize,
    pub(crate) worker_completion_failed_count: usize,
    pub(crate) worker_completion_unknown_count: usize,
    pub(crate) worker_completion_invalid_bitmap_count: usize,
    pub(crate) worker_completion_face_invalidated_count: usize,
    pub(crate) worker_completion_applied_byte_count: usize,
    pub(crate) lru_repair_count: usize,
    pub(crate) evicted_count: usize,
    pub(crate) evicted_byte_count: usize,
    pub(crate) rejected_byte_budget_count: usize,
    pub(crate) invalidated_count: usize,
    pub(crate) entry_count: usize,
    pub(crate) pending_worker_count: usize,
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
    entries: HashMap<CacheKey, NativeBitmapAtlasSourceCacheEntry>,
    lru: NativeBitmapAtlasSourceLru,
    pending_worker_cache_keys: HashMap<TextRasterWorkId, CacheKey>,
    pending_worker_work_ids: HashMap<CacheKey, TextRasterWorkId>,
    // Reset together with face_epoch so an epoch only copies each backend font once.
    raster_font_bytes_by_backend: HashMap<glyphon::fontdb::ID, Arc<[u8]>>,
    // A stable ID lets each worker's Swash ScaleContext reuse parsed face state safely.
    raster_font_identity_by_backend: HashMap<glyphon::fontdb::ID, u64>,
    next_raster_font_id: u64,
    next_worker_id: u64,
    pending_face_invalidated_count: usize,
    pending_worker_cancelled_count: usize,
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
            entries: HashMap::new(),
            lru: NativeBitmapAtlasSourceLru::default(),
            pending_worker_cache_keys: HashMap::new(),
            pending_worker_work_ids: HashMap::new(),
            raster_font_bytes_by_backend: HashMap::new(),
            raster_font_identity_by_backend: HashMap::new(),
            next_raster_font_id: FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID,
            next_worker_id: FIRST_NATIVE_BITMAP_ATLAS_WORK_ID,
            pending_face_invalidated_count: 0,
            pending_worker_cancelled_count: 0,
            frame_report: NativeBitmapAtlasSourceCacheFrameReport::default(),
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        let invalidated_count = self.pending_face_invalidated_count;
        self.pending_face_invalidated_count = 0;
        let cancelled_count = self.pending_worker_cancelled_count;
        self.pending_worker_cancelled_count = 0;
        self.frame_report = NativeBitmapAtlasSourceCacheFrameReport {
            capacity: self.capacity,
            max_byte_count: self.max_byte_count,
            resident_byte_count: self.resident_byte_count,
            evicted_count: invalidated_count,
            invalidated_count,
            worker_request_cancelled_count: cancelled_count,
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
            pending_worker_count: self.pending_worker_cache_keys.len(),
            ..self.frame_report
        }
    }

    pub(crate) fn face_epoch(&self) -> u64 {
        self.face_epoch
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
        self.cancel_pending_worker_requests(worker_pool);
        self.raster_font_bytes_by_backend.clear();
        self.raster_font_identity_by_backend.clear();
        self.next_raster_font_id = FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID;
        self.face_epoch = self.face_epoch.saturating_add(1);
        self.pending_face_invalidated_count = self
            .pending_face_invalidated_count
            .saturating_add(invalidated_count);
    }

    pub(crate) fn register_worker_request(
        &mut self,
        work_id: TextRasterWorkId,
        cache_key: CacheKey,
    ) {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
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
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        if let Some(image) = self.touch_entry(cache_key) {
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return Some(image);
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        None
    }

    pub(crate) fn approximate_cached_image(
        &mut self,
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        for y_bin in approximate_vertical_bin_candidates(cache_key.y_bin) {
            let candidate_key = CacheKey { y_bin, ..cache_key };
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

    pub(crate) fn request_worker_image(
        &mut self,
        font_system: &mut FontSystem,
        font_database: &FontDatabase,
        worker_pool: Option<&TextRasterWorkerPool>,
        face_epoch: u64,
        cache_key: CacheKey,
    ) -> NativeBitmapAtlasWorkerRequestStatus {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        if self.entries.contains_key(&cache_key) {
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        }
        if self.pending_worker_work_ids.contains_key(&cache_key) {
            self.frame_report.worker_request_pending_count = self
                .frame_report
                .worker_request_pending_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Pending;
        }

        let Some(worker_pool) = worker_pool else {
            self.frame_report.worker_request_unavailable_count = self
                .frame_report
                .worker_request_unavailable_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };

        if self.frame_report.worker_request_submitted_count
            >= NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME
        {
            self.frame_report.worker_request_deferred_count = self
                .frame_report
                .worker_request_deferred_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::DeferredByFrameBudget;
        }

        let Some(face_index) = font_system
            .db()
            .face(cache_key.font_id)
            .map(|face| face.index as usize)
        else {
            self.frame_report.worker_request_font_missing_count = self
                .frame_report
                .worker_request_font_missing_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };
        let Some(font) = font_system.get_font(cache_key.font_id, cache_key.font_weight) else {
            self.frame_report.worker_request_font_missing_count = self
                .frame_report
                .worker_request_font_missing_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };

        let font_bytes = match self.raster_font_bytes_by_backend.get(&cache_key.font_id) {
            Some(bytes) => Arc::clone(bytes),
            None => {
                let bytes = Arc::<[u8]>::from(font.data());
                self.frame_report.worker_request_font_copied_byte_count = self
                    .frame_report
                    .worker_request_font_copied_byte_count
                    .saturating_add(bytes.len());
                self.raster_font_bytes_by_backend
                    .insert(cache_key.font_id, Arc::clone(&bytes));
                bytes
            }
        };

        let work_id = self.next_worker_id();
        let request = SwashRasterRequest::glyphon_cache_key(face_index, cache_key)
            .with_font_identity(self.raster_font_identity(cache_key.font_id, face_epoch));
        let request = font_database
            .font_face_id(cache_key.font_id)
            .and_then(|face| {
                font_database
                    .effective_instance_variations_shared(face, None, cache_key.font_weight.0)
                    .ok()
            })
            .map(|variations| request.clone().with_variations(variations))
            .unwrap_or(request);
        let work = TextRasterWorkItem::new(work_id, face_epoch, font_bytes, request);

        match worker_pool.try_request(work) {
            Ok(()) => {}
            Err(TextRasterWorkerRequestError::QueueFull(_)) => {
                self.frame_report.worker_request_backpressured_count = self
                    .frame_report
                    .worker_request_backpressured_count
                    .saturating_add(1);
                return NativeBitmapAtlasWorkerRequestStatus::DeferredByWorkerBackpressure;
            }
            Err(
                TextRasterWorkerRequestError::ChannelClosed(_)
                | TextRasterWorkerRequestError::DuplicateInFlight(_),
            ) => {
                self.frame_report.worker_request_failed_count = self
                    .frame_report
                    .worker_request_failed_count
                    .saturating_add(1);
                return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
            }
        }

        self.pending_worker_cache_keys.insert(work_id, cache_key);
        self.pending_worker_work_ids.insert(cache_key, work_id);
        self.frame_report.worker_request_submitted_count = self
            .frame_report
            .worker_request_submitted_count
            .saturating_add(1);
        NativeBitmapAtlasWorkerRequestStatus::Submitted(work_id)
    }

    pub(crate) fn apply_worker_completion_drain(
        &mut self,
        drain: TextRasterCompletionDrain,
    ) -> NativeBitmapAtlasSourceCacheFrameReport {
        let TextRasterCompletionDrain {
            accepted,
            face_invalidated_ids,
            face_invalidated_count,
        } = drain;

        self.frame_report.worker_completion_face_invalidated_count = self
            .frame_report
            .worker_completion_face_invalidated_count
            .saturating_add(face_invalidated_count);

        for work_id in face_invalidated_ids {
            self.remove_pending_worker_request(work_id);
        }
        for result in accepted {
            self.apply_worker_result(result);
        }

        self.frame_report()
    }

    fn apply_worker_result(&mut self, result: TextRasterWorkResult) {
        let Some(cache_key) = self.remove_pending_worker_request(result.id) else {
            self.frame_report.worker_completion_unknown_count = self
                .frame_report
                .worker_completion_unknown_count
                .saturating_add(1);
            return;
        };

        let Ok(bitmap) = result.result else {
            self.frame_report.worker_completion_failed_count = self
                .frame_report
                .worker_completion_failed_count
                .saturating_add(1);
            return;
        };

        let Some(image) = cached_glyph_image_from_worker_bitmap(bitmap) else {
            self.frame_report.worker_completion_invalid_bitmap_count = self
                .frame_report
                .worker_completion_invalid_bitmap_count
                .saturating_add(1);
            return;
        };

        self.frame_report.worker_completion_applied_byte_count = self
            .frame_report
            .worker_completion_applied_byte_count
            .saturating_add(image.bytes.len());

        if self.insert(cache_key, image) {
            self.frame_report.worker_completion_insert_count = self
                .frame_report
                .worker_completion_insert_count
                .saturating_add(1);
        }
    }

    fn insert(&mut self, cache_key: CacheKey, image: NativeBitmapAtlasCachedGlyphImage) -> bool {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
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
            let existing_byte_count = existing.image.bytes.len();
            self.resident_byte_count = self.resident_byte_count.saturating_sub(existing_byte_count);
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
        true
    }

    fn evict_least_recently_used(&mut self) -> bool {
        let (entry, repaired) = self.lru.pop_least_recent(&mut self.entries);
        self.record_lru_repair(repaired);
        if let Some(entry) = entry {
            let byte_count = entry.image.bytes.len();
            self.resident_byte_count = self.resident_byte_count.saturating_sub(byte_count);
            self.frame_report.evicted_count = self.frame_report.evicted_count.saturating_add(1);
            self.frame_report.evicted_byte_count = self
                .frame_report
                .evicted_byte_count
                .saturating_add(byte_count);
            true
        } else {
            false
        }
    }

    fn touch_entry(&mut self, cache_key: CacheKey) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let (image, repaired) = self.lru.touch(&mut self.entries, cache_key);
        self.record_lru_repair(repaired);
        image
    }

    fn record_lru_repair(&mut self, repaired: bool) {
        if repaired {
            self.frame_report.lru_repair_count =
                self.frame_report.lru_repair_count.saturating_add(1);
        }
    }

    fn cancel_pending_worker_requests(&mut self, worker_pool: Option<&TextRasterWorkerPool>) {
        let cancelled_count = worker_pool
            .map(|worker_pool| {
                self.pending_worker_cache_keys
                    .keys()
                    .filter(|work_id| worker_pool.cancel(**work_id))
                    .count()
            })
            .unwrap_or(0);
        self.pending_worker_cache_keys.clear();
        self.pending_worker_work_ids.clear();
        self.pending_worker_cancelled_count = self
            .pending_worker_cancelled_count
            .saturating_add(cancelled_count);
    }

    fn remove_pending_worker_request(&mut self, work_id: TextRasterWorkId) -> Option<CacheKey> {
        let cache_key = self.pending_worker_cache_keys.remove(&work_id)?;
        self.pending_worker_work_ids.remove(&cache_key);
        Some(cache_key)
    }

    fn next_worker_id(&mut self) -> TextRasterWorkId {
        let work_id = self.next_worker_id;
        self.next_worker_id = self
            .next_worker_id
            .saturating_add(1)
            .max(FIRST_NATIVE_BITMAP_ATLAS_WORK_ID);
        TextRasterWorkId::new(work_id)
    }

    fn raster_font_identity(&mut self, font_id: glyphon::fontdb::ID, face_epoch: u64) -> [u64; 2] {
        let next_raster_font_id = &mut self.next_raster_font_id;
        let raster_font_id = *self
            .raster_font_identity_by_backend
            .entry(font_id)
            .or_insert_with(|| {
                let raster_font_id = *next_raster_font_id;
                *next_raster_font_id = next_raster_font_id
                    .saturating_add(1)
                    .max(FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID);
                raster_font_id
            });
        [face_epoch, raster_font_id]
    }
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

fn approximate_vertical_bin_candidates(requested: SubpixelBin) -> [SubpixelBin; 3] {
    match requested {
        SubpixelBin::Zero => [SubpixelBin::One, SubpixelBin::Two, SubpixelBin::Three],
        SubpixelBin::One => [SubpixelBin::Zero, SubpixelBin::Two, SubpixelBin::Three],
        SubpixelBin::Two => [SubpixelBin::One, SubpixelBin::Three, SubpixelBin::Zero],
        SubpixelBin::Three => [SubpixelBin::Two, SubpixelBin::One, SubpixelBin::Zero],
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

pub(super) fn native_bitmap_atlas_stable_raster_cache_key(mut cache_key: CacheKey) -> CacheKey {
    cache_key.x_bin = SubpixelBin::Zero;
    cache_key
}

#[cfg(test)]
impl NativeBitmapAtlasSourceCache {
    pub(crate) fn corrupt_lru_tail_for_test(&mut self, cache_key: CacheKey) {
        self.lru.corrupt_tail_for_test(cache_key);
    }

    pub(crate) fn insert_test_image(
        &mut self,
        cache_key: CacheKey,
        image: NativeBitmapAtlasCachedGlyphImage,
    ) {
        self.insert(cache_key, image);
    }

    pub(crate) fn cached_test_image(
        &mut self,
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        self.touch_entry(cache_key)
    }
}
