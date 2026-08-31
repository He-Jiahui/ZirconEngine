use crate::core::math::Vec4;

use super::super::super::{EnvironmentExtract, FallbackSkyboxKind, SkyboxMode};

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewEnvironmentExtract {
    pub lighting_enabled: bool,
    pub skybox_enabled: bool,
    pub fallback_skybox: FallbackSkyboxKind,
    pub clear_color: Vec4,
}

impl PreviewEnvironmentExtract {
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
