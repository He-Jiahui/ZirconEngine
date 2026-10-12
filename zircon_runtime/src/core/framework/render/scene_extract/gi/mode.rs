use serde::{Deserialize, Serialize};

/// 区分纯动态照明与烘焙加动态照明；后者须在解析时检查烘焙数据可用性。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHybridGiMode {
    #[default]
    DynamicOnly,
    BakedStaticDynamic,
}

impl RenderHybridGiMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DynamicOnly => "dynamic-only",
            Self::BakedStaticDynamic => "baked-static-dynamic",
        }
    }
}
