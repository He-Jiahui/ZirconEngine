use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::GlyphAtlasFormat;
use super::super::{GlyphAtlasBitmapPlaceholderGlyph, GlyphAtlasBitmapPlaceholderMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapPlaceholderDraw {
    pub(crate) source_index: usize,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) screen_rect: GlyphAtlasScreenRect,
    pub(crate) retry_frame_index: u64,
    pub(crate) mode: GlyphAtlasBitmapPlaceholderMode,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapPlaceholderDrawPlan {
    pub(crate) draws: Vec<GlyphAtlasBitmapPlaceholderDraw>,
    pub(crate) visible_placeholder_count: usize,
    pub(crate) skipped_placeholder_count: usize,
}

pub(crate) fn glyph_atlas_bitmap_placeholder_draw_plan<I>(
    placeholders: I,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapPlaceholderDrawPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapPlaceholderGlyph>,
{
    let mut plan = GlyphAtlasBitmapPlaceholderDrawPlan::default();

    for placeholder in placeholders {
        let Some(screen_rect) = placeholder.screen_rect.clipped_to(clip_rect) else {
            plan.skipped_placeholder_count += 1;
            continue;
        };

        plan.visible_placeholder_count += 1;
        plan.draws.push(glyph_atlas_bitmap_placeholder_draw(
            placeholder,
            screen_rect,
        ));
    }

    plan
}

fn glyph_atlas_bitmap_placeholder_draw(
    placeholder: GlyphAtlasBitmapPlaceholderGlyph,
    screen_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapPlaceholderDraw {
    GlyphAtlasBitmapPlaceholderDraw {
        source_index: placeholder.source_index,
        format: placeholder.format,
        screen_rect,
        retry_frame_index: placeholder.retry_frame_index,
        mode: placeholder.mode,
    }
}
