use std::fmt;
use std::sync::Arc;

use crate::core::PluginContext;

use super::super::{CapabilitySet, HostExportRegistry, HostRegistry, VmPluginPackageSource};
use super::VmPluginSlotLifecycle;

#[derive(Clone)]
pub struct VmPluginHostContext {
    pub plugin: PluginContext,
    pub capabilities: CapabilitySet,
    pub backend_selector: String,
    pub package_source: VmPluginPackageSource,
    pub host_registry: HostRegistry,
    pub host_exports: HostExportRegistry,
    pub slot_lifecycle: Arc<dyn VmPluginSlotLifecycle>,
}

impl fmt::Debug for VmPluginHostContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmPluginHostContext")
            .field("plugin", &self.plugin)
            .field("capabilities", &self.capabilities)
            .field("backend_selector", &self.backend_selector)
            .field("package_source", &self.package_source)
            .field("host_registry", &self.host_registry)
            .field("host_exports", &self.host_exports)
            .field("slot_lifecycle", &"<dyn VmPluginSlotLifecycle>")
            .finish()
    }
}
