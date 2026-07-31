use zircon_runtime_interface::ui::surface::{UiTextDistanceFieldEffects, UiTextRenderMode};

use crate::text::atlas::GlyphAtlasFormat;
use crate::text::raster::{
    GlyphRasterEffects, GlyphRasterPolicyRequest, distance_field_mode_for_request,
};
use crate::text::sdf::SdfMode;

pub(super) fn resolved_text_distance_field_mode(
    render_mode: UiTextRenderMode,
    font_size: f32,
    effects: &UiTextDistanceFieldEffects,
) -> SdfMode {
    let mut request = GlyphRasterPolicyRequest::new(font_size, false);
    request.requested_format = match render_mode {
        UiTextRenderMode::Sdf => GlyphAtlasFormat::Sdf,
        UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => GlyphAtlasFormat::Msdf,
        UiTextRenderMode::Auto | UiTextRenderMode::Native => GlyphAtlasFormat::AlphaMask,
    };
    request.effects = GlyphRasterEffects {
        outline: effects.outline.is_some(),
        shadow: effects.shadow.is_some(),
        glow: effects.glow.is_some(),
        true_distance_effects: effects.requires_true_distance()
            || matches!(render_mode, UiTextRenderMode::Mtsdf),
    };
    distance_field_mode_for_request(request).unwrap_or(SdfMode::Sdf)
}
