use super::super::sdf_upload::{SdfAtlasUploadMode, SdfAtlasUploadPageReport};
use super::super::text_pixel_snap::text_frame_device_origin;
use super::vertices::{
    aligned_text_start_x, build_sdf_vertices, horizontal_sdf_glyph_frame, pixel_to_ndc_x,
    pixel_to_ndc_y, resolve_sdf_glyph_advances, resolve_vertical_sdf_glyph_advances,
    sdf_screen_px_range, sdf_uv_at_destination, vertical_sdf_glyph_frame,
    vertical_shaped_sdf_glyph_frame, RunGlyph, SdfUvRect,
};
use super::*;
use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiShapedGlyph;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::{
    plan_sdf_atlas, SdfAtlasAllocationFailure, SdfAtlasAllocationFailureReason, SdfAtlasPlan,
    SdfAtlasRun,
};
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet, GlyphRasterPlacement,
    GlyphSmoothingMode,
};
use crate::text::font::{FontDatabase, SystemFontPolicy};
use crate::text::sdf::{
    scale_sdf_metrics_for_display, SdfAtlasBake, SdfAtlasBakeReport, SdfBakedGlyph,
    SdfFontBakeCache, SdfGlyphMetrics,
};
use crate::text::sdf::{SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot};
use crate::text::sdf::{SdfBakeParams, SdfMode};
use crate::text::shaping::vertical_glyph_rotation;
use crate::text::TextRenderState;
use crate::text::{ShapedGlyphRotation, VerticalMode};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap, UiTextWritingMode,
};

mod decoration_geometry;
mod draw_plan;
mod layout_placement;
mod material;
mod prepare_report;
mod product_framebuffer;
mod shader_contract;

fn synthetic_layered_plan(page_index: u32) -> SdfAtlasPlan {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index);
    let mut atlas_set = GlyphAtlasSet::default();
    for index in 0..=page_index {
        atlas_set = atlas_set.with_page(GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, index),
            UVec2::splat(64),
        ));
    }
    SdfAtlasPlan {
        atlas_size: UVec2::splat(64),
        atlas_set,
        slots: vec![SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: 'A',
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some("res://fonts/default.font.toml".to_string()),
                font_family: Some("Zircon Sans".to_string()),
                language: None,
                font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
                bake_params: SdfBakeParams::default(),
            },
            page_key,
            rect: SdfAtlasRect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
        }],
        runs: vec![SdfAtlasRun {
            glyph_slot_indices: vec![Some(0)],
            ..Default::default()
        }],
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

fn allocation_failure(
    glyph: char,
    reason: SdfAtlasAllocationFailureReason,
) -> SdfAtlasAllocationFailure {
    SdfAtlasAllocationFailure {
        key: SdfAtlasGlyphKey {
            glyph,
            glyph_id: None,
            font_id: None,
            font_instance_id: None,
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            language: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
            bake_params: SdfBakeParams::default(),
        },
        reason,
        requested_size: UVec2::splat(64),
        atlas_size: UVec2::splat(64),
    }
}

fn synthetic_layered_bake(plan: &SdfAtlasPlan) -> SdfAtlasBake {
    SdfAtlasBake {
        pixels: vec![0; plan.atlas_size.x as usize * plan.atlas_size.y as usize * 2],
        pages: Vec::new(),
        glyphs: vec![SdfBakedGlyph {
            metrics: SdfGlyphMetrics {
                bitmap_width: 16,
                bitmap_height: 16,
                bitmap_left: 0.0,
                bitmap_bottom: 0.0,
                advance: 16.0,
                ascent: 16.0,
            },
            visible: true,
        }],
        generation_failures: Vec::new(),
        report: SdfAtlasBakeReport {
            slot_count: 1,
            visible_glyph_count: 1,
            empty_glyph_count: 0,
            atlas_byte_len: plan.atlas_size.x as usize * plan.atlas_size.y as usize * 2,
            nonzero_pixel_count: 0,
            resident_font_count: 0,
            loaded_font_count: 0,
            generation_failure_count: 0,
            r8_byte_len: plan.atlas_size.x as usize * plan.atlas_size.y as usize * 2,
            rgba_byte_len: 0,
            offline_glyph_count: 0,
            dynamic_glyph_count: 0,
        },
    }
}

fn bake_atlas(
    plan: &SdfAtlasPlan,
) -> (
    SdfFontBakeCache,
    FontDatabase,
    ProjectAssetManager,
    SdfAtlasBake,
) {
    let mut font_bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let atlas_bake = font_bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );
    (font_bake, font_database, asset_manager, atlas_bake)
}

fn text_advance(
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
    text: &ScreenSpaceUiTextBatch,
) -> f32 {
    text.text
        .chars()
        .map(|glyph| {
            font_bake
                .measure_glyph(
                    glyph,
                    text.font.as_deref(),
                    text.font_family.as_deref(),
                    text.language.as_deref(),
                    text.font_weight,
                    text.font_size,
                    font_database,
                    asset_manager,
                )
                .advance
        })
        .sum()
}

fn first_sdf_screen_px_range(text: ScreenSpaceUiTextBatch) -> f32 {
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (_, _, asset_manager, atlas_bake) = bake_atlas(&plan);
    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &asset_manager,
        UVec2::new(128, 64),
    );
    vertices
        .first()
        .map(|vertex| vertex.screen_px_range)
        .expect("visible glyph should emit an SDF vertex")
}

fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame,
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        layout_error: None,
        color: [0.2, 0.3, 0.4, 0.5],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
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
