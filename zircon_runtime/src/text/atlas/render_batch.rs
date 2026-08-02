use super::render_contract::GlyphAtlasRenderContract;
use super::render_plan::{
    glyph_atlas_draw_instance, GlyphAtlasDrawGlyph, GlyphAtlasDrawInstance, GlyphAtlasScreenRect,
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
    pub(crate) instances: Vec<GlyphAtlasDrawInstance>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasDrawBatchPlan {
    pub(crate) batches: Vec<GlyphAtlasDrawBatch>,
    pub(crate) visible_glyph_count: usize,
    pub(crate) skipped_glyph_count: usize,
    pub(crate) instance_count: usize,
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
        let Some(instance) = glyph_atlas_draw_instance(glyph, clip_rect) else {
            plan.skipped_glyph_count += 1;
            continue;
        };

        let key = GlyphAtlasDrawBatchKey {
            page_key: instance.page_key,
            render_contract: instance.render_contract,
        };
        plan.visible_glyph_count += 1;
        plan.instance_count += 1;
        plan.requires_background_composite |=
            instance.render_contract.requires_background_composite();

        // Only merge adjacent compatible glyphs. Joining a later matching page/contract back
        // into an earlier batch would reorder Alpha/Color/Subpixel overlap in painter order.
        if let Some(batch) = plan.batches.last_mut().filter(|batch| batch.key == key) {
            batch.instances.push(instance);
        } else {
            plan.batches.push(GlyphAtlasDrawBatch {
                key,
                instances: vec![instance],
            });
        }
    }

    plan
}

#[cfg(test)]
mod tests;
