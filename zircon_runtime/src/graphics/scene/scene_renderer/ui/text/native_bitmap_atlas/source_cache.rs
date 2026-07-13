use std::collections::HashMap;
use std::sync::Arc;

use glyphon::cosmic_text::SubpixelBin;
use glyphon::{CacheKey, FontSystem, SwashContent};

use crate::graphics::text::atlas::GlyphAtlasFormat;
use crate::graphics::text::font::FontDatabase;
use crate::graphics::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkTarget, TextRasterWorkerPool,
};
use crate::graphics::text::raster::{GlyphBitmap, SwashRasterRequest};

const DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_CAPACITY: usize = 2048;
const FIRST_NATIVE_BITMAP_ATLAS_WORK_ID: u64 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct NativeBitmapAtlasSourceCacheFrameReport {
    pub(in crate::graphics::scene::scene_renderer::ui) hit_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) approximate_hit_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) miss_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) insert_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_request_submitted_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_request_pending_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_request_failed_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_request_font_missing_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_request_unavailable_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_insert_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_failed_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_unknown_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_invalid_bitmap_count:
        usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_stale_page_generation_count:
        usize,
    pub(in crate::graphics::scene::scene_renderer::ui) worker_completion_face_invalidated_count:
        usize,
    pub(in crate::graphics::scene::scene_renderer::ui) evicted_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) invalidated_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) entry_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) pending_worker_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NativeBitmapAtlasCachedGlyphImage {
    pub(super) content: SwashContent,
    pub(super) top: i16,
    pub(super) left: i16,
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) bytes: Vec<u8>,
}

#[derive(Debug)]
pub(in crate::graphics::scene::scene_renderer::ui) struct NativeBitmapAtlasSourceCache {
    capacity: usize,
    tick: u64,
    face_epoch: u64,
    entries: HashMap<CacheKey, NativeBitmapAtlasSourceCacheEntry>,
    pending_worker_cache_keys: HashMap<TextRasterWorkId, CacheKey>,
    pending_worker_work_ids: HashMap<CacheKey, TextRasterWorkId>,
    next_worker_id: u64,
    pending_face_invalidated_count: usize,
    frame_report: NativeBitmapAtlasSourceCacheFrameReport,
}

#[derive(Clone, Debug)]
struct NativeBitmapAtlasSourceCacheEntry {
    image: NativeBitmapAtlasCachedGlyphImage,
    last_used_tick: u64,
}

impl Default for NativeBitmapAtlasSourceCache {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_NATIVE_BITMAP_ATLAS_SOURCE_CACHE_CAPACITY)
    }
}

impl NativeBitmapAtlasSourceCache {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            tick: 0,
            face_epoch: 0,
            entries: HashMap::new(),
            pending_worker_cache_keys: HashMap::new(),
            pending_worker_work_ids: HashMap::new(),
            next_worker_id: FIRST_NATIVE_BITMAP_ATLAS_WORK_ID,
            pending_face_invalidated_count: 0,
            frame_report: NativeBitmapAtlasSourceCacheFrameReport::default(),
        }
    }

    pub(super) fn begin_frame(&mut self) {
        let invalidated_count = self.pending_face_invalidated_count;
        self.pending_face_invalidated_count = 0;
        self.frame_report = NativeBitmapAtlasSourceCacheFrameReport {
            evicted_count: invalidated_count,
            invalidated_count,
            entry_count: self.entries.len(),
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        };
    }

    pub(super) fn frame_report(&self) -> NativeBitmapAtlasSourceCacheFrameReport {
        NativeBitmapAtlasSourceCacheFrameReport {
            entry_count: self.entries.len(),
            pending_worker_count: self.pending_worker_cache_keys.len(),
            ..self.frame_report
        }
    }

    pub(super) fn face_epoch(&self) -> u64 {
        self.face_epoch
    }

    pub(super) fn discard_all_for_idle_frame(&mut self) -> NativeBitmapAtlasSourceCacheFrameReport {
        self.begin_frame();
        let evicted_count = self.entries.len();
        self.entries.clear();
        self.clear_pending_worker_requests();
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(evicted_count);
        self.frame_report()
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn discard_all_for_face_invalidation(
        &mut self,
    ) {
        let invalidated_count = self.entries.len();
        self.entries.clear();
        self.clear_pending_worker_requests();
        self.face_epoch = self.face_epoch.saturating_add(1);
        self.pending_face_invalidated_count = self
            .pending_face_invalidated_count
            .saturating_add(invalidated_count);
    }

    pub(super) fn register_worker_request(
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

    pub(super) fn cached_image(
        &mut self,
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        self.tick = self.tick.saturating_add(1);
        if let Some(entry) = self.entries.get_mut(&cache_key) {
            entry.last_used_tick = self.tick;
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return Some(entry.image.clone());
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        None
    }

    pub(super) fn approximate_cached_image(
        &mut self,
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        self.tick = self.tick.saturating_add(1);
        let requested_x = cache_key.x_bin.as_float();
        let requested_y = cache_key.y_bin.as_float();
        let Some((&candidate_key, _)) = self
            .entries
            .iter()
            .filter(|(candidate_key, _)| approximate_cache_key_matches(cache_key, **candidate_key))
            .min_by(|(left_key, _), (right_key, _)| {
                let left_distance =
                    approximate_cache_key_distance(requested_x, requested_y, **left_key);
                let right_distance =
                    approximate_cache_key_distance(requested_x, requested_y, **right_key);
                left_distance
                    .total_cmp(&right_distance)
                    .then_with(|| left_key.glyph_id.cmp(&right_key.glyph_id))
                    .then_with(|| left_key.font_size_bits.cmp(&right_key.font_size_bits))
            })
        else {
            return None;
        };
        let entry = self.entries.get_mut(&candidate_key)?;
        entry.last_used_tick = self.tick;
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        self.frame_report.approximate_hit_count =
            self.frame_report.approximate_hit_count.saturating_add(1);
        Some(entry.image.clone())
    }

    pub(super) fn request_worker_image(
        &mut self,
        font_system: &mut FontSystem,
        font_database: &FontDatabase,
        worker_pool: Option<&TextRasterWorkerPool>,
        target: TextRasterWorkTarget,
        cache_key: CacheKey,
    ) -> Option<TextRasterWorkId> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        if self.entries.contains_key(&cache_key) {
            return None;
        }
        if self.pending_worker_work_ids.contains_key(&cache_key) {
            self.frame_report.worker_request_pending_count = self
                .frame_report
                .worker_request_pending_count
                .saturating_add(1);
            return None;
        }

        let Some(worker_pool) = worker_pool else {
            self.frame_report.worker_request_unavailable_count = self
                .frame_report
                .worker_request_unavailable_count
                .saturating_add(1);
            return None;
        };

        let Some(face_index) = font_system
            .db()
            .face(cache_key.font_id)
            .map(|face| face.index as usize)
        else {
            self.frame_report.worker_request_font_missing_count = self
                .frame_report
                .worker_request_font_missing_count
                .saturating_add(1);
            return None;
        };
        let Some(font) = font_system.get_font(cache_key.font_id, cache_key.font_weight) else {
            self.frame_report.worker_request_font_missing_count = self
                .frame_report
                .worker_request_font_missing_count
                .saturating_add(1);
            return None;
        };

        let work_id = self.next_worker_id();
        let request = SwashRasterRequest::glyphon_cache_key(face_index, cache_key);
        let request = font_database
            .font_face_id(cache_key.font_id)
            .and_then(|face| {
                font_database
                    .effective_variations(face, cache_key.font_weight.0)
                    .ok()
            })
            .map(|variations| request.clone().with_variations(variations))
            .unwrap_or(request);
        let work =
            TextRasterWorkItem::new(work_id, target, Arc::<[u8]>::from(font.data()), request);

        if worker_pool.request(work).is_err() {
            self.frame_report.worker_request_failed_count = self
                .frame_report
                .worker_request_failed_count
                .saturating_add(1);
            return None;
        }

        self.pending_worker_cache_keys.insert(work_id, cache_key);
        self.pending_worker_work_ids.insert(cache_key, work_id);
        self.frame_report.worker_request_submitted_count = self
            .frame_report
            .worker_request_submitted_count
            .saturating_add(1);
        Some(work_id)
    }

    pub(super) fn has_pending_worker_request(&self, cache_key: CacheKey) -> bool {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        self.pending_worker_work_ids.contains_key(&cache_key)
    }

    pub(super) fn apply_worker_completion_drain(
        &mut self,
        drain: TextRasterCompletionDrain,
    ) -> NativeBitmapAtlasSourceCacheFrameReport {
        let TextRasterCompletionDrain {
            accepted,
            stale_page_generation_ids,
            face_invalidated_ids,
            stale_page_generation_count,
            face_invalidated_count,
        } = drain;

        self.frame_report
            .worker_completion_stale_page_generation_count = self
            .frame_report
            .worker_completion_stale_page_generation_count
            .saturating_add(stale_page_generation_count);
        self.frame_report.worker_completion_face_invalidated_count = self
            .frame_report
            .worker_completion_face_invalidated_count
            .saturating_add(face_invalidated_count);

        for work_id in stale_page_generation_ids {
            self.remove_pending_worker_request(work_id);
        }
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

        self.tick = self.tick.saturating_add(1);
        if self.insert(cache_key, image) {
            self.frame_report.worker_completion_insert_count = self
                .frame_report
                .worker_completion_insert_count
                .saturating_add(1);
        }
    }

    fn insert(&mut self, cache_key: CacheKey, image: NativeBitmapAtlasCachedGlyphImage) -> bool {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        if self.capacity == 0 {
            return false;
        }
        if !self.entries.contains_key(&cache_key) && self.entries.len() >= self.capacity {
            self.evict_least_recently_used();
        }
        self.entries.insert(
            cache_key,
            NativeBitmapAtlasSourceCacheEntry {
                image,
                last_used_tick: self.tick,
            },
        );
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        true
    }

    fn evict_least_recently_used(&mut self) {
        let Some((&cache_key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used_tick)
        else {
            return;
        };
        self.entries.remove(&cache_key);
        self.frame_report.evicted_count = self.frame_report.evicted_count.saturating_add(1);
    }

    fn clear_pending_worker_requests(&mut self) {
        self.pending_worker_cache_keys.clear();
        self.pending_worker_work_ids.clear();
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
        bytes: bitmap.data,
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

fn approximate_cache_key_matches(requested: CacheKey, candidate: CacheKey) -> bool {
    requested.font_id == candidate.font_id
        && requested.glyph_id == candidate.glyph_id
        && requested.font_size_bits == candidate.font_size_bits
        && requested.font_weight == candidate.font_weight
        && requested.flags == candidate.flags
        && (requested.x_bin != candidate.x_bin || requested.y_bin != candidate.y_bin)
}

fn approximate_cache_key_distance(requested_x: f32, requested_y: f32, candidate: CacheKey) -> f32 {
    let dx = requested_x - candidate.x_bin.as_float();
    let dy = requested_y - candidate.y_bin.as_float();
    dx.mul_add(dx, dy * dy)
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

fn native_bitmap_atlas_stable_raster_cache_key(mut cache_key: CacheKey) -> CacheKey {
    cache_key.x_bin = SubpixelBin::Zero;
    cache_key
}

#[cfg(test)]
impl NativeBitmapAtlasSourceCache {
    pub(super) fn insert_test_image(
        &mut self,
        cache_key: CacheKey,
        image: NativeBitmapAtlasCachedGlyphImage,
    ) {
        self.tick = self.tick.saturating_add(1);
        self.insert(cache_key, image);
    }

    pub(super) fn cached_test_image(
        &mut self,
        cache_key: CacheKey,
    ) -> Option<NativeBitmapAtlasCachedGlyphImage> {
        let cache_key = native_bitmap_atlas_stable_raster_cache_key(cache_key);
        self.tick = self.tick.saturating_add(1);
        let entry = self.entries.get_mut(&cache_key)?;
        entry.last_used_tick = self.tick;
        Some(entry.image.clone())
    }
}
