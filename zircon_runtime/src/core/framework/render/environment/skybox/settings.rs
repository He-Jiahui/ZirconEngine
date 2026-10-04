use crate::core::math::Real;

use super::{IblBakeKey, ProceduralSkyParams, SkyboxMode, SourceCubemapEnvironment};

/// 一帧天空模式的入口；场景提取、IBL 请求和渲染器都从当前模式读取同一数据。
/// 改变模式时应使用构造器或同步更新对应 payload，避免模式与可选源图分离。
#[derive(Clone, Debug, PartialEq)]
pub struct SkyboxSettings {
    pub mode: SkyboxMode,
    pub procedural: ProceduralSkyParams,
    pub source_cubemap: Option<SourceCubemapEnvironment>,
}

impl SkyboxSettings {
    pub fn none() -> Self {
        Self {
            mode: SkyboxMode::Disabled,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: None,
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            mode: SkyboxMode::ProceduralGradient,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: None,
        }
    }

    /// 把源图转入天空模式时预先编码 GPU 上传内容，使帧提交只处理资源变更。
    pub fn source_cubemap(source_cubemap: SourceCubemapEnvironment) -> Self {
        Self {
            mode: SkyboxMode::SourceCubemap,
            procedural: ProceduralSkyParams::default_gradient(),
            source_cubemap: Some(source_cubemap.with_prepared_upload_artifact()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self.mode, SkyboxMode::Disabled)
    }

    pub fn intensity(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.intensity,
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(|environment| environment.intensity)
                .unwrap_or(0.0),
        }
    }

    pub fn rotation_radians(&self) -> Real {
        match self.mode {
            SkyboxMode::Disabled => 0.0,
            SkyboxMode::ProceduralGradient => self.procedural.rotation_radians,
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(|environment| environment.rotation_radians)
                .unwrap_or(0.0),
        }
    }

    pub fn ibl_bake_key(&self) -> Option<IblBakeKey> {
        match self.mode {
            SkyboxMode::Disabled => None,
            SkyboxMode::ProceduralGradient => Some(self.procedural.ibl_bake_key()),
            SkyboxMode::SourceCubemap => self
                .source_cubemap
                .as_ref()
                .map(SourceCubemapEnvironment::ibl_bake_key),
        }
    }

    pub fn source_cubemap_environment(&self) -> Option<&SourceCubemapEnvironment> {
        match self.mode {
            SkyboxMode::SourceCubemap => self.source_cubemap.as_ref(),
            SkyboxMode::Disabled | SkyboxMode::ProceduralGradient => None,
        }
    }
}

impl Default for SkyboxSettings {
    fn default() -> Self {
        Self::none()
    }
}
