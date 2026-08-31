use serde::{Deserialize, Serialize};

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
