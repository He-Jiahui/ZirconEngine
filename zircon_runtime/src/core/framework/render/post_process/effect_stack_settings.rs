use crate::core::math::{Real, Vec3};
use crate::core::resource::{ResourceHandle, TextureMarker};

use super::super::{RenderImageDescriptor, RenderImageDimension};

pub const MIN_COLOR_LOOKUP_TEXTURE_SIZE: u32 = 2;
pub const MAX_COLOR_LOOKUP_TEXTURE_SIZE: u32 = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderTonemapOperator {
    #[default]
    None,
    Reinhard,
    Aces,
    Filmic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTonemapSettings {
    pub operator: RenderTonemapOperator,
    pub exposure_bias: Real,
    pub white_point: Real,
}

impl Default for RenderTonemapSettings {
    fn default() -> Self {
        Self {
            operator: RenderTonemapOperator::None,
            exposure_bias: 0.0,
            white_point: 1.0,
        }
    }
}

impl RenderTonemapSettings {
    pub fn is_enabled(self) -> bool {
        self.operator != RenderTonemapOperator::None
            || self.exposure_bias != 0.0
            || self.white_point != 1.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderColorLookupTextureLayout {
    #[default]
    Auto,
    Texture2dStrip {
        size: u32,
    },
    Texture3d {
        size: u32,
    },
}

impl RenderColorLookupTextureLayout {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Texture2dStrip { .. } => "texture-2d-strip",
            Self::Texture3d { .. } => "texture-3d",
        }
    }

    pub fn requested_size(self) -> Option<u32> {
        match self {
            Self::Auto => None,
            Self::Texture2dStrip { size } | Self::Texture3d { size } => Some(size),
        }
    }

    pub fn has_valid_requested_size(self) -> bool {
        self.requested_size().is_none_or(|size| {
            (MIN_COLOR_LOOKUP_TEXTURE_SIZE..=MAX_COLOR_LOOKUP_TEXTURE_SIZE).contains(&size)
        })
    }

    pub fn matches_texture_2d_strip(self, descriptor: &RenderImageDescriptor) -> bool {
        let size = match self {
            Self::Auto => {
                let Some(size) = inferred_strip_size(descriptor) else {
                    return false;
                };
                size
            }
            Self::Texture2dStrip { size } => size,
            Self::Texture3d { .. } => return false,
        };
        self.has_valid_requested_size()
            && descriptor.dimension == RenderImageDimension::D2
            && descriptor.width == size.saturating_mul(size)
            && descriptor.height == size
            && descriptor.depth_or_array_layers.max(1) == 1
            && descriptor.array_layer_count.max(1) == 1
    }

    pub fn matches_texture_3d(self, descriptor: &RenderImageDescriptor) -> bool {
        let size = match self {
            Self::Auto => return descriptor.dimension == RenderImageDimension::D3,
            Self::Texture3d { size } => size,
            Self::Texture2dStrip { .. } => return false,
        };
        self.has_valid_requested_size()
            && descriptor.dimension == RenderImageDimension::D3
            && descriptor.width == size
            && descriptor.height == size
            && descriptor.depth_or_array_layers == size
    }

    pub fn accepts_current_post_process_binding(self, descriptor: &RenderImageDescriptor) -> bool {
        match self {
            Self::Auto => {
                descriptor.dimension == RenderImageDimension::D2
                    && descriptor.depth_or_array_layers.max(1) == 1
                    && descriptor.array_layer_count.max(1) == 1
            }
            Self::Texture2dStrip { .. } => self.matches_texture_2d_strip(descriptor),
            Self::Texture3d { .. } => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderColorLookupSettings {
    pub texture: Option<ResourceHandle<TextureMarker>>,
    pub texture_layout: RenderColorLookupTextureLayout,
    pub intensity: Real,
}

impl Default for RenderColorLookupSettings {
    fn default() -> Self {
        Self {
            texture: None,
            texture_layout: RenderColorLookupTextureLayout::Auto,
            intensity: 0.0,
        }
    }
}

impl RenderColorLookupSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBlurSettings {
    pub radius: Real,
}

impl Default for RenderBlurSettings {
    fn default() -> Self {
        Self { radius: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDepthOfFieldSettings {
    pub focus_distance: Real,
    pub aperture: Real,
    pub max_blur_radius: Real,
}

impl Default for RenderDepthOfFieldSettings {
    fn default() -> Self {
        Self {
            focus_distance: 10.0,
            aperture: 0.0,
            max_blur_radius: 0.0,
        }
    }
}

impl RenderDepthOfFieldSettings {
    pub fn is_enabled(self) -> bool {
        self.aperture > 0.0 || self.max_blur_radius > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderScreenSpaceReflectionSettings {
    pub intensity: Real,
    pub thickness: Real,
    pub max_ray_distance: Real,
    pub max_steps: u32,
}

impl Default for RenderScreenSpaceReflectionSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            thickness: 0.1,
            max_ray_distance: 50.0,
            max_steps: 64,
        }
    }
}

impl RenderScreenSpaceReflectionSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0 && self.max_steps > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVignetteSettings {
    pub intensity: Real,
    pub smoothness: Real,
    pub roundness: Real,
}

impl Default for RenderVignetteSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            smoothness: 0.5,
            roundness: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFilmGrainSettings {
    pub intensity: Real,
    pub response: Real,
}

impl Default for RenderFilmGrainSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            response: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDitherSettings {
    pub intensity: Real,
    pub scale: Real,
}

impl Default for RenderDitherSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            scale: 1.0,
        }
    }
}

impl RenderDitherSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderChromaticAberrationSettings {
    pub intensity: Real,
    pub sample_spread: Real,
}

impl Default for RenderChromaticAberrationSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            sample_spread: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFogSettings {
    pub density: Real,
    pub height_falloff: Real,
    pub color: Vec3,
}

impl Default for RenderFogSettings {
    fn default() -> Self {
        Self {
            density: 0.0,
            height_falloff: 0.0,
            color: Vec3::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct RenderPostProcessEffectStackSettings {
    pub tonemap: RenderTonemapSettings,
    pub color_lookup: RenderColorLookupSettings,
    pub blur: RenderBlurSettings,
    pub depth_of_field: RenderDepthOfFieldSettings,
    pub screen_space_reflection: RenderScreenSpaceReflectionSettings,
    pub vignette: RenderVignetteSettings,
    pub grain: RenderFilmGrainSettings,
    pub dither: RenderDitherSettings,
    pub chromatic_aberration: RenderChromaticAberrationSettings,
    pub fog: RenderFogSettings,
}

impl RenderPostProcessEffectStackSettings {
    pub fn is_enabled(self) -> bool {
        self.tonemap.is_enabled()
            || self.color_lookup.is_enabled()
            || self.blur.radius > 0.0
            || self.depth_of_field.is_enabled()
            || self.screen_space_reflection.is_enabled()
            || self.vignette.intensity > 0.0
            || self.grain.intensity > 0.0
            || self.dither.is_enabled()
            || self.chromatic_aberration.intensity > 0.0
            || self.fog.density > 0.0
    }

    pub fn report(self) -> RenderPostProcessEffectStackReport {
        RenderPostProcessEffectStackReport::from_settings(self)
    }

    pub fn report_with_resources(
        self,
        resources: RenderPostProcessEffectStackResourceStatus,
    ) -> RenderPostProcessEffectStackReport {
        RenderPostProcessEffectStackReport::from_settings_with_resources(self, resources)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPostProcessEffectStackResourceStatus {
    pub ssr_normal_available: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPostProcessEffectStackReport {
    pub enabled: bool,
    pub active_family_count: usize,
    pub active_families: Vec<String>,
    pub approximated_family_count: usize,
    pub approximated_families: Vec<String>,
    pub missing_resource_count: usize,
    pub missing_resources: Vec<String>,
}

impl RenderPostProcessEffectStackReport {
    pub fn from_settings(settings: RenderPostProcessEffectStackSettings) -> Self {
        Self::from_settings_with_resources(
            settings,
            RenderPostProcessEffectStackResourceStatus::default(),
        )
    }

    pub fn from_settings_with_resources(
        settings: RenderPostProcessEffectStackSettings,
        resources: RenderPostProcessEffectStackResourceStatus,
    ) -> Self {
        let mut report = Self::default();

        push_label(
            &mut report.active_families,
            settings.tonemap.is_enabled(),
            "tonemap",
        );
        push_label(
            &mut report.active_families,
            settings.color_lookup.is_enabled(),
            "lut",
        );
        push_label(
            &mut report.active_families,
            settings.blur.radius > 0.0,
            "blur",
        );
        push_label(
            &mut report.active_families,
            settings.depth_of_field.is_enabled(),
            "depth-of-field",
        );
        push_label(
            &mut report.active_families,
            settings.screen_space_reflection.is_enabled(),
            "screen-space-reflection",
        );
        push_label(
            &mut report.active_families,
            settings.vignette.intensity > 0.0,
            "vignette",
        );
        push_label(
            &mut report.active_families,
            settings.grain.intensity > 0.0,
            "film-grain",
        );
        push_label(
            &mut report.active_families,
            settings.dither.is_enabled(),
            "dither",
        );
        push_label(
            &mut report.active_families,
            settings.chromatic_aberration.intensity > 0.0,
            "chromatic-aberration",
        );
        push_label(
            &mut report.active_families,
            settings.fog.density > 0.0,
            "fog",
        );

        push_label(
            &mut report.approximated_families,
            settings.depth_of_field.is_enabled(),
            "depth-of-field",
        );
        push_label(
            &mut report.approximated_families,
            settings.screen_space_reflection.is_enabled(),
            "screen-space-reflection",
        );

        if settings.color_lookup.intensity > 0.0 && settings.color_lookup.texture.is_none() {
            report
                .missing_resources
                .push("effect-stack.lut.texture".to_string());
        }
        if settings.color_lookup.intensity > 0.0
            && !settings
                .color_lookup
                .texture_layout
                .has_valid_requested_size()
        {
            report
                .missing_resources
                .push("effect-stack.lut.texture-layout".to_string());
        }
        if settings.screen_space_reflection.is_enabled() && !resources.ssr_normal_available {
            report
                .missing_resources
                .push("effect-stack.ssr.normal".to_string());
        }

        report.enabled = !report.active_families.is_empty();
        report.active_family_count = report.active_families.len();
        report.approximated_family_count = report.approximated_families.len();
        report.missing_resource_count = report.missing_resources.len();
        report
    }
}

fn push_label(labels: &mut Vec<String>, enabled: bool, label: &str) {
    if enabled {
        labels.push(label.to_string());
    }
}

fn inferred_strip_size(descriptor: &RenderImageDescriptor) -> Option<u32> {
    let height = descriptor.height;
    (height > 0 && descriptor.width == height.saturating_mul(height)).then_some(height)
}

#[cfg(test)]
mod tests {
    use super::{
        RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderDepthOfFieldSettings,
        RenderDitherSettings, RenderPostProcessEffectStackResourceStatus,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
        RenderTonemapOperator, RenderTonemapSettings, RenderVignetteSettings,
    };
    use crate::core::framework::render::{
        RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
        RenderImageFallbackKind, RenderImageUsage, RenderSamplerDescriptor,
    };
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

    #[test]
    fn extended_effect_stack_settings_enable_product_node_without_legacy_fields() {
        let settings = RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Aces,
                ..Default::default()
            },
            dither: RenderDitherSettings {
                intensity: 0.1,
                ..Default::default()
            },
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 32,
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(settings.is_enabled());
    }

    #[test]
    fn color_lookup_intensity_requests_lut_even_without_texture_handle() {
        let settings = RenderColorLookupSettings {
            texture: None,
            texture_layout: RenderColorLookupTextureLayout::Auto,
            intensity: 0.25,
        };

        assert!(settings.is_enabled());
    }

    #[test]
    fn effect_stack_report_records_active_approximated_and_missing_resources() {
        let settings = RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Aces,
                ..Default::default()
            },
            color_lookup: RenderColorLookupSettings {
                texture: None,
                texture_layout: RenderColorLookupTextureLayout::Auto,
                intensity: 0.5,
            },
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 2.0,
                ..Default::default()
            },
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.4,
                max_steps: 16,
                ..Default::default()
            },
            vignette: RenderVignetteSettings {
                intensity: 0.25,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report();

        assert!(report.enabled);
        assert_eq!(report.active_family_count, 5);
        assert_eq!(
            report.active_families,
            labels([
                "tonemap",
                "lut",
                "depth-of-field",
                "screen-space-reflection",
                "vignette",
            ])
        );
        assert_eq!(report.approximated_family_count, 2);
        assert_eq!(
            report.approximated_families,
            labels(["depth-of-field", "screen-space-reflection"])
        );
        assert_eq!(report.missing_resource_count, 2);
        assert_eq!(
            report.missing_resources,
            labels(["effect-stack.lut.texture", "effect-stack.ssr.normal"])
        );
    }

    #[test]
    fn effect_stack_report_treats_authored_lut_as_renderer_bound_resource() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "postprocess/lut/filmic",
        ));
        let settings = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: Some(texture),
                texture_layout: RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
                intensity: 0.8,
            },
            ..Default::default()
        };

        let report = settings.report();

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["lut"]));
        assert_eq!(report.approximated_family_count, 0);
        assert!(report.approximated_families.is_empty());
        assert_eq!(report.missing_resource_count, 0);
        assert!(report.missing_resources.is_empty());
    }

    #[test]
    fn effect_stack_report_treats_bound_ssr_normal_as_available() {
        let settings = RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            ssr_normal_available: true,
        });

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["screen-space-reflection"]));
        assert_eq!(
            report.approximated_families,
            labels(["screen-space-reflection"])
        );
        assert_eq!(report.missing_resource_count, 0);
        assert!(report.missing_resources.is_empty());
    }

    #[test]
    fn color_lookup_texture_layout_accepts_current_2d_strip_contract() {
        let descriptor = texture_descriptor(33 * 33, 33, 1, RenderImageDimension::D2);
        let layout = RenderColorLookupTextureLayout::Texture2dStrip { size: 33 };

        assert_eq!(layout.label(), "texture-2d-strip");
        assert!(layout.matches_texture_2d_strip(&descriptor));
        assert!(layout.accepts_current_post_process_binding(&descriptor));
    }

    #[test]
    fn color_lookup_texture_3d_layout_is_recognized_but_not_2d_bindable() {
        let descriptor = texture_descriptor(33, 33, 33, RenderImageDimension::D3);
        let layout = RenderColorLookupTextureLayout::Texture3d { size: 33 };

        assert_eq!(layout.label(), "texture-3d");
        assert!(layout.matches_texture_3d(&descriptor));
        assert!(!layout.accepts_current_post_process_binding(&descriptor));
    }

    #[test]
    fn effect_stack_report_records_invalid_lut_layout_size() {
        let settings = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: None,
                texture_layout: RenderColorLookupTextureLayout::Texture3d { size: 0 },
                intensity: 0.5,
            },
            ..Default::default()
        };

        let report = settings.report();

        assert_eq!(report.missing_resource_count, 2);
        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.lut.texture",
                "effect-stack.lut.texture-layout"
            ])
        );
    }

    fn labels<const N: usize>(items: [&str; N]) -> Vec<String> {
        items.into_iter().map(str::to_string).collect()
    }

    fn texture_descriptor(
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        dimension: RenderImageDimension,
    ) -> RenderImageDescriptor {
        RenderImageDescriptor {
            width,
            height,
            depth_or_array_layers,
            dimension,
            format: "rgba8unorm".to_string(),
            color_space: RenderImageColorSpace::Linear,
            sampler: RenderSamplerDescriptor::default(),
            usage: vec![RenderImageUsage::Sampled],
            asset_usage: Vec::new(),
            mip_count: 1,
            array_layer_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }
}
