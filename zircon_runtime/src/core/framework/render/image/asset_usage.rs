use serde::{Deserialize, Serialize};

/// Where image asset data is expected to stay resident after render preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageAssetUsage {
    MainWorld,
    RenderWorld,
}
