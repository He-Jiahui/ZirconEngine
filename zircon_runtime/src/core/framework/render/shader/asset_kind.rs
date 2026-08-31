use serde::{Deserialize, Serialize};

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
