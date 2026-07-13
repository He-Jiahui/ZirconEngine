use std::collections::HashMap;

use super::*;
use crate::core::framework::render::{FontFaceId, InstancedFaceId, ShapedGlyphRotation};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::graphics::text::atlas::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet,
    GlyphAtlasStorageFormat, GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};
use crate::graphics::text::sdf::{SdfBakeParams, SdfMode};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
};

mod allocation;
mod cache_report;
mod owner;
mod plan;

fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame,
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 12.0,
        line_height: 14.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: SdfMode::Sdf,
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}

fn glyph_slots(indices: &[usize]) -> Vec<Option<usize>> {
    indices.iter().copied().map(Some).collect()
}

fn sdf_rect(x: u32, y: u32, width: u32, height: u32) -> SdfAtlasRect {
    SdfAtlasRect {
        x,
        y,
        width,
        height,
    }
}

fn dirty_pages(rects: &[SdfAtlasRect]) -> Vec<SdfAtlasDirtyPageReport> {
    rects
        .iter()
        .copied()
        .map(|dirty_rect| dirty_page(0, dirty_rect))
        .collect()
}

fn dirty_pages_for_indices(
    page_indices: &[u32],
    dirty_rect: SdfAtlasRect,
) -> Vec<SdfAtlasDirtyPageReport> {
    page_indices
        .iter()
        .copied()
        .map(|page_index| dirty_page(page_index, dirty_rect))
        .collect()
}

fn dirty_page(page_index: u32, dirty_rect: SdfAtlasRect) -> SdfAtlasDirtyPageReport {
    SdfAtlasDirtyPageReport {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index),
        dirty_rect,
    }
}

fn synthetic_plan(slots: Vec<SdfAtlasSlot>) -> SdfAtlasPlan {
    synthetic_plan_with_rebuilt_pages(slots, Vec::new())
}

fn synthetic_plan_with_rebuilt_pages(
    slots: Vec<SdfAtlasSlot>,
    rebuilt_pages: Vec<GlyphAtlasPageKey>,
) -> SdfAtlasPlan {
    SdfAtlasPlan {
        atlas_size: UVec2::splat(256),
        atlas_set: GlyphAtlasSet::default(),
        slots,
        runs: Vec::new(),
        rebuilt_pages,
        allocation_failures: Vec::new(),
    }
}

fn slot_on_page(glyph: char, page_index: u32, rect: SdfAtlasRect) -> SdfAtlasSlot {
    SdfAtlasSlot {
        key: glyph_key(glyph),
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index),
        rect,
    }
}

fn glyph_key(glyph: char) -> SdfAtlasGlyphKey {
    glyph_key_for_mode(glyph, SdfMode::Sdf)
}

fn glyph_key_for_mode(glyph: char, mode: SdfMode) -> SdfAtlasGlyphKey {
    SdfAtlasGlyphKey {
        glyph,
        glyph_id: None,
        font_id: None,
        font_instance_id: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        bake_params: SdfBakeParams::for_mode(mode),
    }
}

fn glyph_range_string(start: u32, count: usize) -> String {
    (0..count)
        .map(|index| char::from_u32(start + index as u32).unwrap())
        .collect()
}
