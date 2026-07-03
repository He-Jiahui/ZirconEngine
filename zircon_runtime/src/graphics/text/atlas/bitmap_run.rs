use std::collections::BTreeMap;

use crate::core::math::UVec2;

use super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasShelfAllocator};

mod allocation;
mod failure;
mod placeholder;
mod types;
mod upload;
mod validation;

use allocation::{allocate_bitmap_source, mark_bitmap_dirty};
use failure::record_bitmap_allocation_failure;
use upload::bitmap_upload_commands;
use validation::validate_bitmap_source;

pub(crate) use failure::{
    GlyphAtlasBitmapAllocationFailure, GlyphAtlasBitmapAllocationFailureReason,
    GlyphAtlasBitmapQueuedGlyph,
};
pub(crate) use placeholder::{GlyphAtlasBitmapPlaceholderGlyph, GlyphAtlasBitmapPlaceholderMode};
pub(crate) use types::{GlyphAtlasBitmapGlyph, GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource};

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
    let mut plan = GlyphAtlasBitmapRunPlan::default();
    let mut allocators = BTreeMap::<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>::new();
    let mut active_pages = BTreeMap::<GlyphAtlasFormat, GlyphAtlasPageKey>::new();

    for (source_index, source) in sources.into_iter().enumerate() {
        if let Err(reason) = validate_bitmap_source(source) {
            record_bitmap_allocation_failure(&mut plan, source_index, source, reason, frame_index);
            continue;
        }

        let allocation = match allocate_bitmap_source(
            &mut plan,
            &mut allocators,
            &mut active_pages,
            source,
            page_size,
            frame_index,
            max_pages_per_format,
            padding_px,
        ) {
            Ok(allocation) => allocation,
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
        };

        plan.atlas.mark_page_used(allocation.page_key, frame_index);
        mark_bitmap_dirty(&mut plan.dirty_pages, allocation.page_key, allocation.rect);

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
            draw_glyph,
        });
    }

    plan.upload_commands = bitmap_upload_commands(&plan.atlas, &plan.dirty_pages);
    plan
}

#[cfg(test)]
mod tests;
