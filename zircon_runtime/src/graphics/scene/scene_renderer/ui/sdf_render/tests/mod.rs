use super::super::sdf_upload::{SdfAtlasUploadMode, SdfAtlasUploadPageReport};
use super::super::text_pixel_snap::text_frame_device_origin;
use super::vertices::{
    RunGlyph, SDF_TEXT_PRIMITIVE_GLYPH, SdfUvRect, aligned_text_start_x, build_sdf_vertices,
    horizontal_sdf_glyph_frame, horizontal_shaped_sdf_glyph_frame, pixel_to_ndc_x, pixel_to_ndc_y,
    resolve_sdf_glyph_advances, resolve_vertical_sdf_glyph_advances, sdf_screen_px_range,
    sdf_uv_at_destination, vertical_sdf_glyph_frame, vertical_shaped_sdf_glyph_frame,
};
use super::*;
use crate::asset::ProjectAssetManager;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiGlyphArtifactLine;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::{
    SdfAtlasAllocationFailure, SdfAtlasAllocationFailureReason, SdfAtlasPlan, SdfAtlasRun,
    plan_sdf_atlas,
};
use crate::text::TextRenderState;
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet, GlyphRasterPlacement,
    GlyphSmoothingMode,
};
use crate::text::font::{FontDatabase, SystemFontPolicy};
use crate::text::sdf::{
    SdfAtlasBake, SdfAtlasBakeReport, SdfBakedGlyph, SdfFontBakeCache, SdfGlyphMetrics,
    scale_sdf_metrics_for_display,
};
use crate::text::sdf::{SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot};
use crate::text::sdf::{SdfBakeParams, SdfMode};
use crate::text::shaping::vertical_glyph_rotation;
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, VerticalMode};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
    UiTextWritingMode,
};

mod compiled_frame;
mod decoration_geometry;
mod draw_plan;
mod layout_placement;
mod material;
mod prepare_report;
mod product_framebuffer;
mod shader_contract;
mod shaped_advances;

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
                font: Some("res://fonts/default.font.toml".into()),
                font_family: Some("Zircon Sans".into()),
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
            font: Some("res://fonts/default.font.toml".into()),
            font_family: Some("Zircon Sans".into()),
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
        pages: Vec::new().into(),
        dirty_pages: Vec::new().into(),
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
        }]
        .into(),
        generation_failures: Vec::new().into(),
        report: SdfAtlasBakeReport {
            slot_count: 1,
            visible_glyph_count: 1,
            empty_glyph_count: 0,
            atlas_byte_len: plan.atlas_size.x as usize * plan.atlas_size.y as usize * 2,
            nonzero_pixel_count: 0,
            resident_font_count: 0,
            loaded_font_count: 0,
            generation_failure_count: 0,
            resident_font_asset_error_count: 0,
            resident_font_asset_no_registered_faces_count: 0,
            r8_byte_len: plan.atlas_size.x as usize * plan.atlas_size.y as usize * 2,
            rgba_byte_len: 0,
            offline_glyph_count: 0,
            dynamic_glyph_count: 0,
            offline_resident_manifest_count: 0,
            offline_resident_artifact_identity_count: 0,
            offline_resident_artifact_byte_count: 0,
            offline_resident_glyph_bitmap_count: 0,
            offline_resident_glyph_bitmap_byte_count: 0,
            offline_manifest_parse_count: 0,
            offline_artifact_stat_count: 0,
            offline_artifact_read_count: 0,
            offline_artifact_read_byte_count: 0,
            offline_artifact_decode_count: 0,
            offline_pixel_copy_count: 0,
            offline_pixel_copy_byte_count: 0,
            offline_manifest_eviction_count: 0,
            offline_artifact_eviction_count: 0,
            offline_glyph_bitmap_eviction_count: 0,
            offline_oldest_artifact_idle_access_count: 0,
            offline_oldest_glyph_bitmap_idle_access_count: 0,
            resident_baked_glyph_count: 0,
            resident_baked_glyph_byte_count: 0,
            baked_glyph_eviction_count: 0,
            oldest_baked_glyph_idle_access_count: 0,
            resident_source_context_count: 0,
            resident_source_byte_count: 0,
            source_context_created_count: 0,
            source_context_eviction_count: 0,
            oldest_source_context_idle_access_count: 0,
            source_hash_count: 0,
            face_parse_count: 0,
            generation_batch_count: 0,
            generation_requested_glyph_count: 0,
            generation_unique_glyph_count: 0,
            generation_duplicate_glyph_count: 0,
            bitmap_clone_byte_count: 0,
            resident_atlas_page_count: 0,
            atlas_page_alloc_count: 0,
            atlas_page_zero_byte_count: 0,
            atlas_page_clear_count: 0,
            atlas_page_clear_byte_count: 0,
            atlas_page_write_count: 0,
            atlas_page_write_byte_count: 0,
            atlas_page_reused_slot_count: 0,
            atlas_full_page_scan_byte_count: 0,
            compiled_atlas_build_count: 1,
            compiled_atlas_reuse_count: 0,
            generation_scheduler: crate::text::sdf::SdfGenerationSchedulerDiagnostics::default(),
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
        route_identity:
            crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextRouteIdentity::new(
                "runtime.sdf-render.test",
                zircon_runtime_interface::ui::event_ui::UiNodeId::new(1),
                None,
            ),
        command_generation: 1,
        raster_scale: 1.0,
        text: text.to_string(),
        frame,
        clip_frame: None,
        source_range: None,
        is_source_isomorphic_layout_line: false,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: false,
        glyph_artifact_line: None,
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

#[test]
fn text_owned_artifact_line_emits_rotated_sdf_vertices() {
    let artifact = std::sync::Arc::new(ResolvedTextGlyphArtifact {
        source_text: std::sync::Arc::from("fi"),
        source_text_origin: 0,
        font_generation: 7,
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::VerticalRl,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![TextGlyph {
                glyph_id: 0xfb02,
                source_range: 0..2,
                visual_range: 0..1,
                advance: 24.0,
                position: [0.0, 0.0],
                offset: [0.0, 0.0],
                font_face: None,
                font_instance: None,
                rotation: TextGlyphRotation::Clockwise90,
                bidi_level: 0,
                flags: TextGlyphFlags::default(),
                requires_rasterization: true,
            }],
            layout_line: UiResolvedTextLine {
                text: "fi".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(16.0, 16.0, 40.0, 40.0),
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
        logical_virtual_line_sequences: None,
    });
    let mut text = text_batch("fi", UiFrame::new(16.0, 16.0, 40.0, 40.0));
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.glyph_advances = vec![24.0];
    text.glyph_artifact_line = Some(ScreenSpaceUiGlyphArtifactLine {
        artifact,
        line_index: 0,
        font_generation: 7,
        glyph_range: 0..1,
    });

    let plan = synthetic_layered_plan(0);
    let atlas_bake = synthetic_layered_bake(&plan);
    let asset_manager = ProjectAssetManager::default();
    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &asset_manager,
        UVec2::new(96, 96),
    );

    assert_eq!(vertices.len(), 6);
    assert_eq!(vertices[0].uv, [0.0, 0.5]);
    assert_eq!(vertices[0].primitive_kind, SDF_TEXT_PRIMITIVE_GLYPH);
}
