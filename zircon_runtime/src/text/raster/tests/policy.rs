use super::*;

#[test]
fn text_raster_policy_prefers_bitmap_for_small_static_text() {
    assert_eq!(raster_path_for(12.0, false), GlyphRasterPath::Bitmap);
}

#[test]
fn text_raster_policy_uses_sdf_for_large_or_scalable_text() {
    assert_eq!(raster_path_for(32.0, false), GlyphRasterPath::Sdf);
    assert_eq!(raster_path_for(12.0, true), GlyphRasterPath::Sdf);
}

#[test]
fn text_raster_policy_can_disable_scalable_sdf_preference() {
    let policy = GlyphRasterPolicy {
        sdf_min_size_px: 18.0,
        scalable_prefers_sdf: false,
    };

    assert_eq!(policy.path_for(17.0, true), GlyphRasterPath::Bitmap);
    assert_eq!(policy.path_for(18.0, true), GlyphRasterPath::Sdf);
}

#[test]
fn text_policy_outline_effect_forces_sdf_path() {
    let mut request = GlyphRasterPolicyRequest::new(12.0, false);
    request.effects.outline = true;

    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Sdf);

    request.effects = GlyphRasterEffects {
        outline: false,
        shadow: true,
        glow: false,
        true_distance_effects: false,
    };
    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Sdf);
}

#[test]
fn text_raster_policy_honors_explicit_distance_field_formats() {
    let mut request = GlyphRasterPolicyRequest::new(12.0, false);
    request.requested_format = GlyphAtlasFormat::Sdf;
    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Sdf);

    request.requested_format = GlyphAtlasFormat::Msdf;
    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Msdf);
}

#[test]
fn text_raster_policy_keeps_color_glyphs_on_bitmap_path() {
    let mut request = GlyphRasterPolicyRequest::new(64.0, true);
    request.requested_format = GlyphAtlasFormat::Color;
    request.effects.glow = true;

    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Bitmap);

    request.requested_format = GlyphAtlasFormat::SubpixelMask;
    request.effects.outline = true;

    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Bitmap);
}

#[test]
fn text_raster_policy_has_no_unreachable_format_branch() {
    let source = include_str!("../policy.rs");

    assert!(!source.contains(concat!("unreachable", "!(")));
}

#[test]
fn text_raster_policy_selects_mtsdf_only_for_explicit_true_distance_effects() {
    let mut request = GlyphRasterPolicyRequest::new(48.0, false);
    request.effects = GlyphRasterEffects {
        outline: true,
        true_distance_effects: true,
        ..GlyphRasterEffects::default()
    };

    assert_eq!(raster_path_for_request(request), GlyphRasterPath::Mtsdf);
    assert_eq!(
        distance_field_mode_for_request(request),
        Some(SdfMode::Mtsdf)
    );
    request.effects.true_distance_effects = false;
    assert_eq!(distance_field_mode_for_request(request), Some(SdfMode::Sdf));
}

#[test]
fn text_raster_policy_upgrades_explicit_sdf_or_msdf_when_glow_needs_true_distance() {
    for requested_format in [GlyphAtlasFormat::Sdf, GlyphAtlasFormat::Msdf] {
        let mut request = GlyphRasterPolicyRequest::new(12.0, false);
        request.requested_format = requested_format;
        request.effects = GlyphRasterEffects {
            glow: true,
            true_distance_effects: true,
            ..GlyphRasterEffects::default()
        };

        assert_eq!(
            distance_field_mode_for_request(request),
            Some(SdfMode::Mtsdf)
        );
    }
}
