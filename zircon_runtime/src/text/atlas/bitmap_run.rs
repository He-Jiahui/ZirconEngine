use std::collections::{BTreeMap, HashSet};

use crate::core::math::UVec2;

use super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasSet, GlyphAtlasShelfAllocator};

mod allocation;
mod failure;
mod placeholder;
mod retry;
mod staged_upload;
mod staging;
mod types;
mod upload;
mod validation;

use allocation::{allocate_bitmap_source, mark_bitmap_dirty, record_bitmap_page_rebuild};
use failure::record_bitmap_allocation_failure;
use upload::{bitmap_upload_commands, bitmap_upload_copy};
use validation::validate_bitmap_source;

pub(crate) use failure::{
    GlyphAtlasBitmapAllocationFailure, GlyphAtlasBitmapAllocationFailureReason,
    GlyphAtlasBitmapQueuedGlyph,
};
pub(crate) use placeholder::{GlyphAtlasBitmapPlaceholderGlyph, GlyphAtlasBitmapPlaceholderMode};
pub(crate) use retry::{
    glyph_atlas_bitmap_retry_frame_input, glyph_atlas_bitmap_retry_frame_input_with_backpressure,
    glyph_atlas_bitmap_retry_frame_input_with_backpressure_and_new_source_budget_predicate,
    glyph_atlas_bitmap_retry_frame_outcome, glyph_atlas_bitmap_retry_plan,
    glyph_atlas_bitmap_retry_plan_with_backpressure, GlyphAtlasBitmapRetryBackpressurePolicy,
    GlyphAtlasBitmapRetryFrameInput, GlyphAtlasBitmapRetryFrameOutcome, GlyphAtlasBitmapRetryPlan,
    GlyphAtlasBitmapRetrySourceOrigin,
};
pub(crate) use staged_upload::{
    glyph_atlas_bitmap_page_shadow_commit, glyph_atlas_bitmap_prepared_upload_plan,
    glyph_atlas_bitmap_staged_upload_plan, glyph_atlas_bitmap_texture_upload_request_plan,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity,
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapRequeueReason, GlyphAtlasBitmapRequeuedUpload, GlyphAtlasBitmapStagedUpload,
    GlyphAtlasBitmapStagedUploadFailure, GlyphAtlasBitmapStagedUploadFailureReason,
    GlyphAtlasBitmapStagedUploadPlan, GlyphAtlasBitmapTextureUploadRequest,
    GlyphAtlasBitmapTextureUploadRequestPlan,
};
pub(crate) use staging::{
    glyph_atlas_bitmap_upload_staging_plan, GlyphAtlasBitmapPageUploadStaging,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasBitmapUploadStagingFailure,
    GlyphAtlasBitmapUploadStagingFailureReason, GlyphAtlasBitmapUploadStagingPlan,
};
pub(crate) use types::{
    GlyphAtlasBitmapGlyph, GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSlotInvalidation,
    GlyphAtlasBitmapSource, GlyphAtlasBitmapUploadCopy,
};

pub(crate) const GLYPH_BITMAP_ATLAS_PADDING_PX: u32 = 2;

pub(crate) fn glyph_atlas_bitmap_run_plan<I>(
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
) -> GlyphAtlasBitmapRunPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_run_plan_with_padding(
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        GLYPH_BITMAP_ATLAS_PADDING_PX,
    )
}

pub(crate) fn glyph_atlas_bitmap_run_plan_with_padding<I>(
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
) -> GlyphAtlasBitmapRunPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        GlyphAtlasSet::default(),
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
    )
}

pub(crate) fn glyph_atlas_bitmap_run_plan_with_atlas<I>(
    atlas: GlyphAtlasSet,
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
) -> GlyphAtlasBitmapRunPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        GLYPH_BITMAP_ATLAS_PADDING_PX,
    )
}

pub(crate) fn glyph_atlas_bitmap_run_plan_with_atlas_and_padding<I>(
    mut atlas: GlyphAtlasSet,
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
) -> GlyphAtlasBitmapRunPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let sources = sources.into_iter().collect::<Vec<_>>();
    let uses_persistent_slots = max_pages_per_format > 0
        && !sources.is_empty()
        && sources.iter().all(|source| {
            validate_bitmap_source(*source).is_ok()
                && source
                    .raster_key
                    .is_some_and(|key| key.format == source.format)
        });
    atlas.begin_frame();
    let mut plan = GlyphAtlasBitmapRunPlan {
        atlas,
        ..GlyphAtlasBitmapRunPlan::default()
    };
    let retained_slot_rects_by_page =
        uses_persistent_slots.then(|| plan.atlas.persistent_bitmap_slot_rects_by_page());
    let mut allocators = BTreeMap::<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>::new();
    let mut active_pages = BTreeMap::<GlyphAtlasFormat, GlyphAtlasPageKey>::new();
    let mut uploaded_raster_keys = HashSet::new();

    for (source_index, source) in sources.into_iter().enumerate() {
        if let Err(reason) = validate_bitmap_source(source) {
            record_bitmap_allocation_failure(&mut plan, source_index, source, reason, frame_index);
            continue;
        }

        let persistent_key = uses_persistent_slots.then_some(source.raster_key).flatten();
        let cached_slot = persistent_key.and_then(|key| {
            plan.atlas
                .persistent_bitmap_slot(key, source.content_size, page_size, frame_index)
        });
        let (allocation, requires_upload) = if let Some(slot) = cached_slot {
            plan.slot_cache_hit_count = plan.slot_cache_hit_count.saturating_add(1);
            (
                super::GlyphAtlasAllocation {
                    page_key: slot.page_key,
                    rect: slot.rect,
                },
                slot.inserted_frame_index == frame_index
                    && persistent_key.is_some_and(|key| uploaded_raster_keys.insert(key)),
            )
        } else if let Some(raster_key) = persistent_key {
            plan.slot_cache_miss_count = plan.slot_cache_miss_count.saturating_add(1);
            if source.content_size.x > page_size.x || source.content_size.y > page_size.y {
                record_bitmap_allocation_failure(
                    &mut plan,
                    source_index,
                    source,
                    GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph,
                    frame_index,
                );
                continue;
            }
            let Some((slot, decision)) = plan.atlas.allocate_persistent_bitmap_slot(
                raster_key,
                source.content_size,
                page_size,
                frame_index,
                max_pages_per_format,
                padding_px,
            ) else {
                record_bitmap_allocation_failure(
                    &mut plan,
                    source_index,
                    source,
                    GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked,
                    frame_index,
                );
                continue;
            };
            if matches!(
                decision,
                Some(super::GlyphAtlasPageResidencyDecision::Evict(_))
            ) {
                record_bitmap_page_rebuild(
                    &mut plan,
                    slot.page_key,
                    slot.page_generation,
                    page_size,
                );
                plan.zero_initialize_shadow_pages.insert(slot.page_key);
            }
            plan.slot_cache_insert_count = plan.slot_cache_insert_count.saturating_add(1);
            uploaded_raster_keys.insert(raster_key);
            (
                super::GlyphAtlasAllocation {
                    page_key: slot.page_key,
                    rect: slot.rect,
                },
                true,
            )
        } else {
            match allocate_bitmap_source(
                &mut plan,
                &mut allocators,
                &mut active_pages,
                source,
                page_size,
                frame_index,
                max_pages_per_format,
                padding_px,
            ) {
                Ok(allocation) => (allocation, true),
                Err(reason) => {
                    record_bitmap_allocation_failure(
                        &mut plan,
                        source_index,
                        source,
                        reason,
                        frame_index,
                    );
                    continue;
                }
            }
        };

        plan.atlas.mark_page_used(allocation.page_key, frame_index);
        if requires_upload {
            let retained_regions = retained_slot_rects_by_page
                .as_ref()
                .map(|pages| pages.get(&allocation.page_key).map_or(&[], Vec::as_slice));
            let has_replayable_shadow = plan.atlas.has_bitmap_page_shadow(allocation.page_key);
            let can_zero_initialize_shadow = uses_persistent_slots
                && !has_replayable_shadow
                && retained_regions.is_some_and(|regions| regions.is_empty());
            if can_zero_initialize_shadow {
                plan.zero_initialize_shadow_pages
                    .insert(allocation.page_key);
            }
            mark_bitmap_dirty(
                &mut plan.dirty_pages,
                allocation.page_key,
                allocation.rect,
                retained_regions,
                has_replayable_shadow || can_zero_initialize_shadow,
            );
            plan.upload_copies
                .push(bitmap_upload_copy(source_index, source, allocation));
        }

        let draw_glyph = GlyphAtlasDrawGlyph {
            page_key: allocation.page_key,
            atlas_size: page_size,
            atlas_rect: allocation.rect,
            content_size: source.content_size,
            screen_rect: source.screen_rect,
            foreground_color: source.foreground_color,
            background_color: source.background_color,
        };
        plan.draw_glyphs.push(draw_glyph);
        plan.glyphs.push(GlyphAtlasBitmapGlyph {
            source_index,
            page_key: allocation.page_key,
            atlas_rect: allocation.rect,
        });
    }

    plan.invalidated_raster_keys = plan.atlas.take_pending_invalidated_bitmap_raster_keys();
    plan.upload_commands = bitmap_upload_commands(&plan.atlas, &plan.dirty_pages);
    plan
}

#[cfg(test)]
mod tests;
