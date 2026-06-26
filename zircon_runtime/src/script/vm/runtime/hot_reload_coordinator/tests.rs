use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{CoreRuntime, PluginContext};
use crate::script::{
    CapabilitySet, HostExportRegistry, HostRegistry, VmBackend, VmPluginHostContext,
    VmPluginManifest, VmPluginPackage, VmPluginPackageSource, VmPluginSlotLifecycle,
};

use super::*;

#[derive(Debug)]
struct PolicyRecordingBackend {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmBackend for PolicyRecordingBackend {
    fn backend_name(&self) -> &str {
        "policy-recording"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.events.lock().unwrap().push("load");
        Ok(Box::new(PolicyRecordingInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
}

#[derive(Debug)]
struct PolicyRecordingInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmPluginInstance for PolicyRecordingInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        self.events.lock().unwrap().push("activate");
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        self.events.lock().unwrap().push("deactivate");
        Ok(())
    }

    fn save_state(&mut self) -> Result<crate::script::VmStateBlob, VmError> {
        self.events.lock().unwrap().push("save_state");
        Ok(crate::script::VmStateBlob {
            bytes: b"saved".to_vec(),
        })
    }

    fn restore_state(&mut self, _state: &crate::script::VmStateBlob) -> Result<(), VmError> {
        self.events.lock().unwrap().push("restore_state");
        Ok(())
    }
}

#[derive(Debug)]
struct LifecycleQueryBackend {
    events: Arc<Mutex<Vec<String>>>,
}

impl VmBackend for LifecycleQueryBackend {
    fn backend_name(&self) -> &str {
        "lifecycle-query"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.events.lock().unwrap().push("load".to_string());
        Ok(Box::new(LifecycleQueryInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
}

#[derive(Debug)]
struct LifecycleQueryInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<String>>>,
}

impl VmPluginInstance for LifecycleQueryInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, host: &VmPluginHostContext) -> Result<(), VmError> {
        let records = host.slot_lifecycle.list_slots();
        let event = if records
            .iter()
            .any(|record| record.state == VmPluginSlotState::Reloading)
        {
            "activate_query_reloading"
        } else {
            "activate_query_empty"
        };
        self.events.lock().unwrap().push(event.to_string());
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        self.events.lock().unwrap().push("deactivate".to_string());
        Ok(())
    }

    fn restore_state(&mut self, _state: &crate::script::VmStateBlob) -> Result<(), VmError> {
        self.events
            .lock()
            .unwrap()
            .push("restore_state_query".to_string());
        Ok(())
    }
}

#[derive(Debug, Default)]
struct NoopSlotLifecycle;

impl VmPluginSlotLifecycle for NoopSlotLifecycle {
    fn load_package(
        &self,
        _backend_selector: &str,
        _package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        Err(VmError::Operation("noop lifecycle cannot load".to_string()))
    }

    fn hot_reload_slot(
        &self,
        slot: PluginSlotId,
        _package: VmPluginPackage,
    ) -> Result<(), VmError> {
        Err(VmError::MissingSlot(slot.get()))
    }

    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        Err(VmError::MissingSlot(slot.get()))
    }

    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        Err(VmError::MissingSlot(slot.get()))
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        Vec::new()
    }
}

#[derive(Debug)]
struct CoordinatorSlotLifecycle {
    coordinator: Arc<HotReloadCoordinator>,
}

impl VmPluginSlotLifecycle for CoordinatorSlotLifecycle {
    fn load_package(
        &self,
        _backend_selector: &str,
        _package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        Err(VmError::Operation(
            "test coordinator lifecycle cannot load packages".to_string(),
        ))
    }

    fn hot_reload_slot(
        &self,
        slot: PluginSlotId,
        _package: VmPluginPackage,
    ) -> Result<(), VmError> {
        Err(VmError::Operation(format!(
            "test coordinator lifecycle cannot hot reload slot {}",
            slot.get()
        )))
    }

    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        Err(VmError::Operation(format!(
            "test coordinator lifecycle cannot unload slot {}",
            slot.get()
        )))
    }

    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        self.coordinator.slot(slot)
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        self.coordinator.list_slots()
    }
}

#[test]
fn hot_reload_policy_preserves_state_and_increments_generation_by_default() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = PolicyRecordingBackend {
        events: Arc::clone(&events),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();

    let slot = coordinator
        .load_package("policy-recording", &backend, test_package("0.1.0"), &host)
        .unwrap();
    assert_eq!(coordinator.slot(slot).unwrap().generation, 1);

    coordinator
        .hot_reload(
            slot,
            "policy-recording",
            &backend,
            test_package("0.2.0"),
            &host,
        )
        .unwrap();

    let record = coordinator.slot(slot).unwrap();
    assert_eq!(record.generation, 2);
    assert_eq!(record.state, VmPluginSlotState::Active);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &[
            "load",
            "activate",
            "save_state",
            "deactivate",
            "load",
            "activate",
            "restore_state"
        ]
    );
}

#[test]
fn stateless_hot_reload_policy_skips_state_transfer() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = PolicyRecordingBackend {
        events: Arc::clone(&events),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();

    let slot = coordinator
        .load_package(
            "policy-recording",
            &backend,
            test_package("0.1.0").with_hot_reload_policy(VmPluginHotReloadPolicy::Stateless),
            &host,
        )
        .unwrap();

    coordinator
        .hot_reload(
            slot,
            "policy-recording",
            &backend,
            test_package("0.2.0"),
            &host,
        )
        .unwrap();

    assert_eq!(coordinator.slot(slot).unwrap().generation, 2);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["load", "activate", "deactivate", "load", "activate"]
    );
}

#[test]
fn disabled_hot_reload_policy_rejects_reload_without_deactivating_slot() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = PolicyRecordingBackend {
        events: Arc::clone(&events),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();

    let slot = coordinator
        .load_package(
            "policy-recording",
            &backend,
            test_package("0.1.0").with_hot_reload_policy(VmPluginHotReloadPolicy::Disabled),
            &host,
        )
        .unwrap();

    let error = coordinator
        .hot_reload(
            slot,
            "policy-recording",
            &backend,
            test_package("0.2.0"),
            &host,
        )
        .unwrap_err();

    assert!(error.to_string().contains("does not allow hot reload"));
    let record = coordinator.slot(slot).unwrap();
    assert_eq!(record.generation, 1);
    assert_eq!(
        record.management.hot_reload,
        VmPluginHotReloadPolicy::Disabled
    );
    assert_eq!(events.lock().unwrap().as_slice(), &["load", "activate"]);
}

#[test]
fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = Arc::new(LifecycleQueryBackend {
        events: Arc::clone(&events),
    });
    let coordinator = Arc::new(HotReloadCoordinator::new());
    let host = test_host_context_with_lifecycle(Arc::new(CoordinatorSlotLifecycle {
        coordinator: Arc::clone(&coordinator),
    }));

    let slot = coordinator
        .load_package(
            "lifecycle-query",
            backend.as_ref(),
            test_package("0.1.0"),
            &host,
        )
        .unwrap();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = {
        let coordinator = Arc::clone(&coordinator);
        let backend = Arc::clone(&backend);
        let host = host.clone();
        std::thread::spawn(move || {
            let result = coordinator.hot_reload(
                slot,
                "lifecycle-query",
                backend.as_ref(),
                test_package("0.2.0"),
                &host,
            );
            let _ = done_tx.send(result);
        })
    };

    let result = done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("hot reload hooks should not block on slot lifecycle queries");
    worker.join().unwrap();
    result.unwrap();

    assert_eq!(coordinator.slot(slot).unwrap().generation, 2);
    assert_eq!(
        events.lock().unwrap().clone(),
        vec![
            "load".to_string(),
            "activate_query_empty".to_string(),
            "deactivate".to_string(),
            "load".to_string(),
            "activate_query_reloading".to_string(),
            "restore_state_query".to_string()
        ]
    );
}

#[test]
fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock() {
    let coordinator = HotReloadCoordinator::new();

    let poison_result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = coordinator.slots.lock().unwrap();
        panic!("poison hot reload slot table");
    }));
    assert!(poison_result.is_err());
    assert!(coordinator.list_slots().is_empty());

    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = PolicyRecordingBackend {
        events: Arc::clone(&events),
    };
    let host = test_host_context();
    let slot = coordinator
        .load_package("policy-recording", &backend, test_package("0.1.0"), &host)
        .unwrap();

    assert_eq!(
        coordinator.slot(slot).unwrap().state,
        VmPluginSlotState::Active
    );
    assert_eq!(coordinator.slot_for_package_name("policy").unwrap(), slot);
    assert_eq!(coordinator.unload_slot(slot).unwrap().name, "policy");
    assert!(coordinator.list_slots().is_empty());
}

fn test_package(version: &str) -> VmPluginPackage {
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: "policy".to_string(),
            version: version.to_string(),
            entry: "main".to_string(),
            capabilities: CapabilitySet::default(),
            management: crate::script::VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    }
}

trait TestPackagePolicyExt {
    fn with_hot_reload_policy(self, policy: VmPluginHotReloadPolicy) -> Self;
}

impl TestPackagePolicyExt for VmPluginPackage {
    fn with_hot_reload_policy(mut self, policy: VmPluginHotReloadPolicy) -> Self {
        self.manifest.management.hot_reload = policy;
        self
    }
}

fn test_host_context() -> VmPluginHostContext {
    test_host_context_with_lifecycle(Arc::new(NoopSlotLifecycle))
}

fn test_host_context_with_lifecycle(
    slot_lifecycle: Arc<dyn VmPluginSlotLifecycle>,
) -> VmPluginHostContext {
    let runtime = CoreRuntime::new();
    let package_root = PathBuf::from("policy-package");
    VmPluginHostContext {
        plugin: PluginContext {
            plugin_name: "VmPluginRuntime".to_string(),
            core: runtime.handle().downgrade(),
            package_root: Some(package_root.clone()),
            source_root: Some(package_root.clone()),
            data_root: Some(package_root.join("data")),
        },
        capabilities: CapabilitySet::default(),
        backend_selector: "policy-recording".to_string(),
        package_source: VmPluginPackageSource {
            package_root: Some(package_root),
            manifest_path: None,
            bytecode_path: None,
            zr_vm_project_path: None,
        },
        host_registry: HostRegistry::default(),
        host_exports: HostExportRegistry::default(),
        slot_lifecycle,
    }
}
