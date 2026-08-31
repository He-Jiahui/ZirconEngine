use std::sync::Arc;

use crate::core::math::UVec2;
use crate::text::sdf::{SdfGenerationSchedulerDiagnostics, SdfGlyphGenerationError};

use super::font_asset_cache::SdfFontAssetCacheReport;
use super::{SdfAtlasBake, SdfAtlasBakeReport, SdfAtlasSlot};

pub(super) struct SdfPreparedAtlasCache {
    atlas_size: UVec2,
    slots: Vec<SdfAtlasSlot>,
    bake: Option<SdfAtlasBake>,
    clean_dirty_pages: Arc<[super::SdfAtlasBakeDirtyPage]>,
}

impl Default for SdfPreparedAtlasCache {
    fn default() -> Self {
        Self {
            atlas_size: UVec2::default(),
            slots: Vec::new(),
            bake: None,
            clean_dirty_pages: Arc::from([]),
        }
    }
}

impl SdfPreparedAtlasCache {
    pub(super) fn cached_slots(&self) -> &[SdfAtlasSlot] {
        &self.slots
    }

    pub(super) fn reuse(
        &self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        scheduler: SdfGenerationSchedulerDiagnostics,
        font_asset_cache_report: SdfFontAssetCacheReport,
    ) -> Option<SdfAtlasBake> {
        let cached = self.bake.as_ref()?;
        if self.atlas_size != atlas_size
            || self.slots.as_slice() != slots
            || cached
                .generation_failures
                .iter()
                .any(|failure| generation_requires_retry(failure.error))
        {
            return None;
        }

        let mut reused = cached.clone();
        reused.dirty_pages = Arc::clone(&self.clean_dirty_pages);
        reused.report = stable_reuse_report(cached.report, scheduler, font_asset_cache_report);
        Some(reused)
    }

    pub(super) fn replace(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        bake: &SdfAtlasBake,
    ) {
        self.atlas_size = atlas_size;
        self.slots.clear();
        self.slots.extend_from_slice(slots);
        self.bake = Some(bake.clone());
    }
}

fn generation_requires_retry(error: SdfGlyphGenerationError) -> bool {
    matches!(
        error,
        SdfGlyphGenerationError::GenerationPending
            | SdfGlyphGenerationError::GenerationBudgetDeferred
    )
}

fn stable_reuse_report(
    mut report: SdfAtlasBakeReport,
    scheduler: SdfGenerationSchedulerDiagnostics,
    font_asset_cache_report: SdfFontAssetCacheReport,
) -> SdfAtlasBakeReport {
    report.loaded_font_count = 0;
    report.resident_font_asset_error_count = font_asset_cache_report.resident_error_count;
    report.resident_font_asset_no_registered_faces_count =
        font_asset_cache_report.resident_no_registered_faces_count;
    report.offline_manifest_parse_count = 0;
    report.offline_artifact_stat_count = 0;
    report.offline_artifact_read_count = 0;
    report.offline_artifact_read_byte_count = 0;
    report.offline_artifact_decode_count = 0;
    report.offline_pixel_copy_count = 0;
    report.offline_pixel_copy_byte_count = 0;
    report.offline_manifest_eviction_count = 0;
    report.offline_artifact_eviction_count = 0;
    report.offline_glyph_bitmap_eviction_count = 0;
    report.baked_glyph_eviction_count = 0;
    report.source_context_created_count = 0;
    report.source_context_eviction_count = 0;
    report.source_hash_count = 0;
    report.face_parse_count = 0;
    report.generation_batch_count = 0;
    report.generation_requested_glyph_count = 0;
    report.generation_unique_glyph_count = 0;
    report.generation_duplicate_glyph_count = 0;
    report.bitmap_clone_byte_count = 0;
    report.atlas_page_alloc_count = 0;
    report.atlas_page_zero_byte_count = 0;
    report.atlas_page_clear_count = 0;
    report.atlas_page_clear_byte_count = 0;
    report.atlas_page_write_count = 0;
    report.atlas_page_write_byte_count = 0;
    report.atlas_page_reused_slot_count = report.slot_count;
    report.atlas_full_page_scan_byte_count = 0;
    report.compiled_atlas_build_count = 0;
    report.compiled_atlas_reuse_count = 1;
    report.generation_scheduler = scheduler;
    report
}
