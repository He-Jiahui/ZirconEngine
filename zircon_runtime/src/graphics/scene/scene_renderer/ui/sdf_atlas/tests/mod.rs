use std::collections::HashMap;
use std::sync::Arc;

use super::*;
use crate::core::framework::text::{
    TextFontFaceHandle, TextGlyph, TextGlyphFlags, TextGlyphRotation,
};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
    ScreenSpaceUiTextRouteIdentity,
};
use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasPageSpec, GlyphAtlasSet, GlyphAtlasStorageFormat,
};
use crate::text::sdf::{SdfBakeParams, SdfMode};
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, ShapedGlyphRotation};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
};

mod allocation;
mod cache_report;
mod owner;
mod plan;

fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            "runtime.sdf-atlas.test",
            UiNodeId::new(1),
            None,
        ),
        command_generation: 1,
        text: text.to_string(),
        frame,
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: false,
        glyph_artifact_line: None,
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".into()),
        font_family: Some("Zircon Sans".into()),
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

fn artifact_text_batch(glyph_id: u32, writing_mode: UiTextWritingMode) -> ScreenSpaceUiTextBatch {
    let mut text = text_batch("fi", UiFrame::new(0.0, 0.0, 24.0, 24.0));
    text.writing_mode = writing_mode;
    text.glyph_advances = vec![24.0];
    text.glyph_artifact_line = Some(ScreenSpaceUiGlyphArtifactLine {
        artifact: Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from("fi"),
            source_text_origin: 0,
            font_generation: 7,
            style: UiResolvedStyle::default(),
            writing_mode,
            lines: vec![Some(ResolvedTextGlyphArtifactLine {
                glyphs: vec![TextGlyph {
                    glyph_id,
                    source_range: 0..2,
                    visual_range: 0..1,
                    advance: 24.0,
                    position: [0.0, 0.0],
                    offset: [0.0, 0.0],
                    font_face: None,
                    font_instance: None,
                    rotation: TextGlyphRotation::None,
                    bidi_level: 0,
                    flags: TextGlyphFlags::default(),
                    requires_rasterization: true,
                }],
                layout_line: UiResolvedTextLine {
                    text: "fi".to_string(),
                    frame: text.frame,
                    source_range: UiTextRange { start: 0, end: 2 },
                    visual_range: UiTextRange { start: 0, end: 1 },
                    measured_width: 24.0,
                    glyph_advances: vec![24.0],
                    baseline: 16.0,
                    direction: UiTextDirection::LeftToRight,
                    runs: Vec::new(),
                    ellipsized: false,
                },
            })],
        }),
        line_index: 0,
        refreshed_line: None,
        font_generation: 7,
    });
    text
}

fn refreshed_artifact_text_batch() -> (ScreenSpaceUiTextBatch, ScreenSpaceUiTextBatch) {
    let original = artifact_text_batch(0xfb01, UiTextWritingMode::HorizontalTb);
    let replacement = artifact_text_batch(0xfb02, UiTextWritingMode::HorizontalTb);
    let refreshed_line = replacement
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.artifact.lines.first())
        .and_then(Option::as_ref)
        .expect("replacement artifact line")
        .clone();
    let mut refreshed = original.clone();
    let artifact_line = refreshed
        .glyph_artifact_line
        .as_mut()
        .expect("original artifact line");
    assert!(Arc::ptr_eq(
        &artifact_line.artifact,
        &original
            .glyph_artifact_line
            .as_ref()
            .expect("original artifact line")
            .artifact
    ));
    artifact_line.refreshed_line = Some(Arc::new(refreshed_line));
    artifact_line.font_generation = 8;
    (original, refreshed)
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
        font: Some("res://fonts/default.font.toml".into()),
        font_family: Some("Zircon Sans".into()),
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
