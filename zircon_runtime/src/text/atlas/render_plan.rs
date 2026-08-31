use crate::core::math::UVec2;

use super::render_contract::GlyphAtlasRenderContract;
use super::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect, GlyphRasterPlacement, GlyphSmoothingMode,
};

const COLOR_CHANNEL_MIN: f32 = 0.0;
const COLOR_CHANNEL_MAX: f32 = 1.0;
const DEFAULT_COLOR_CHANNEL: f32 = 0.0;
const OPAQUE_BACKGROUND_ALPHA: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasScreenRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl GlyphAtlasScreenRect {
    pub(crate) fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) fn from_raster_placement(
        placement: GlyphRasterPlacement,
        y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        Self::new(placement.snapped_x, y, width, height)
    }

    fn right(self) -> f32 {
        self.x + self.width
    }

    fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn is_drawable(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.right().is_finite()
            && self.bottom().is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }

    pub(crate) fn clipped_to(self, clip: Self) -> Option<Self> {
        if !self.is_drawable() || !clip.is_drawable() {
            return None;
        }

        let x0 = self.x.max(clip.x);
        let y0 = self.y.max(clip.y);
        let x1 = self.right().min(clip.right());
        let y1 = self.bottom().min(clip.bottom());
        let width = x1 - x0;
        let height = y1 - y0;
        let clipped = Self::new(x0, y0, width, height);
        clipped.is_drawable().then_some(clipped)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasUvRect {
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasDrawGlyph {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) atlas_size: UVec2,
    pub(crate) atlas_rect: GlyphAtlasRect,
    pub(crate) content_size: UVec2,
    pub(crate) screen_rect: GlyphAtlasScreenRect,
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
}

/// One glyph occurrence after clipping, before GPU instance packing.
///
/// This is the only CPU draw artifact: the shader expands it to two triangles from
/// `vertex_index`, so no per-glyph six-vertex intermediate is materialized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasDrawInstance {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) render_contract: GlyphAtlasRenderContract,
    pub(crate) screen_rect: GlyphAtlasScreenRect,
    pub(crate) uv_rect: GlyphAtlasUvRect,
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
}

pub(crate) fn glyph_atlas_draw_instance(
    glyph: GlyphAtlasDrawGlyph,
    clip_rect: GlyphAtlasScreenRect,
) -> Option<GlyphAtlasDrawInstance> {
    let clipped = glyph.screen_rect.clipped_to(clip_rect)?;
    let uv = glyph_atlas_content_uv_rect(glyph.atlas_rect, glyph.content_size, glyph.atlas_size)?;
    let left = (clipped.x - glyph.screen_rect.x) / glyph.screen_rect.width;
    let top = (clipped.y - glyph.screen_rect.y) / glyph.screen_rect.height;
    let right = (clipped.right() - glyph.screen_rect.x) / glyph.screen_rect.width;
    let bottom = (clipped.bottom() - glyph.screen_rect.y) / glyph.screen_rect.height;

    let uv_width = uv.x1 - uv.x0;
    let uv_height = uv.y1 - uv.y0;
    let uv0 = [uv.x0 + uv_width * left, uv.y0 + uv_height * top];
    let uv1 = [uv.x0 + uv_width * right, uv.y0 + uv_height * bottom];
    let contract = GlyphAtlasRenderContract::for_sampling_semantics(
        glyph.page_key.format.sampling_semantics(),
    );

    Some(GlyphAtlasDrawInstance {
        page_key: glyph.page_key,
        render_contract: contract,
        screen_rect: clipped,
        uv_rect: GlyphAtlasUvRect {
            x0: uv0[0],
            y0: uv0[1],
            x1: uv1[0],
            y1: uv1[1],
        },
        foreground_color: normalized_gpu_color(glyph.foreground_color),
        background_color: glyph_atlas_background_color_for_contract(
            glyph.background_color,
            contract,
        ),
    })
}

fn glyph_atlas_content_uv_rect(
    atlas_rect: GlyphAtlasRect,
    content_size: UVec2,
    atlas_size: UVec2,
) -> Option<GlyphAtlasUvRect> {
    if atlas_size.x == 0 || atlas_size.y == 0 || content_size.x == 0 || content_size.y == 0 {
        return None;
    }

    if content_size.x > atlas_rect.width || content_size.y > atlas_rect.height {
        return None;
    }
    let slot_right = atlas_rect.x.checked_add(atlas_rect.width)?;
    let slot_bottom = atlas_rect.y.checked_add(atlas_rect.height)?;
    if slot_right > atlas_size.x || slot_bottom > atlas_size.y {
        return None;
    }
    let content_right = atlas_rect.x.checked_add(content_size.x)?;
    let content_bottom = atlas_rect.y.checked_add(content_size.y)?;
    if content_right > atlas_size.x || content_bottom > atlas_size.y {
        return None;
    }
    let atlas_content_rect = GlyphAtlasRect {
        x: atlas_rect.x,
        y: atlas_rect.y,
        width: content_size.x,
        height: content_size.y,
    };

    let atlas_width = atlas_size.x as f32;
    let atlas_height = atlas_size.y as f32;
    Some(GlyphAtlasUvRect {
        x0: atlas_content_rect.x as f32 / atlas_width,
        y0: atlas_content_rect.y as f32 / atlas_height,
        x1: content_right as f32 / atlas_width,
        y1: content_bottom as f32 / atlas_height,
    })
}

fn glyph_atlas_background_color_for_contract(
    background_color: [f32; 4],
    render_contract: GlyphAtlasRenderContract,
) -> [f32; 4] {
    let mut background_color = normalized_gpu_color(background_color);
    if !render_contract.requires_background_composite() {
        return background_color;
    }

    background_color[3] = OPAQUE_BACKGROUND_ALPHA;
    background_color
}

fn normalized_gpu_color(color: [f32; 4]) -> [f32; 4] {
    color.map(normalized_gpu_color_channel)
}

fn normalized_gpu_color_channel(value: f32) -> f32 {
    if !value.is_finite() {
        return DEFAULT_COLOR_CHANNEL;
    }

    value.clamp(COLOR_CHANNEL_MIN, COLOR_CHANNEL_MAX)
}

#[cfg(test)]
mod tests;
