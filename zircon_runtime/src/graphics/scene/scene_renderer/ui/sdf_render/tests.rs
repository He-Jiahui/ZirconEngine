use super::super::sdf_upload::SdfAtlasUploadMode;
use super::*;
use crate::asset::ProjectAssetManager;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::plan_sdf_atlas;
use crate::graphics::text::font::FontDatabase;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextWrap};

#[test]
fn sdf_draw_plan_creates_one_textured_quad_per_glyph() {
    let text = text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    assert_eq!(vertices.len(), 12);
    assert_eq!(vertices[0].color, [0.2, 0.3, 0.4, 0.5]);
    assert!(vertices[0].uv[0] >= 0.0);
    assert!(vertices[0].uv[1] >= 0.0);
    assert!(vertices[0].uv[0] < vertices[2].uv[0]);
    assert!(vertices[6].uv[0] > vertices[0].uv[0]);
}

#[test]
fn sdf_draw_plan_skips_whitespace_quads_but_preserves_advance() {
    let text = text_batch("A B", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);
    let a = font_bake.measure_glyph(
        'A',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.font_size,
        &mut font_database,
        &asset_manager,
    );
    let space = font_bake.measure_glyph(
        ' ',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.font_size,
        &mut font_database,
        &asset_manager,
    );
    let b = font_bake.measure_glyph(
        'B',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.font_size,
        &mut font_database,
        &asset_manager,
    );

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    let expected_second_glyph_x = 8.0 + a.advance + space.advance + b.bitmap_left;
    assert_eq!(vertices.len(), 12);
    assert!(
        (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 128.0)).abs() < 0.0001
    );
}

#[test]
fn sdf_draw_plan_clips_to_text_frame_without_explicit_clip() {
    let text = text_batch("AAAA", UiFrame::new(8.0, 12.0, 24.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    let max_x = vertices
        .iter()
        .map(|vertex| vertex.position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(!vertices.is_empty());
    assert!(vertices.len() <= 24);
    assert!(max_x <= pixel_to_ndc_x(text.frame.right(), 128.0) + 0.0001);
}

#[test]
fn sdf_draw_plan_clips_glyph_vertices_and_uvs() {
    let mut text = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
    text.clip_frame = Some(UiFrame::new(12.0, 12.0, 32.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    assert_eq!(vertices.len(), 6);
    assert!(vertices[0].position[0] > -0.875);
    assert!(vertices[0].uv[0] > 0.0);
}

#[test]
fn sdf_draw_plan_applies_text_alignment_inside_frame() {
    let mut centered = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    centered.text_align = UiTextAlign::Center;
    let centered_plan = plan_sdf_atlas(std::slice::from_ref(&centered));
    let (mut centered_bake, mut centered_database, centered_assets, centered_atlas_bake) =
        bake_atlas(&centered_plan);
    let centered_width = text_advance(
        &mut centered_bake,
        &mut centered_database,
        &centered_assets,
        &centered,
    );
    let centered_first = centered_bake.measure_glyph(
        'A',
        centered.font.as_deref(),
        centered.font_family.as_deref(),
        centered.font_size,
        &mut centered_database,
        &centered_assets,
    );

    let centered_vertices = build_sdf_vertices(
        std::slice::from_ref(&centered),
        &centered_plan,
        &centered_atlas_bake,
        &mut centered_bake,
        &mut centered_database,
        &centered_assets,
        UVec2::new(128, 64),
    );

    let expected_centered_x = centered.frame.x
        + (centered.frame.width - centered_width) * 0.5
        + centered_first.bitmap_left;
    assert!(
        (centered_vertices[0].position[0] - pixel_to_ndc_x(expected_centered_x, 128.0)).abs()
            < 0.0001
    );

    let mut right_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    right_aligned.text_align = UiTextAlign::Right;
    let right_plan = plan_sdf_atlas(std::slice::from_ref(&right_aligned));
    let (mut right_bake, mut right_database, right_assets, right_atlas_bake) =
        bake_atlas(&right_plan);
    let right_width = text_advance(
        &mut right_bake,
        &mut right_database,
        &right_assets,
        &right_aligned,
    );
    let right_first = right_bake.measure_glyph(
        'A',
        right_aligned.font.as_deref(),
        right_aligned.font_family.as_deref(),
        right_aligned.font_size,
        &mut right_database,
        &right_assets,
    );

    let right_vertices = build_sdf_vertices(
        std::slice::from_ref(&right_aligned),
        &right_plan,
        &right_atlas_bake,
        &mut right_bake,
        &mut right_database,
        &right_assets,
        UVec2::new(128, 64),
    );

    let expected_right_x = right_aligned.frame.right() - right_width + right_first.bitmap_left;
    assert!(
        (right_vertices[0].position[0] - pixel_to_ndc_x(expected_right_x, 128.0)).abs() < 0.0001
    );
}

#[test]
fn sdf_draw_plan_maps_start_end_through_rtl_direction() {
    let mut start_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    start_aligned.text_align = UiTextAlign::Start;
    start_aligned.text_direction = UiTextDirection::RightToLeft;
    let start_plan = plan_sdf_atlas(std::slice::from_ref(&start_aligned));
    let (mut start_bake, mut start_database, start_assets, start_atlas_bake) =
        bake_atlas(&start_plan);
    let start_width = text_advance(
        &mut start_bake,
        &mut start_database,
        &start_assets,
        &start_aligned,
    );
    assert!(
        (aligned_text_start_x(&start_aligned, start_width)
            - (start_aligned.frame.right() - start_width))
            .abs()
            < 0.0001
    );
    let start_first = start_bake.measure_glyph(
        'A',
        start_aligned.font.as_deref(),
        start_aligned.font_family.as_deref(),
        start_aligned.font_size,
        &mut start_database,
        &start_assets,
    );

    let start_vertices = build_sdf_vertices(
        std::slice::from_ref(&start_aligned),
        &start_plan,
        &start_atlas_bake,
        &mut start_bake,
        &mut start_database,
        &start_assets,
        UVec2::new(128, 64),
    );

    let expected_start_x = start_aligned.frame.right() - start_width + start_first.bitmap_left;
    assert!(
        (start_vertices[0].position[0] - pixel_to_ndc_x(expected_start_x, 128.0)).abs() < 0.0001
    );

    let mut end_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    end_aligned.text_align = UiTextAlign::End;
    end_aligned.text_direction = UiTextDirection::RightToLeft;
    let end_plan = plan_sdf_atlas(std::slice::from_ref(&end_aligned));
    let (mut end_bake, mut end_database, end_assets, end_atlas_bake) = bake_atlas(&end_plan);
    let end_width = text_advance(&mut end_bake, &mut end_database, &end_assets, &end_aligned);
    assert!((aligned_text_start_x(&end_aligned, end_width) - end_aligned.frame.x).abs() < 0.0001);
    let end_first = end_bake.measure_glyph(
        'A',
        end_aligned.font.as_deref(),
        end_aligned.font_family.as_deref(),
        end_aligned.font_size,
        &mut end_database,
        &end_assets,
    );

    let end_vertices = build_sdf_vertices(
        std::slice::from_ref(&end_aligned),
        &end_plan,
        &end_atlas_bake,
        &mut end_bake,
        &mut end_database,
        &end_assets,
        UVec2::new(128, 64),
    );

    let expected_end_x = (end_aligned.frame.x + end_first.bitmap_left).max(end_aligned.frame.x);
    assert!((end_vertices[0].position[0] - pixel_to_ndc_x(expected_end_x, 128.0)).abs() < 0.0001);
}

#[test]
fn sdf_prepare_report_summarizes_atlas_bake_and_vertices() {
    let plan = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0))]);
    let bake_report = super::SdfAtlasBakeReport {
        slot_count: 2,
        visible_glyph_count: 2,
        empty_glyph_count: 0,
        atlas_byte_len: 512 * 512,
        nonzero_pixel_count: 64,
        loaded_font_count: 1,
    };

    let cache_report = SdfAtlasCacheReport {
        previous_slot_count: 0,
        current_slot_count: 2,
        retained_slot_count: 0,
        stable_slot_count: 0,
        relocated_slot_count: 0,
        added_slot_count: 2,
        evicted_slot_count: 0,
        atlas_resized: true,
    };

    let report = sdf_prepare_report(
        1,
        &plan,
        cache_report,
        true,
        bake_report,
        512 * 512,
        true,
        12,
    );

    assert_eq!(
        report,
        ScreenSpaceUiSdfPrepareReport {
            text_batch_count: 1,
            atlas_slot_count: 2,
            atlas_size: plan.atlas_size,
            atlas_resized: true,
            bake: bake_report,
            atlas_upload_byte_len: 512 * 512,
            atlas_upload_full_texture: true,
            atlas_upload: SdfAtlasUploadReport {
                mode: SdfAtlasUploadMode::FullTexture,
                byte_len: 512 * 512,
                full_texture: true,
                dirty_slot_count: 2,
                dirty_byte_len: 512 * 512,
            },
            vertex_count: 12,
        }
    );
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
    let atlas_bake = font_bake.build_atlas(plan, &mut font_database, &asset_manager);
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
                    text.font_size,
                    font_database,
                    asset_manager,
                )
                .advance
        })
        .sum()
}

fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame,
        clip_frame: None,
        color: [0.2, 0.3, 0.4, 0.5],
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        wrap: UiTextWrap::None,
        style: Default::default(),
    }
}
