use crate::core::framework::render::{
    AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile,
};
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessSettingsComponent {
    pub ambient_occlusion: AoSourceSettings,
    pub bloom: RenderBloomSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
}

impl Default for PostProcessSettingsComponent {
    fn default() -> Self {
        Self {
            ambient_occlusion: AoSourceSettings::default(),
            bloom: RenderBloomSettings::default(),
            color_grading: RenderColorGradingSettings::default(),
            effect_stack: RenderPostProcessEffectStackSettings::default(),
        }
    }
}

impl PostProcessSettingsComponent {
    pub fn from_parts(
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        Self {
            ambient_occlusion: AoSourceSettings::default(),
            bloom,
            color_grading,
            effect_stack,
        }
    }

    pub const fn with_ambient_occlusion(mut self, ambient_occlusion: AoSourceSettings) -> Self {
        self.ambient_occlusion = ambient_occlusion;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessVolumeComponent {
    pub active: bool,
    pub is_global: bool,
    pub priority: Real,
    pub weight: Real,
    /// Scene-space fade distance outside a local volume collider.
    /// Global volumes ignore this field.
    pub blend_distance: Real,
    pub profile: RenderPostProcessVolumeProfile,
}

impl Default for PostProcessVolumeComponent {
    fn default() -> Self {
        Self {
            active: true,
            is_global: true,
            priority: 0.0,
            weight: 1.0,
            blend_distance: 0.0,
            profile: RenderPostProcessVolumeProfile::default(),
        }
    }
}

impl PostProcessVolumeComponent {
    pub fn global(priority: Real, profile: RenderPostProcessVolumeProfile) -> Self {
        Self {
            priority,
            profile,
            ..Self::default()
        }
    }

    pub fn local(
        priority: Real,
        weight: Real,
        blend_distance: Real,
        profile: RenderPostProcessVolumeProfile,
    ) -> Self {
        Self {
            is_global: false,
            priority,
            weight,
            blend_distance,
            profile,
            ..Self::default()
        }
    }

    pub const fn with_weight(mut self, weight: Real) -> Self {
        self.weight = weight;
        self
    }
}
