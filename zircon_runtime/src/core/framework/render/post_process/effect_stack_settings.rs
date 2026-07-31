mod blur_settings;
mod color_transform_settings;
mod depth_of_field_settings;
mod motion_blur_settings;
mod report;
mod resource_status;
mod screen_space_reflection_settings;
mod style_settings;

pub use blur_settings::RenderBlurSettings;
pub use color_transform_settings::{
    MAX_COLOR_LOOKUP_TEXTURE_SIZE, MIN_COLOR_LOOKUP_TEXTURE_SIZE, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderTonemapOperator, RenderTonemapSettings,
};
pub use depth_of_field_settings::RenderDepthOfFieldSettings;
pub use motion_blur_settings::RenderMotionBlurSettings;
pub use report::RenderPostProcessEffectStackReport;
pub use resource_status::RenderPostProcessEffectStackResourceStatus;
pub use screen_space_reflection_settings::RenderScreenSpaceReflectionSettings;
pub use style_settings::{
    RenderChromaticAberrationSettings, RenderDitherSettings, RenderFilmGrainSettings,
    RenderFogSettings, RenderVignetteSettings,
};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct RenderPostProcessEffectStackSettings {
    pub tonemap: RenderTonemapSettings,
    pub color_lookup: RenderColorLookupSettings,
    pub blur: RenderBlurSettings,
    pub motion_blur: RenderMotionBlurSettings,
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
            || self.blur.is_enabled()
            || self.motion_blur.is_enabled()
            || self.depth_of_field.is_enabled()
            || self.screen_space_reflection.is_enabled()
            || self.vignette.is_enabled()
            || self.grain.is_enabled()
            || self.dither.is_enabled()
            || self.chromatic_aberration.is_enabled()
            || self.fog.is_enabled()
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

#[cfg(test)]
mod tests {
    use super::{
        RenderDitherSettings, RenderPostProcessEffectStackSettings,
        RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
    };

    #[test]
    fn extended_effect_stack_settings_enable_product_node_without_retired_fields() {
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
}
