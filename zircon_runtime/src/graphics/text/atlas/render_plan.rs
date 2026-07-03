use crate::core::math::UVec2;

use super::render_contract::GlyphAtlasRenderContract;
use super::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect, GlyphRasterPlacement, GlyphSmoothingMode,
};

const COLOR_CHANNEL_MIN: f32 = 0.0;
const COLOR_CHANNEL_MAX: f32 = 1.0;
const DEFAULT_BACKGROUND_CHANNEL: f32 = 0.0;
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
        (width > 0.0 && height > 0.0).then(|| Self::new(x0, y0, width, height))
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasDrawQuad {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) render_contract: GlyphAtlasRenderContract,
    pub(crate) vertices: [GlyphAtlasDrawVertex; 6],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasDrawVertex {
    pub(crate) position_px: [f32; 2],
    pub(crate) uv: [f32; 2],
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
    pub(crate) page_index: u32,
}

pub(crate) fn glyph_atlas_draw_quad(
    glyph: GlyphAtlasDrawGlyph,
    clip_rect: GlyphAtlasScreenRect,
) -> Option<GlyphAtlasDrawQuad> {
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
    let x0 = clipped.x;
    let y0 = clipped.y;
    let x1 = clipped.right();
    let y1 = clipped.bottom();
    let contract = GlyphAtlasRenderContract::for_sampling_semantics(
        glyph.page_key.format.sampling_semantics(),
    );

    Some(GlyphAtlasDrawQuad {
        page_key: glyph.page_key,
        render_contract: contract,
        vertices: [
            glyph_atlas_draw_vertex(&glyph, contract, [x0, y0], [uv0[0], uv0[1]]),
            glyph_atlas_draw_vertex(&glyph, contract, [x1, y0], [uv1[0], uv0[1]]),
            glyph_atlas_draw_vertex(&glyph, contract, [x1, y1], [uv1[0], uv1[1]]),
            glyph_atlas_draw_vertex(&glyph, contract, [x0, y0], [uv0[0], uv0[1]]),
            glyph_atlas_draw_vertex(&glyph, contract, [x1, y1], [uv1[0], uv1[1]]),
            glyph_atlas_draw_vertex(&glyph, contract, [x0, y1], [uv0[0], uv1[1]]),
        ],
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

    let content_width = content_size.x.min(atlas_rect.width);
    let content_height = content_size.y.min(atlas_rect.height);
    if content_width == 0 || content_height == 0 {
        return None;
    }

    let atlas_width = atlas_size.x as f32;
    let atlas_height = atlas_size.y as f32;
    Some(GlyphAtlasUvRect {
        x0: atlas_rect.x as f32 / atlas_width,
        y0: atlas_rect.y as f32 / atlas_height,
        x1: atlas_rect.x.saturating_add(content_width) as f32 / atlas_width,
        y1: atlas_rect.y.saturating_add(content_height) as f32 / atlas_height,
    })
}

fn glyph_atlas_draw_vertex(
    glyph: &GlyphAtlasDrawGlyph,
    render_contract: GlyphAtlasRenderContract,
    position_px: [f32; 2],
    uv: [f32; 2],
) -> GlyphAtlasDrawVertex {
    GlyphAtlasDrawVertex {
        position_px,
        uv,
        foreground_color: glyph.foreground_color,
        background_color: glyph_atlas_background_color_for_contract(
            glyph.background_color,
            render_contract,
        ),
        page_index: glyph.page_key.page_index,
    }
}

fn glyph_atlas_background_color_for_contract(
    background_color: [f32; 4],
    render_contract: GlyphAtlasRenderContract,
) -> [f32; 4] {
    if !render_contract.requires_background_composite() {
        return background_color;
    }

    [
        normalized_background_channel(background_color[0]),
        normalized_background_channel(background_color[1]),
        normalized_background_channel(background_color[2]),
        OPAQUE_BACKGROUND_ALPHA,
    ]
}

fn normalized_background_channel(value: f32) -> f32 {
    if !value.is_finite() {
        return DEFAULT_BACKGROUND_CHANNEL;
    }

    value.clamp(COLOR_CHANNEL_MIN, COLOR_CHANNEL_MAX)
}

#[cfg(test)]
mod tests;
