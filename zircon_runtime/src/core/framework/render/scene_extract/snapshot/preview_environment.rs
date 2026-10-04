use crate::core::math::Vec4;

use super::super::super::{EnvironmentExtract, FallbackSkyboxKind, SkyboxMode};

/// 视口预览开关与清屏偏好；环境资源本身仍由 EnvironmentExtract 提供。
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewEnvironmentExtract {
    pub lighting_enabled: bool,
    pub skybox_enabled: bool,
    pub fallback_skybox: FallbackSkyboxKind,
    pub clear_color: Vec4,
}

impl PreviewEnvironmentExtract {
    /// 从环境模式推导预览后备天空盒；源立方体图由环境路径处理，不在此模拟。
    pub fn from_environment(
        environment: &EnvironmentExtract,
        lighting_enabled: bool,
        clear_color: Vec4,
    ) -> Self {
        Self {
            lighting_enabled,
            skybox_enabled: environment.skybox_enabled(),
            fallback_skybox: match environment.skybox.mode {
                SkyboxMode::Disabled => FallbackSkyboxKind::None,
                SkyboxMode::ProceduralGradient => FallbackSkyboxKind::ProceduralGradient,
                SkyboxMode::SourceCubemap => FallbackSkyboxKind::None,
            },
            clear_color,
        }
    }
}
