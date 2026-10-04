use serde::{Deserialize, Serialize};

/// 资产导入与模板组装共用的类型判别；只有 Surface 参与材质变体，Include 仅作为模块依赖。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderAssetKind {
    Module,
    Surface,
    Include,
    Compute,
    Fullscreen,
}

impl ShaderAssetKind {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Surface => "surface",
            Self::Include => "include",
            Self::Compute => "compute",
            Self::Fullscreen => "fullscreen",
        }
    }

    pub const fn participates_in_material_variants(self) -> bool {
        matches!(self, Self::Surface)
    }

    pub const fn is_include(self) -> bool {
        matches!(self, Self::Include)
    }
}
