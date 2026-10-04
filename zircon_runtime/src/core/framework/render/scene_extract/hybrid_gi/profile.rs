use serde::{Deserialize, Serialize};

/// 预设只给出模式和预算基线；自定义字段仍可覆盖非零预算。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHybridGiProfile {
    FullyDynamic,
    IndoorStatic,
    OpenWorld,
    Cinematic,
    #[default]
    Custom,
}

impl RenderHybridGiProfile {
    pub const fn label(self) -> &'static str {
        match self {
            Self::FullyDynamic => "fully-dynamic",
            Self::IndoorStatic => "indoor-static",
            Self::OpenWorld => "open-world",
            Self::Cinematic => "cinematic",
            Self::Custom => "custom",
        }
    }
}
