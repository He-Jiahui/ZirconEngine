use super::super::material::{
    SDF_TEXT_EFFECT_GLOW, SDF_TEXT_EFFECT_OUTLINE, SDF_TEXT_EFFECT_SHADOW, SdfScreenPxRangeMode,
    SdfTextMaterial, SdfTextMaterialDrawPlan, SdfTextMaterialUniform, aligned_uniform_stride,
    fragment_screen_px_range, mtsdf_glow_coverage, sdf_effect_coverage, shadow_sample_uv,
    straight_alpha_over,
};
use super::*;
use crate::graphics::scene::scene_renderer::ui::render::text_effects::{
    ScreenSpaceUiTextEffects, ScreenSpaceUiTextGlow, ScreenSpaceUiTextOutline,
    ScreenSpaceUiTextShadow,
};
use crate::graphics::scene::scene_renderer::ui::render::text_projection::ScreenSpaceUiTextClipTransform;

#[test]
fn render_text_stable_materials_reuse_upload_state() {
    let source = include_str!("../material.rs");
    let uploaded_materials = ["uploaded_", "materials"].concat();
    let upload_bytes = ["upload_", "bytes"].concat();

    assert!(source.contains(&uploaded_materials));
    assert!(source.contains(&upload_bytes));
}

#[test]
fn render_text_advance_count_does_not_materialize_render_scalars() {
    let source = include_str!("../../sdf_render.rs");
    let allocating_count = ["render_scalars()", ".len()"].concat();

    assert!(!source.contains(&allocating_count));
}

#[test]
fn render_text_material_uniform_abi_is_seven_aligned_vec4_slots() {
    assert_eq!(std::mem::size_of::<SdfTextMaterialUniform>(), 112);
    assert_eq!(std::mem::align_of::<SdfTextMaterialUniform>(), 16);
    assert_eq!(std::mem::size_of::<SdfTextMaterialUniform>() % 16, 0);
    assert_eq!(aligned_uniform_stride(112, 256), 256);
    assert_eq!(aligned_uniform_stride(112, 64), 128);
}

#[test]
fn render_text_material_clamps_effects_to_available_distance_range() {
    let mut text = text_batch("effects", UiFrame::new(8.0, 12.0, 120.0, 36.0));
    text.font_size = 12.0;
    text.distance_field_mode = SdfMode::Mtsdf;
    text.text_effects = ScreenSpaceUiTextEffects {
        outline: Some(ScreenSpaceUiTextOutline {
            width_px: 20.0,
            color: [1.0, 0.0, 0.0, 1.0],
        }),
        shadow: Some(ScreenSpaceUiTextShadow {
            offset_px: [20.0, -20.0],
            color: [0.0, 0.0, 0.0, 0.5],
        }),
        glow: Some(ScreenSpaceUiTextGlow {
            radius_px: 20.0,
            color: [0.0, 0.5, 1.0, 0.75],
        }),
    };

    let material = SdfTextMaterial::from_text(&text, UVec2::splat(512));
    let limit = (SdfBakeParams::default().screen_px_range(12.0) * 0.5).max(1.0);
    assert_eq!(material.outline_width_px, limit);
    assert_eq!(material.shadow_offset_px, [limit, -limit]);
    assert_eq!(material.glow_radius_px, limit);
    assert_eq!(
        material.effect_flags,
        SDF_TEXT_EFFECT_OUTLINE | SDF_TEXT_EFFECT_SHADOW | SDF_TEXT_EFFECT_GLOW
    );
    assert_eq!(
        material.projection_mode,
        SdfScreenPxRangeMode::CpuScreenSpace
    );
    assert_eq!(
        material.uniform().flags[2],
        SdfMode::Mtsdf.shader_discriminant()
    );
}

#[test]
fn render_text_material_draw_plan_coalesces_adjacent_equal_batches() {
    let first = text_batch("A", UiFrame::new(0.0, 0.0, 20.0, 20.0));
    let mut second = first.clone();
    second.text = "B".to_string();
    let plan = SdfTextMaterialDrawPlan::from_ranges(
        &[first, second],
        UVec2::splat(512),
        0,
        &[0..6, 6..12],
    );

    assert_eq!(plan.materials.len(), 1);
    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.draws[0].vertices, 0..12);
    assert_eq!(plan.draws[0].material_index, 0);
}

#[test]
fn render_text_outline_thickness_matches_distance_offset() {
    assert_eq!(sdf_effect_coverage(0.5, 8.0, 0.0), 0.5);
    assert_eq!(sdf_effect_coverage(0.375, 8.0, 0.0), 0.0);
    assert_eq!(sdf_effect_coverage(0.375, 8.0, 1.0), 0.5);
    assert_eq!(sdf_effect_coverage(0.375, 8.0, 1.5), 1.0);
}

#[test]
fn render_text_shadow_offset_correct_for_rotated_uv_derivatives() {
    let shifted = shadow_sample_uv([0.5, 0.5], [0.0, 0.01], [-0.01, 0.0], [3.0, -2.0]);
    assert!((shifted[0] - 0.48).abs() < 0.0001);
    assert!((shifted[1] - 0.47).abs() < 0.0001);
}

#[test]
fn render_text_sdf_rotated_screen_px_range_sharp() {
    let derivative = (1.0 / 1024.0) / std::f32::consts::SQRT_2;
    let screen_range = fragment_screen_px_range(
        8.0,
        [512.0, 512.0],
        [derivative, derivative],
        [-derivative, derivative],
    );

    assert!((screen_range - 8.0 * std::f32::consts::SQRT_2).abs() < 0.001);
    assert!(sdf_effect_coverage(0.5 - 0.5 / screen_range, screen_range, 0.0).abs() < 0.00001);
    assert!(
        (sdf_effect_coverage(0.5 + 0.5 / screen_range, screen_range, 0.0) - 1.0).abs() < 0.00001
    );
}

#[test]
fn render_text_msdf_3d_space_sharp_at_distance() {
    let near = fragment_screen_px_range(
        8.0,
        [512.0, 512.0],
        [1.0 / 1024.0, 0.0],
        [0.0, 1.0 / 1024.0],
    );
    let far = fragment_screen_px_range(8.0, [512.0, 512.0], [1.0 / 512.0, 0.0], [0.0, 1.0 / 512.0]);

    assert!((near - 16.0).abs() < 0.001);
    assert!((far - 8.0).abs() < 0.001);
    assert!((near / far - 2.0).abs() < 0.001);
}

#[test]
fn render_text_fragment_projection_mode_follows_clip_transform() {
    let mut text = text_batch("Perspective", UiFrame::new(8.0, 12.0, 120.0, 36.0));
    text.clip_transform = Some(ScreenSpaceUiTextClipTransform::from_rows([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.35, 0.0, 0.0, 1.0],
    ]));

    let material = SdfTextMaterial::from_text(&text, UVec2::new(512, 256));

    assert_eq!(
        material.projection_mode,
        SdfScreenPxRangeMode::FragmentDerived
    );
    assert_eq!(
        material.uniform().projection_params,
        [512.0, 256.0, 0.0, 0.0]
    );
}

#[test]
fn render_text_mtsdf_glow_uses_true_distance_outside_fill() {
    let edge = mtsdf_glow_coverage(0.5, 8.0, 2.0);
    let one_pixel_outside = mtsdf_glow_coverage(0.375, 8.0, 2.0);
    let outside_radius = mtsdf_glow_coverage(0.25, 8.0, 2.0);
    assert!(edge > 0.0);
    assert!(one_pixel_outside > outside_radius);
    assert_eq!(outside_radius, 0.0);
}

#[test]
fn render_text_effect_layers_use_straight_alpha_over() {
    let result = straight_alpha_over([0.0, 0.0, 1.0, 0.5], [1.0, 0.0, 0.0, 0.5]);
    assert!((result[0] - 2.0 / 3.0).abs() < 0.0001);
    assert!((result[2] - 1.0 / 3.0).abs() < 0.0001);
    assert!((result[3] - 0.75).abs() < 0.0001);
}

#[test]
fn render_text_effect_shader_uses_group2_material_and_derivative_shadow() {
    assert!(SDF_TEXT_SHADER.contains("@group(2) @binding(0)"));
    assert!(SDF_TEXT_SHADER.contains("dpdx(input.uv) * offset.x"));
    assert!(SDF_TEXT_SHADER.contains("dpdy(input.uv) * offset.y"));
    assert!(SDF_TEXT_SHADER.contains("fn straight_alpha_over"));
    assert!(SDF_TEXT_SHADER.contains("distances.y"));
    assert!(SDF_TEXT_SHADER.contains("max(fwidth(input.uv)"));
    assert!(SDF_TEXT_SHADER.contains("text_material.projection_params.xy"));
    naga::front::wgsl::parse_str(SDF_TEXT_SHADER)
        .expect("group2 text material effect shader should parse");
}
