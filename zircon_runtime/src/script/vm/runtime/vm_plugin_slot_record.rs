use crate::script::{
    PluginSlotId, VmPluginManagementPolicy, VmPluginManifest, VmPluginPackageSource,
};

use super::vm_plugin_slot_state::VmPluginSlotState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmPluginSlotRecord {
    pub slot: PluginSlotId,
    pub backend_name: String,
    pub state: VmPluginSlotState,
    pub generation: u32,
    pub source: VmPluginPackageSource,
    pub manifest: VmPluginManifest,
    pub management: VmPluginManagementPolicy,
}
