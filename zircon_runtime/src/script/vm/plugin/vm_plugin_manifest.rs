use super::super::CapabilitySet;
use super::management_policy::VmPluginManagementPolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub capabilities: CapabilitySet,
    #[serde(default)]
    pub management: VmPluginManagementPolicy,
}
