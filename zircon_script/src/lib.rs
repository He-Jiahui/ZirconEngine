//! VM plugin contracts, host handles, and hot reload coordination.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use zircon_core::{ModuleDescriptor, ServiceObject, StartupMode};
use zircon_module::{dependency_on, factory, qualified_name};
use zircon_manager::{CapabilitySet, HostHandle, PluginSlotId};

pub const SCRIPT_MODULE_NAME: &str = "ScriptModule";
pub const PLUGIN_HOST_DRIVER_NAME: &str = "ScriptModule.Driver.PluginHostDriver";
pub const VM_PLUGIN_MANAGER_NAME: &str = "ScriptModule.Manager.VmPluginManager";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmStateBlob {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub capabilities: CapabilitySet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginPackage {
    pub manifest: VmPluginManifest,
    pub bytecode: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("vm backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("plugin slot missing: {0}")]
    MissingSlot(u64),
    #[error("plugin operation failed: {0}")]
    Operation(String),
    #[error("package parse failed: {0}")]
    Parse(String),
}

pub trait VmPluginInstance: Send + Sync {
    fn manifest(&self) -> &VmPluginManifest;
    fn activate(&mut self, _host: &HostRegistry) -> Result<(), VmError> {
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), VmError> {
        Ok(())
    }
    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        Ok(VmStateBlob::default())
    }
    fn restore_state(&mut self, _state: &VmStateBlob) -> Result<(), VmError> {
        Ok(())
    }
}

pub trait VmBackend: Send + Sync {
    fn backend_name(&self) -> &str;
    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError>;
}

#[derive(Clone, Debug, Default)]
pub struct HostRegistry {
    next_handle: Arc<AtomicU64>,
    handles: Arc<Mutex<HashMap<HostHandle, String>>>,
}

impl HostRegistry {
    pub fn register_capability(&self, label: impl Into<String>) -> HostHandle {
        let handle = HostHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        self.handles.lock().unwrap().insert(handle, label.into());
        handle
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        self.handles.lock().unwrap().contains_key(&handle)
    }
}

#[derive(Debug, Default)]
pub struct PluginHostDriver {
    registry: HostRegistry,
}

impl PluginHostDriver {
    pub fn registry(&self) -> HostRegistry {
        self.registry.clone()
    }
}

pub struct HotReloadCoordinator {
    backend: Arc<dyn VmBackend>,
    host: HostRegistry,
    next_slot: AtomicU64,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
}

struct PluginSlot {
    package: VmPluginPackage,
    instance: Box<dyn VmPluginInstance>,
}

impl fmt::Debug for HotReloadCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadCoordinator")
            .field("backend", &self.backend.backend_name())
            .finish()
    }
}

impl HotReloadCoordinator {
    pub fn new(backend: Arc<dyn VmBackend>, host: HostRegistry) -> Self {
        Self {
            backend,
            host,
            next_slot: AtomicU64::new(1),
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_package(&self, package: VmPluginPackage) -> Result<PluginSlotId, VmError> {
        let mut instance = self.backend.load_package(&package, self.host.clone())?;
        instance.activate(&self.host)?;
        let slot = PluginSlotId::new(self.next_slot.fetch_add(1, Ordering::SeqCst));
        self.slots
            .lock()
            .unwrap()
            .insert(slot, PluginSlot { package, instance });
        Ok(slot)
    }

    pub fn hot_reload(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError> {
        let mut slots = self.slots.lock().unwrap();
        let state = {
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let state = slot_entry.instance.save_state()?;
            slot_entry.instance.deactivate()?;
            state
        };

        let mut next_instance = self.backend.load_package(&package, self.host.clone())?;
        next_instance.activate(&self.host)?;
        next_instance.restore_state(&state)?;

        slots.insert(
            slot,
            PluginSlot {
                package,
                instance: next_instance,
            },
        );
        Ok(())
    }

    pub fn manifest(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        let slots = self.slots.lock().unwrap();
        let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
        Ok(slot_entry.package.manifest.clone())
    }
}

#[derive(Debug, Default)]
pub struct UnavailableVmBackend;

impl VmBackend for UnavailableVmBackend {
    fn backend_name(&self) -> &str {
        "unavailable"
    }

    fn load_package(
        &self,
        _package: &VmPluginPackage,
        _host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Err(VmError::BackendUnavailable(
            "zr_vm integration is not wired yet".to_string(),
        ))
    }
}

#[derive(Debug, Default)]
pub struct MockVmBackend;

impl VmBackend for MockVmBackend {
    fn backend_name(&self) -> &str {
        "mock"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(MockVmPluginInstance {
            manifest: package.manifest.clone(),
            state: VmStateBlob::default(),
            activations: 0,
        }))
    }
}

#[derive(Debug)]
struct MockVmPluginInstance {
    manifest: VmPluginManifest,
    state: VmStateBlob,
    activations: usize,
}

impl VmPluginInstance for MockVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &HostRegistry) -> Result<(), VmError> {
        self.activations += 1;
        Ok(())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        Ok(self.state.clone())
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        self.state = state.clone();
        Ok(())
    }
}

#[derive(Debug)]
pub struct VmPluginManager {
    coordinator: HotReloadCoordinator,
}

impl VmPluginManager {
    pub fn unavailable() -> Self {
        let host = HostRegistry::default();
        Self {
            coordinator: HotReloadCoordinator::new(Arc::new(UnavailableVmBackend), host),
        }
    }

    pub fn mock() -> Self {
        let host = HostRegistry::default();
        Self {
            coordinator: HotReloadCoordinator::new(Arc::new(MockVmBackend), host),
        }
    }

    pub fn coordinator(&self) -> &HotReloadCoordinator {
        &self.coordinator
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SCRIPT_MODULE_NAME, "VM plugin hosting and hot reload")
        .with_driver(zircon_core::DriverDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Driver,
                "PluginHostDriver",
            ),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(PluginHostDriver::default()) as ServiceObject)),
        ))
        .with_manager(zircon_core::ManagerDescriptor::new(
            qualified_name(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Manager,
                "VmPluginManager",
            ),
            StartupMode::Immediate,
            vec![dependency_on(
                SCRIPT_MODULE_NAME,
                zircon_core::ServiceKind::Driver,
                "PluginHostDriver",
            )],
            factory(|_| Ok(Arc::new(VmPluginManager::unavailable()) as ServiceObject)),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_handles_are_stable_and_valid() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("RenderingManager");
        assert!(registry.is_valid(handle));
    }

    #[test]
    fn mock_backend_hot_reload_roundtrip() {
        let coordinator =
            HotReloadCoordinator::new(Arc::new(MockVmBackend), HostRegistry::default());
        let slot = coordinator
            .load_package(VmPluginPackage {
                manifest: VmPluginManifest {
                    name: "sample".to_string(),
                    version: "0.1.0".to_string(),
                    entry: "main".to_string(),
                    capabilities: CapabilitySet::default().with("render"),
                },
                bytecode: vec![1, 2, 3],
            })
            .unwrap();
        coordinator
            .hot_reload(
                slot,
                VmPluginPackage {
                    manifest: VmPluginManifest {
                        name: "sample".to_string(),
                        version: "0.2.0".to_string(),
                        entry: "main".to_string(),
                        capabilities: CapabilitySet::default().with("render"),
                    },
                    bytecode: vec![4, 5, 6],
                },
            )
            .unwrap();

        let manifest = coordinator.manifest(slot).unwrap();
        assert_eq!(manifest.version, "0.2.0");
    }

    #[test]
    fn unavailable_backend_reports_error() {
        let backend = UnavailableVmBackend;
        let error = match backend.load_package(
            &VmPluginPackage {
                manifest: VmPluginManifest {
                    name: "sample".to_string(),
                    version: "0.1.0".to_string(),
                    entry: "main".to_string(),
                    capabilities: CapabilitySet::default(),
                },
                bytecode: Vec::new(),
            },
            HostRegistry::default(),
        ) {
            Ok(_) => panic!("expected unavailable backend to reject package"),
            Err(error) => error,
        };
        assert!(matches!(error, VmError::BackendUnavailable(_)));
    }
}
