use super::render_contract::GlyphAtlasRenderContract;
use super::render_plan::{
    glyph_atlas_draw_quad, GlyphAtlasDrawGlyph, GlyphAtlasDrawQuad, GlyphAtlasScreenRect,
};
use super::GlyphAtlasPageKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasDrawBatchKey {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) render_contract: GlyphAtlasRenderContract,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasDrawBatch {
    pub(crate) key: GlyphAtlasDrawBatchKey,
    pub(crate) quads: Vec<GlyphAtlasDrawQuad>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasDrawBatchPlan {
    pub(crate) batches: Vec<GlyphAtlasDrawBatch>,
    pub(crate) visible_glyph_count: usize,
    pub(crate) skipped_glyph_count: usize,
    pub(crate) vertex_count: usize,
    pub(crate) requires_background_composite: bool,
}

pub(crate) fn glyph_atlas_draw_batch_plan<I>(
    glyphs: I,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasDrawBatchPlan
where
    I: IntoIterator<Item = GlyphAtlasDrawGlyph>,
{
    let mut plan = GlyphAtlasDrawBatchPlan::default();

    for glyph in glyphs {
        let Some(quad) = glyph_atlas_draw_quad(glyph, clip_rect) else {
            plan.skipped_glyph_count += 1;
            continue;
        };

        let key = GlyphAtlasDrawBatchKey {
            page_key: quad.page_key,
            render_contract: quad.render_contract,
        };
        plan.visible_glyph_count += 1;
        plan.vertex_count += quad.vertices.len();
        plan.requires_background_composite |= quad.render_contract.requires_background_composite();

        if let Some(batch) = plan.batches.iter_mut().find(|batch| batch.key == key) {
            batch.quads.push(quad);
        } else {
            plan.batches.push(GlyphAtlasDrawBatch {
                key,
                quads: vec![quad],
            });
        }
    }

    plan
}

#[cfg(test)]
mod tests;
