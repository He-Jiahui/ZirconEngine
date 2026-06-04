use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmPluginSlotState {
    #[default]
    Active,
    Reloading,
    Failed,
}

impl VmPluginSlotState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Reloading => "reloading",
            Self::Failed => "failed",
        }
    }
}
