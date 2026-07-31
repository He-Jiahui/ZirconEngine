use crate::core::math::Real;

const DEFAULT_SSR_TEMPORAL_BLEND_FACTOR: Real = 0.18;
const MIN_SSR_ROUGHNESS_MIP_BIAS: Real = -1.0;
const MAX_SSR_ROUGHNESS_MIP_BIAS: Real = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderScreenSpaceReflectionSettings {
    pub intensity: Real,
    pub thickness: Real,
    pub max_ray_distance: Real,
    pub max_steps: u32,
    pub temporal_blend_factor: Real,
    pub roughness_mip_bias: Real,
}

impl Default for RenderScreenSpaceReflectionSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            thickness: 0.1,
            max_ray_distance: 50.0,
            max_steps: 64,
            temporal_blend_factor: DEFAULT_SSR_TEMPORAL_BLEND_FACTOR,
            roughness_mip_bias: 0.0,
        }
    }
}

impl RenderScreenSpaceReflectionSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0 && self.max_steps > 0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }

    pub fn render_thickness(self) -> Real {
        self.thickness.max(0.0)
    }

    pub fn render_max_ray_distance(self) -> Real {
        self.max_ray_distance.max(0.0)
    }

    pub fn render_temporal_blend_factor(self) -> Real {
        self.temporal_blend_factor.clamp(0.0, 1.0)
    }

    pub fn render_roughness_mip_bias(self) -> Real {
        self.roughness_mip_bias
            .clamp(MIN_SSR_ROUGHNESS_MIP_BIAS, MAX_SSR_ROUGHNESS_MIP_BIAS)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SSR_ROUGHNESS_MIP_BIAS, MIN_SSR_ROUGHNESS_MIP_BIAS, RenderScreenSpaceReflectionSettings,
    };

    #[test]
    fn screen_space_reflection_settings_sanitize_renderer_upload_values() {
        let settings = RenderScreenSpaceReflectionSettings {
            intensity: -0.5,
            thickness: -0.1,
            max_ray_distance: -12.0,
            temporal_blend_factor: 2.0,
            roughness_mip_bias: 5.0,
            ..Default::default()
        };

        assert_eq!(settings.render_intensity(), 0.0);
        assert_eq!(settings.render_thickness(), 0.0);
        assert_eq!(settings.render_max_ray_distance(), 0.0);
        assert_eq!(settings.render_temporal_blend_factor(), 1.0);
        assert_eq!(
            settings.render_roughness_mip_bias(),
            MAX_SSR_ROUGHNESS_MIP_BIAS
        );

        let settings = RenderScreenSpaceReflectionSettings {
            roughness_mip_bias: -5.0,
            ..Default::default()
        };

        assert_eq!(
            settings.render_roughness_mip_bias(),
            MIN_SSR_ROUGHNESS_MIP_BIAS
        );
    }
}
