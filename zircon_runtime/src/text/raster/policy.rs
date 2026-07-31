use crate::text::atlas::GlyphAtlasFormat;
use crate::text::sdf::SdfMode;

const DEFAULT_SDF_MIN_SIZE_PX: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphRasterPath {
    Bitmap,
    Sdf,
    Msdf,
    Mtsdf,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphRasterEffects {
    pub(crate) outline: bool,
    pub(crate) shadow: bool,
    pub(crate) glow: bool,
    pub(crate) true_distance_effects: bool,
}

impl GlyphRasterEffects {
    fn requires_distance_field(self) -> bool {
        self.outline || self.shadow || self.glow
    }

    fn requires_true_distance(self) -> bool {
        self.requires_distance_field() && self.true_distance_effects
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphRasterPolicyRequest {
    pub(crate) size_px: f32,
    pub(crate) scalable: bool,
    pub(crate) requested_format: GlyphAtlasFormat,
    pub(crate) effects: GlyphRasterEffects,
}

impl GlyphRasterPolicyRequest {
    pub(crate) fn new(size_px: f32, scalable: bool) -> Self {
        Self {
            size_px,
            scalable,
            requested_format: GlyphAtlasFormat::AlphaMask,
            effects: GlyphRasterEffects::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphRasterPolicy {
    pub(crate) sdf_min_size_px: f32,
    pub(crate) scalable_prefers_sdf: bool,
}

impl Default for GlyphRasterPolicy {
    fn default() -> Self {
        Self {
            sdf_min_size_px: DEFAULT_SDF_MIN_SIZE_PX,
            scalable_prefers_sdf: true,
        }
    }
}

pub(crate) fn raster_path_for(size_px: f32, scalable: bool) -> GlyphRasterPath {
    GlyphRasterPolicy::default().path_for(size_px, scalable)
}

pub(crate) fn raster_path_for_request(request: GlyphRasterPolicyRequest) -> GlyphRasterPath {
    GlyphRasterPolicy::default().path_for_request(request)
}

pub(crate) fn distance_field_mode_for_request(
    request: GlyphRasterPolicyRequest,
) -> Option<SdfMode> {
    match raster_path_for_request(request) {
        GlyphRasterPath::Bitmap => None,
        GlyphRasterPath::Sdf => Some(SdfMode::Sdf),
        GlyphRasterPath::Msdf => Some(SdfMode::Msdf),
        GlyphRasterPath::Mtsdf => Some(SdfMode::Mtsdf),
    }
}

impl GlyphRasterPolicy {
    pub(crate) fn path_for(self, size_px: f32, scalable: bool) -> GlyphRasterPath {
        self.path_for_request(GlyphRasterPolicyRequest::new(size_px, scalable))
    }

    pub(crate) fn path_for_request(self, request: GlyphRasterPolicyRequest) -> GlyphRasterPath {
        match request.requested_format {
            GlyphAtlasFormat::SubpixelMask | GlyphAtlasFormat::Color => {
                return GlyphRasterPath::Bitmap;
            }
            _ if request.effects.requires_true_distance() => return GlyphRasterPath::Mtsdf,
            GlyphAtlasFormat::Sdf => return GlyphRasterPath::Sdf,
            GlyphAtlasFormat::Msdf => return GlyphRasterPath::Msdf,
            GlyphAtlasFormat::AlphaMask => {}
        }

        if request.effects.requires_distance_field() {
            return GlyphRasterPath::Sdf;
        }

        if request.scalable && self.scalable_prefers_sdf {
            return GlyphRasterPath::Sdf;
        }

        if request.size_px >= self.sdf_min_size_px {
            GlyphRasterPath::Sdf
        } else {
            GlyphRasterPath::Bitmap
        }
    }
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("policy.rs");

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
}
