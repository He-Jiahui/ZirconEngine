use super::*;
use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasStorageFormat};

#[test]
fn render_text_atlas_subpixel_coverage_uses_dedicated_blend_contract() {
    let contract = GlyphAtlasRenderContract::for_sampling_semantics(
        GlyphAtlasSamplingSemantics::SubpixelCoverage,
    );

    assert_eq!(
        contract.shader_decode,
        GlyphAtlasShaderDecode::SubpixelRgbCoverage
    );
    assert_eq!(
        contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert!(contract.requires_background_composite());
}

#[test]
fn render_text_atlas_rgba_storage_does_not_choose_color_shader_by_storage_alone() {
    let subpixel_page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
        UVec2::new(64, 64),
    );
    let color_page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
        UVec2::new(64, 64),
    );

    assert_eq!(
        subpixel_page.storage_format,
        GlyphAtlasStorageFormat::Rgba8Unorm
    );
    assert_eq!(
        color_page.storage_format,
        GlyphAtlasStorageFormat::Rgba8Unorm
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_page(&subpixel_page).shader_decode,
        GlyphAtlasShaderDecode::SubpixelRgbCoverage
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_page(&color_page).shader_decode,
        GlyphAtlasShaderDecode::ColorRgba
    );
}

#[test]
fn render_text_atlas_sampling_shader_declares_subpixel_rgb_decode() {
    assert!(GLYPH_ATLAS_SAMPLING_SHADER.contains("glyph_atlas_decode_subpixel_rgb_coverage"));
    assert!(
        GLYPH_ATLAS_SAMPLING_SHADER.contains("mix(colors.background.rgb, colors.foreground.rgb")
    );
    assert!(GLYPH_ATLAS_SAMPLING_SHADER.contains("vec4<f32>(rgb, colors.background.a)"));
}

#[test]
fn render_text_atlas_sampling_shader_declares_distance_field_decodes() {
    assert!(GLYPH_ATLAS_SAMPLING_SHADER.contains("glyph_atlas_decode_signed_distance_coverage"));
    assert!(
        GLYPH_ATLAS_SAMPLING_SHADER
            .contains("glyph_atlas_decode_multi_channel_signed_distance_coverage")
    );
    assert!(GLYPH_ATLAS_SAMPLING_SHADER.contains("glyph_atlas_median_rgb"));
}

#[test]
fn render_text_atlas_shader_entry_points_follow_decode_contracts() {
    assert_eq!(
        GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::AlphaCoverage
        )
        .shader_entry_points(),
        GlyphAtlasShaderEntryPoints {
            vertex: "vs_main",
            fragment: "fs_alpha_coverage",
        }
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::SubpixelCoverage,
        )
        .shader_entry_points()
        .fragment,
        "fs_subpixel_rgb_coverage"
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::SignedDistance,
        )
        .shader_entry_points()
        .fragment,
        "fs_signed_distance_coverage"
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::MultiChannelSignedDistance,
        )
        .shader_entry_points()
        .fragment,
        "fs_multi_channel_signed_distance_coverage"
    );
    assert_eq!(
        GlyphAtlasRenderContract::for_sampling_semantics(GlyphAtlasSamplingSemantics::ColorRgba)
            .shader_entry_points()
            .fragment,
        "fs_color_rgba"
    );
}

#[test]
fn render_text_atlas_pipeline_shader_declares_gpu_plan_bindings() {
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("@group(0) @binding(0)"));
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("texture_2d_array<f32>"));
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("@group(0) @binding(1)"));
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("@location(4) page_index"));
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("fs_subpixel_rgb_coverage"));
    assert!(GLYPH_ATLAS_PIPELINE_SHADER.contains("fs_color_rgba"));
}

#[test]
fn render_text_atlas_sampling_shader_wgsl_parses() {
    naga::front::wgsl::parse_str(GLYPH_ATLAS_SAMPLING_SHADER)
        .expect("glyph atlas sampling shader contract should parse");
}

#[test]
fn render_text_atlas_full_pipeline_shader_wgsl_parses() {
    naga::front::wgsl::parse_str(GLYPH_ATLAS_TEXT_SHADER)
        .expect("glyph atlas full pipeline shader contract should parse");
}
