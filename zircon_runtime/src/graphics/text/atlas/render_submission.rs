mod placeholder;
mod plan;
mod report;

pub(crate) use placeholder::{
    glyph_atlas_bitmap_placeholder_draw_plan, GlyphAtlasBitmapPlaceholderDraw,
    GlyphAtlasBitmapPlaceholderDrawPlan,
};
pub(crate) use plan::{
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_padding, GlyphAtlasBitmapRenderSubmissionPlan,
};
pub(crate) use report::{
    glyph_atlas_bitmap_render_submission_report, GlyphAtlasBitmapRenderSubmissionReport,
};

#[cfg(test)]
mod tests;
