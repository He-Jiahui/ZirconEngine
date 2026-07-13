use std::fmt;
use std::sync::Arc;

use crate::core::PluginContext;

use super::super::{
    CapabilitySet, HostExportRegistry, HostRegistry, PluginSlotId, VmHostInterfaceError,
    VmHostInterfaceRegistry, VmInterfaceCaller, VmPluginPackageSource,
};
use super::VmPluginSlotLifecycle;

#[derive(Clone)]
pub struct VmPluginHostContext {
    pub plugin: PluginContext,
    pub capabilities: CapabilitySet,
    pub backend_selector: String,
    pub package_source: VmPluginPackageSource,
    pub host_registry: HostRegistry,
    pub host_exports: HostExportRegistry,
    pub host_interfaces: VmHostInterfaceRegistry,
    pub slot_lifecycle: Arc<dyn VmPluginSlotLifecycle>,
    /// Assigned by the coordinator before backend load/activation. Backends use
    /// this identity when a VM package registers host extension callbacks.
    pub vm_owner: Option<(PluginSlotId, u32)>,
}

impl VmPluginHostContext {
    pub fn with_vm_owner(&self, slot: PluginSlotId, generation: u32) -> Self {
        let mut context = self.clone();
        context.vm_owner = Some((slot, generation));
        context
    }

    pub fn interface_caller(&self) -> Result<VmInterfaceCaller, VmHostInterfaceError> {
        let (slot, generation) = self.vm_owner.ok_or(VmHostInterfaceError::MissingCaller)?;
        Ok(VmInterfaceCaller::new(
            slot,
            generation,
            self.capabilities.clone(),
        ))
    }

    pub fn vm_owner(&self) -> Option<(PluginSlotId, u32)> {
        self.vm_owner
    }
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
            .field("host_interfaces", &self.host_interfaces)
            .field("slot_lifecycle", &"<dyn VmPluginSlotLifecycle>")
            .field("vm_owner", &self.vm_owner)
            .finish()
    }
}
