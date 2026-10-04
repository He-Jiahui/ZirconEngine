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
    RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderTonemapOperator,
    RenderTonemapSettings, MAX_COLOR_LOOKUP_TEXTURE_SIZE, MIN_COLOR_LOOKUP_TEXTURE_SIZE,
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

/// 体积求值和相机提取共享的创作参数集合；启用谓词决定后处理栈是否安排效果族。
/// 资源是否已绑定由运行时状态报告，不能把此值对象当作 GPU 资源就绪证明。
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
#[path = "tests/effect_stack_settings.rs"]
mod tests;
