use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};
use crate::text::VariationCoords;

use super::*;

mod decode;
mod fdsm_gen;
mod generation_scheduler;
mod offline;

#[test]
fn text_sdf_shared_params_use_plan_approved_48_8_defaults() {
    let params = SdfBakeParams::default();

    assert_eq!(params.mode, SdfMode::Sdf);
    assert_eq!(params.bake_em_px, 48);
    assert_eq!(params.spread_px_milli, 8_000);
    assert!((params.screen_px_range(48.0) - 8.0).abs() < f32::EPSILON);
}

#[test]
fn text_sdf_shared_params_normalize_zero_dimensions() {
    let params = SdfBakeParams {
        mode: SdfMode::Mtsdf,
        bake_em_px: 0,
        spread_px_milli: 0,
    }
    .normalized();

    assert_eq!(params.bake_em_px, 1);
    assert_eq!(params.spread_px_milli, 1);
    assert_eq!(params.mode, SdfMode::Mtsdf);
}

#[test]
fn text_sdf_modes_have_stable_atlas_and_shader_identity() {
    assert_eq!(SdfMode::Sdf.channel_count(), 1);
    assert_eq!(SdfMode::Sdf.atlas_format(), GlyphAtlasFormat::Sdf);
    assert_eq!(SdfMode::Sdf.shader_discriminant(), 0);

    for (mode, shader_discriminant) in [(SdfMode::Msdf, 1), (SdfMode::Mtsdf, 2)] {
        assert_eq!(mode.channel_count(), 4);
        assert_eq!(mode.atlas_format(), GlyphAtlasFormat::Msdf);
        assert_eq!(
            mode.atlas_format().storage_format(),
            GlyphAtlasStorageFormat::Rgba8Unorm
        );
        assert_eq!(mode.shader_discriminant(), shader_discriminant);
    }
}

#[test]
fn text_sdf_glyph_data_rejects_mismatched_byte_length() {
    let glyph = SdfGlyphData {
        size: UVec2::new(2, 2),
        bitmap_left: 0.0,
        bitmap_bottom: 0.0,
        advance: 2.0,
        ascent: 2.0,
        pixels: vec![0; 15],
        channels: 4,
        spread_px: 8.0,
        mode: SdfMode::Msdf,
    };

    assert_eq!(
        glyph.validate(),
        Err(SdfGlyphGenerationError::InvalidOutputLength {
            expected: 16,
            actual: 15,
        })
    );
}

#[test]
fn text_sdf_variation_hash_is_order_stable_and_instance_sensitive() {
    let forward = VariationCoords(vec![
        (u32::from_be_bytes(*b"wght"), 650.0),
        (u32::from_be_bytes(*b"wdth"), 90.0),
    ]);
    let reversed = VariationCoords(vec![
        (u32::from_be_bytes(*b"wdth"), 90.0),
        (u32::from_be_bytes(*b"wght"), 650.0),
    ]);
    let expanded = VariationCoords(vec![
        (u32::from_be_bytes(*b"wdth"), 110.0),
        (u32::from_be_bytes(*b"wght"), 650.0),
    ]);

    assert_eq!(sdf_variation_hash(&forward), sdf_variation_hash(&reversed));
    assert_ne!(sdf_variation_hash(&forward), sdf_variation_hash(&expanded));
    assert_eq!(
        sdf_default_variation_hash(),
        sdf_variation_hash(&VariationCoords::default())
    );
}
