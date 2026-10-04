//! 闭合材质夹具同时约束项目生成、缓存复用和管线证据。

/// 同一诊断材质的生成、复用校验与管线准入身份；变体用独立项目身份避免错误复用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ViewerMaterialFixture {
    #[default]
    MetalMirror,
    DielectricIor,
}

impl ViewerMaterialFixture {
    pub(crate) const fn cli_value(self) -> &'static str {
        match self {
            Self::MetalMirror => "metal-mirror",
            Self::DielectricIor => "dielectric-ior",
        }
    }

    pub(crate) const fn project_root_component(self) -> Option<&'static str> {
        match self {
            // Keep the original project path stable for the established mirror baseline.
            Self::MetalMirror => None,
            Self::DielectricIor => Some("dielectric-ior"),
        }
    }

    pub(crate) fn from_cli_value(value: &str) -> Result<Self, String> {
        match value {
            "metal-mirror" => Ok(Self::MetalMirror),
            "dielectric-ior" => Ok(Self::DielectricIor),
            _ => Err(format!(
                "--material-fixture must be metal-mirror or dielectric-ior, got {value}"
            )),
        }
    }

    /// 非默认介质反射参数需要通用前向路径，不能使用镜面专用预热结果证明其就绪。
    pub(crate) const fn requires_generic_forward_pipeline(self) -> bool {
        matches!(self, Self::DielectricIor)
    }

    pub(crate) const fn project_asset_identity_prefix(self) -> &'static str {
        match self {
            Self::MetalMirror => "viewer-project-v4",
            Self::DielectricIor => "viewer-project-v4/dielectric-ior",
        }
    }

    pub(crate) const fn material_name(self) -> &'static str {
        match self {
            Self::MetalMirror => "Interactive Perfect Mirror Sphere",
            Self::DielectricIor => "Interactive Dielectric IOR 2.0 Sphere",
        }
    }

    pub(crate) const fn base_color(self) -> [f32; 4] {
        match self {
            Self::MetalMirror => [1.0, 1.0, 1.0, 1.0],
            Self::DielectricIor => [0.86, 0.9, 1.0, 1.0],
        }
    }

    pub(crate) const fn metallic(self) -> f32 {
        match self {
            Self::MetalMirror => 1.0,
            Self::DielectricIor => 0.0,
        }
    }

    pub(crate) const fn roughness(self) -> f32 {
        match self {
            Self::MetalMirror => 0.0,
            Self::DielectricIor => 0.08,
        }
    }

    pub(crate) const fn dielectric_ior(self) -> Option<f64> {
        match self {
            Self::MetalMirror => None,
            Self::DielectricIor => Some(2.0),
        }
    }
}

#[cfg(test)]
#[path = "tests/material_fixture.rs"]
mod tests;
