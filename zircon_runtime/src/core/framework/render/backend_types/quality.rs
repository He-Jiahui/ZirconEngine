use super::super::{ShaderQualityTier, SolariSettings, TaaQualityPreset};
use super::handles::RenderPipelineHandle;

pub const DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA: u16 = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderFeatureQualitySettings {
    pub clustered_lighting: bool,
    pub screen_space_ambient_occlusion: bool,
    pub temporal_history: bool,
    pub bloom: bool,
    pub color_grading: bool,
    pub anti_alias: bool,
    pub reflection_probes: bool,
    pub baked_lighting: bool,
    pub particle_rendering: bool,
    pub virtual_geometry: bool,
    pub hybrid_global_illumination: bool,
    pub solari: bool,
    /// Routes eligible transparent rendering through the half-resolution graph path.
    /// Disabled by default because it is a bandwidth-quality tradeoff.
    pub half_resolution_transparency: bool,
    pub allow_async_compute: bool,
}

impl Default for RenderFeatureQualitySettings {
    fn default() -> Self {
        Self {
            clustered_lighting: true,
            screen_space_ambient_occlusion: false,
            temporal_history: false,
            bloom: true,
            color_grading: true,
            anti_alias: true,
            reflection_probes: true,
            baked_lighting: true,
            particle_rendering: true,
            virtual_geometry: false,
            hybrid_global_illumination: false,
            solari: false,
            half_resolution_transparency: false,
            allow_async_compute: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderQualityProfile {
    pub name: String,
    pub pipeline_override: Option<RenderPipelineHandle>,
    pub features: RenderFeatureQualitySettings,
    /// Positive LOD offset applied to every material texture sample for this viewport.
    pub texture_mip_bias: u8,
    /// Upper bound for material texture anisotropy in this viewport.
    pub texture_max_anisotropy: u8,
    /// Depth-discontinuity falloff for half-resolution transparency upsampling.
    pub half_resolution_transparency_depth_sigma: u16,
    pub shader_quality: ShaderQualityTier,
    pub taa_quality: TaaQualityPreset,
    pub solari: SolariSettings,
}

impl RenderQualityProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pipeline_override: None,
            features: RenderFeatureQualitySettings::default(),
            texture_mip_bias: 0,
            texture_max_anisotropy: 16,
            half_resolution_transparency_depth_sigma: DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            shader_quality: ShaderQualityTier::default(),
            taa_quality: TaaQualityPreset::default(),
            solari: SolariSettings::default(),
        }
    }

    pub fn with_pipeline_asset(mut self, pipeline: RenderPipelineHandle) -> Self {
        self.pipeline_override = Some(pipeline);
        self
    }

    pub fn with_clustered_lighting(mut self, enabled: bool) -> Self {
        self.features.clustered_lighting = enabled;
        self
    }

    pub fn with_screen_space_ambient_occlusion(mut self, enabled: bool) -> Self {
        self.features.screen_space_ambient_occlusion = enabled;
        self
    }

    pub fn with_temporal_history(mut self, enabled: bool) -> Self {
        self.features.temporal_history = enabled;
        self
    }

    pub fn with_bloom(mut self, enabled: bool) -> Self {
        self.features.bloom = enabled;
        self
    }

    pub fn with_color_grading(mut self, enabled: bool) -> Self {
        self.features.color_grading = enabled;
        self
    }

    pub fn with_anti_alias(mut self, enabled: bool) -> Self {
        self.features.anti_alias = enabled;
        self
    }

    pub fn with_texture_mip_bias(mut self, mip_bias: u8) -> Self {
        self.texture_mip_bias = mip_bias;
        self
    }

    pub fn with_texture_max_anisotropy(mut self, max_anisotropy: u8) -> Self {
        self.texture_max_anisotropy = normalize_texture_max_anisotropy(max_anisotropy);
        self
    }

    pub fn with_shader_quality(mut self, quality: ShaderQualityTier) -> Self {
        self.shader_quality = quality;
        self
    }

    pub fn with_taa_quality(mut self, quality: TaaQualityPreset) -> Self {
        self.taa_quality = quality;
        self
    }

    pub fn with_reflection_probes(mut self, enabled: bool) -> Self {
        self.features.reflection_probes = enabled;
        self
    }

    pub fn with_baked_lighting(mut self, enabled: bool) -> Self {
        self.features.baked_lighting = enabled;
        self
    }

    pub fn with_particle_rendering(mut self, enabled: bool) -> Self {
        self.features.particle_rendering = enabled;
        self
    }

    pub fn with_virtual_geometry(mut self, enabled: bool) -> Self {
        self.features.virtual_geometry = enabled;
        self
    }

    pub fn with_hybrid_global_illumination(mut self, enabled: bool) -> Self {
        self.features.hybrid_global_illumination = enabled;
        self
    }

    pub fn with_solari(mut self, enabled: bool) -> Self {
        self.features.solari = enabled;
        self
    }

    pub fn with_solari_settings(mut self, settings: SolariSettings) -> Self {
        self.solari = settings;
        self
    }

    pub fn with_solari_experimental_enabled(mut self, enabled: bool) -> Self {
        self.solari = if enabled {
            SolariSettings::experimental_enabled()
        } else {
            SolariSettings::default()
        };
        self
    }

    pub fn with_half_resolution_transparency(mut self, enabled: bool) -> Self {
        self.features.half_resolution_transparency = enabled;
        self
    }

    pub fn with_half_resolution_transparency_depth_sigma(mut self, sigma: u16) -> Self {
        self.half_resolution_transparency_depth_sigma = sigma.max(1);
        self
    }

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.features.allow_async_compute = enabled;
        self
    }
}

pub(crate) const fn normalize_texture_max_anisotropy(max_anisotropy: u8) -> u8 {
    match max_anisotropy {
        16.. => 16,
        8.. => 8,
        4.. => 4,
        2.. => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderFeatureQualitySettings, RenderQualityProfile};

    #[test]
    fn ssao_is_fail_closed_until_the_quality_profile_explicitly_enables_it() {
        assert!(!RenderFeatureQualitySettings::default().screen_space_ambient_occlusion);
        assert!(
            !RenderQualityProfile::new("default")
                .features
                .screen_space_ambient_occlusion
        );
        assert!(
            RenderQualityProfile::new("explicit-ssao")
                .with_screen_space_ambient_occlusion(true)
                .features
                .screen_space_ambient_occlusion
        );
    }
}
