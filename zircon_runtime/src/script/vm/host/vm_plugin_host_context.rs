use std::fmt;
use std::sync::{Arc, RwLock};

use crate::core::PluginContext;

use super::super::{
    CapabilitySet, HostExportRegistry, HostRegistry, PluginSlotId, VmHostInterfaceError,
    VmHostInterfaceRegistry, VmInterfaceCaller, VmPluginPackageSource, VmReflectionCatalog,
    VmReflectionRegistrySnapshot,
};
use super::VmPluginSlotLifecycle;

type ReflectionSchemaInstallCallback =
    dyn Fn(&VmReflectionRegistrySnapshot) -> Result<(), super::super::VmError> + Send + Sync;

/// Package-local hook installed by a concrete backend to consume the coordinator's single schema read.
#[derive(Clone, Default)]
pub struct VmReflectionSchemaInstaller {
    callback: Arc<RwLock<Option<Arc<ReflectionSchemaInstallCallback>>>>,
}

impl VmReflectionSchemaInstaller {
    pub fn register(
        &self,
        callback: impl Fn(&VmReflectionRegistrySnapshot) -> Result<(), super::super::VmError>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), super::super::VmError> {
        let mut current = self
            .callback
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if current.is_some() {
            return Err(super::super::VmError::Operation(
                "VM reflection schema installer already registered for this package load"
                    .to_string(),
            ));
        }
        *current = Some(Arc::new(callback));
        Ok(())
    }

    fn install(
        &self,
        snapshot: &VmReflectionRegistrySnapshot,
    ) -> Result<(), super::super::VmError> {
        let callback = self
            .callback
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        match callback {
            Some(callback) => callback(snapshot),
            None => Ok(()),
        }
    }
}

impl fmt::Debug for VmReflectionSchemaInstaller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VmReflectionSchemaInstaller")
            .field(
                "registered",
                &self
                    .callback
                    .read()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .is_some(),
            )
            .finish()
    }
}

#[derive(Clone)]
pub struct VmPluginHostContext {
    pub plugin: PluginContext,
    pub capabilities: CapabilitySet,
    pub backend_selector: String,
    pub package_source: VmPluginPackageSource,
    pub host_registry: HostRegistry,
    pub host_exports: HostExportRegistry,
    pub host_interfaces: VmHostInterfaceRegistry,
    /// Shared catalog receiving public VM-owned reflected component schemas.
    pub reflection_catalog: VmReflectionCatalog,
    /// Concrete-backend hook that compiles the exact schema snapshot read by the coordinator.
    pub reflection_schema_installer: VmReflectionSchemaInstaller,
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

    pub fn install_reflection_schema(
        &self,
        snapshot: &VmReflectionRegistrySnapshot,
    ) -> Result<(), super::super::VmError> {
        self.vm_owner.ok_or_else(|| {
            super::super::VmError::Operation(
                "VM reflection schema install requires a package slot owner".to_string(),
            )
        })?;
        self.reflection_schema_installer.install(snapshot)
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
            .field("reflection_catalog", &self.reflection_catalog)
            .field(
                "reflection_schema_installer",
                &self.reflection_schema_installer,
            )
            .field("slot_lifecycle", &"<dyn VmPluginSlotLifecycle>")
            .field("vm_owner", &self.vm_owner)
            .finish()
    }
}
