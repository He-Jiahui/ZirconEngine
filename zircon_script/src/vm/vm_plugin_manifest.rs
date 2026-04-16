use serde::{Deserialize, Serialize};
use zircon_manager::CapabilitySet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub capabilities: CapabilitySet,
}
