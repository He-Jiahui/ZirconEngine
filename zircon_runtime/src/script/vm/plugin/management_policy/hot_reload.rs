use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmPluginHotReloadPolicy {
    Disabled,
    Stateless,
    #[default]
    PreserveState,
}
