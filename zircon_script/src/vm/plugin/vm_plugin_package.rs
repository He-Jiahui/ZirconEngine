use serde::{Deserialize, Serialize};

use super::vm_plugin_manifest::VmPluginManifest;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginPackage {
    pub manifest: VmPluginManifest,
    pub bytecode: Vec<u8>,
}
