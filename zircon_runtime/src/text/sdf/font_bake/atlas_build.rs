use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::text::font::FontDatabase;
use crate::text::sdf::SdfGenerationScheduler;

use super::{
    RawBakedGlyphSource, SdfAtlasBake, SdfAtlasBakePage, SdfAtlasBakeReport,
    SdfAtlasGlyphGenerationFailure, SdfAtlasSlot, SdfBakedGlyph, SdfFontBakeCache,
};

impl SdfFontBakeCache {
    pub(crate) fn build_atlas_from_slots(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        self.build_atlas_from_slots_internal(
            atlas_size,
            slots,
            font_database,
            asset_manager,
            None,
            0,
        )
    }

    pub(crate) fn build_atlas_from_slots_scheduled(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
        scheduler: &SdfGenerationScheduler,
        frame_index: u64,
    ) -> SdfAtlasBake {
        self.build_atlas_from_slots_internal(
            atlas_size,
            slots,
            font_database,
            asset_manager,
            Some(scheduler),
            frame_index,
        )
    }

    fn build_atlas_from_slots_internal(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
        scheduler: Option<&SdfGenerationScheduler>,
        frame_index: u64,
    ) -> SdfAtlasBake {
        if let Some(scheduler) = scheduler {
            self.ensure_current_font_generation_scheduled(scheduler);
        } else {
            self.ensure_current_font_generation();
        }
        let reuse_scheduler_report = scheduler
            .map(|scheduler| scheduler.diagnostics(frame_index))
            .unwrap_or_default();
        if !self.async_generation.has_pending_work() {
            if let Some(bake) = self
                .prepared_atlas
                .reuse(atlas_size, slots, reuse_scheduler_report)
            {
                return bake;
            }
        }
        let slot_keys = slots
            .iter()
            .map(|slot| slot.key.clone())
            .collect::<Vec<_>>();
        self.prime_shaped_face_resolutions(&slot_keys, font_database);
        let previous_slots = self.prepared_atlas.cached_slots().to_vec();
        // A prepared-atlas hit bypasses glyph-cache lookups. Refresh the last visible
        // key set only when rebuilding so steady frames keep their zero-maintenance path.
        self.touch_cached_glyph_slots(&previous_slots);
        let resident_font_count_before = self.fonts.len();
        if let Some(scheduler) = scheduler {
            self.prepare_missing_glyphs_async(
                slots,
                font_database,
                asset_manager,
                scheduler,
                frame_index,
            );
        } else {
            self.prepare_missing_glyphs(slots, font_database, asset_manager);
        }
        self.enforce_baked_glyph_budget(slots);
        let mut glyphs = Vec::with_capacity(slots.len());
        let mut baked_glyphs = Vec::with_capacity(slots.len());
        let mut generation_failures = Vec::new();
        let mut offline_glyph_count = 0_usize;
        let mut dynamic_glyph_count = 0_usize;

        for (slot_index, slot) in slots.iter().enumerate() {
            let baked = self.bake_glyph_cached(&slot.key);
            if let Some(error) = baked.generation_error {
                generation_failures.push(SdfAtlasGlyphGenerationFailure {
                    slot_index,
                    key: slot.key.clone(),
                    error,
                });
            }
            match baked.source {
                RawBakedGlyphSource::Offline => {
                    offline_glyph_count = offline_glyph_count.saturating_add(1)
                }
                RawBakedGlyphSource::Dynamic => {
                    dynamic_glyph_count = dynamic_glyph_count.saturating_add(1)
                }
                RawBakedGlyphSource::Failed => {}
            }
            glyphs.push(SdfBakedGlyph {
                metrics: baked.metrics,
                visible: baked.visible,
            });
            baked_glyphs.push(baked);
        }

        let (pages, dirty_pages, atlas_page_report) =
            self.atlas_pages.update(atlas_size, slots, &baked_glyphs);

        let visible_glyph_count = glyphs.iter().filter(|glyph| glyph.visible).count();
        let resident_font_count = self.fonts.len();
        let source_context_report = self
            .source_contexts
            .report()
            .delta_since(self.reported_source_contexts);
        let dynamic_generation_report = self
            .dynamic_generation_totals
            .delta_since(self.reported_dynamic_generation);
        let offline_source_report = self
            .offline_source
            .report()
            .delta_since(self.reported_offline_source);
        let baked_glyph_cache_report = self.report_baked_glyph_cache();
        self.reported_source_contexts = self.source_contexts.report();
        self.reported_dynamic_generation = self.dynamic_generation_totals;
        self.reported_offline_source = self.offline_source.report();
        let report = SdfAtlasBakeReport {
            slot_count: glyphs.len(),
            visible_glyph_count,
            empty_glyph_count: glyphs.len().saturating_sub(visible_glyph_count),
            atlas_byte_len: atlas_page_report.atlas_byte_len,
            nonzero_pixel_count: atlas_page_report.nonzero_pixel_count,
            resident_font_count,
            loaded_font_count: resident_font_count.saturating_sub(resident_font_count_before),
            generation_failure_count: generation_failures.len(),
            r8_byte_len: page_byte_len(&pages, 1),
            rgba_byte_len: page_byte_len(&pages, 4),
            offline_glyph_count,
            dynamic_glyph_count,
            offline_resident_manifest_count: offline_source_report.resident_manifest_count,
            offline_resident_artifact_identity_count: offline_source_report
                .resident_artifact_identity_count,
            offline_resident_artifact_byte_count: offline_source_report
                .resident_artifact_byte_count,
            offline_resident_glyph_bitmap_count: offline_source_report.resident_glyph_bitmap_count,
            offline_resident_glyph_bitmap_byte_count: offline_source_report
                .resident_glyph_bitmap_byte_count,
            offline_manifest_parse_count: offline_source_report.manifest_parse_count,
            offline_artifact_stat_count: offline_source_report.artifact_stat_count,
            offline_artifact_read_count: offline_source_report.artifact_read_count,
            offline_artifact_read_byte_count: offline_source_report.artifact_read_byte_count,
            offline_artifact_decode_count: offline_source_report.artifact_decode_count,
            offline_pixel_copy_count: offline_source_report.pixel_copy_count,
            offline_pixel_copy_byte_count: offline_source_report.pixel_copy_byte_count,
            offline_manifest_eviction_count: offline_source_report.manifest_eviction_count,
            offline_artifact_eviction_count: offline_source_report.artifact_eviction_count,
            offline_glyph_bitmap_eviction_count: offline_source_report.glyph_bitmap_eviction_count,
            offline_oldest_artifact_idle_access_count: offline_source_report
                .oldest_artifact_idle_access_count,
            offline_oldest_glyph_bitmap_idle_access_count: offline_source_report
                .oldest_glyph_bitmap_idle_access_count,
            resident_baked_glyph_count: baked_glyph_cache_report.resident_count,
            resident_baked_glyph_byte_count: baked_glyph_cache_report.resident_byte_count,
            baked_glyph_eviction_count: baked_glyph_cache_report.eviction_count,
            oldest_baked_glyph_idle_access_count: baked_glyph_cache_report.oldest_idle_access_count,
            resident_source_context_count: source_context_report.resident_context_count,
            resident_source_byte_count: source_context_report.resident_source_byte_count,
            source_context_created_count: source_context_report.context_created_count,
            source_context_eviction_count: source_context_report.context_eviction_count,
            oldest_source_context_idle_access_count: source_context_report
                .oldest_context_idle_access_count,
            source_hash_count: source_context_report.source_hash_count,
            face_parse_count: source_context_report.face_parse_count,
            generation_batch_count: dynamic_generation_report.batch_count,
            generation_requested_glyph_count: dynamic_generation_report.requested_glyph_count,
            generation_unique_glyph_count: dynamic_generation_report.unique_glyph_count,
            generation_duplicate_glyph_count: dynamic_generation_report.duplicate_glyph_count,
            bitmap_clone_byte_count: 0,
            resident_atlas_page_count: atlas_page_report.resident_page_count,
            atlas_page_alloc_count: atlas_page_report.page_alloc_count,
            atlas_page_zero_byte_count: atlas_page_report.page_zero_byte_count,
            atlas_page_clear_count: atlas_page_report.page_clear_count,
            atlas_page_clear_byte_count: atlas_page_report.page_clear_byte_count,
            atlas_page_write_count: atlas_page_report.page_write_count,
            atlas_page_write_byte_count: atlas_page_report.page_write_byte_count,
            atlas_page_reused_slot_count: atlas_page_report.reused_slot_count,
            atlas_full_page_scan_byte_count: atlas_page_report.full_page_scan_byte_count,
            compiled_atlas_build_count: 1,
            compiled_atlas_reuse_count: 0,
            generation_scheduler: scheduler
                .map(|scheduler| scheduler.diagnostics(frame_index))
                .unwrap_or_default(),
        };

        let bake = SdfAtlasBake {
            pages: pages.into(),
            dirty_pages: dirty_pages.into(),
            glyphs: glyphs.into(),
            generation_failures: generation_failures.into(),
            report,
        };
        self.prepared_atlas.replace(atlas_size, slots, &bake);
        bake
    }
}

fn page_byte_len(pages: &[SdfAtlasBakePage], bytes_per_pixel: u32) -> usize {
    pages
        .iter()
        .filter(|page| page.page_key.format.storage_format().bytes_per_pixel() == bytes_per_pixel)
        .map(|page| page.byte_len)
        .sum()
}
