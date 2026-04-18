use crate::{PluginSlotId, VmPluginManifest, VmPluginPackageSource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmPluginSlotRecord {
    pub slot: PluginSlotId,
    pub backend_name: String,
    pub source: VmPluginPackageSource,
    pub manifest: VmPluginManifest,
}
