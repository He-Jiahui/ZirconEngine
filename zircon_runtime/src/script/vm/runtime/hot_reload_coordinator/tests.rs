use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{CoreRuntime, PluginContext};
use crate::script::{
    CapabilitySet, HostExportRegistry, HostRegistry, VmBackend, VmGcBudget, VmGcStepOutcome,
    VmHostInterfaceRegistry, VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy,
    VmPluginHostContext, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle,
};

use super::*;

#[derive(Debug)]
struct PolicyRecordingBackend {
    events: Arc<Mutex<Vec<&'static str>>>,
}

#[derive(Debug)]
struct GcRecordingBackend {
    calls: Arc<Mutex<Vec<(PluginSlotId, u64)>>>,
}

impl VmBackend for GcRecordingBackend {
    fn backend_name(&self) -> &str {
        "gc-recording"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        let (slot, _) = host
            .vm_owner()
            .ok_or_else(|| VmError::Operation("GC test instance has no slot owner".to_string()))?;
        let pause_micros = package
            .manifest
            .version
            .parse::<u64>()
            .map_err(|error| VmError::Operation(error.to_string()))?;
        Ok(Box::new(GcRecordingInstance {
            manifest: package.manifest.clone(),
            slot,
            pause_micros,
            calls: Arc::clone(&self.calls),
            panic_once: package.manifest.name == "panic-once",
        }))
    }
}

#[derive(Debug)]
struct GcRecordingInstance {
    manifest: VmPluginManifest,
    slot: PluginSlotId,
    pause_micros: u64,
    calls: Arc<Mutex<Vec<(PluginSlotId, u64)>>>,
    panic_once: bool,
}

#[derive(Debug)]
struct BlockingGcBackend {
    entered: mpsc::SyncSender<()>,
    release: Arc<Mutex<mpsc::Receiver<()>>>,
    calls: Arc<AtomicUsize>,
}

impl VmBackend for BlockingGcBackend {
    fn backend_name(&self) -> &str {
        "blocking-gc"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(BlockingGcInstance {
            manifest: package.manifest.clone(),
            entered: self.entered.clone(),
            release: Arc::clone(&self.release),
            calls: Arc::clone(&self.calls),
        }))
    }
}

#[derive(Debug)]
struct BlockingGcInstance {
    manifest: VmPluginManifest,
    entered: mpsc::SyncSender<()>,
    release: Arc<Mutex<mpsc::Receiver<()>>>,
    calls: Arc<AtomicUsize>,
}

impl VmPluginInstance for BlockingGcInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn gc_step(&mut self, _budget: VmGcBudget) -> Result<VmGcStepOutcome, VmError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.entered
            .send(())
            .map_err(|error| VmError::Operation(error.to_string()))?;
        self.release
            .lock()
            .unwrap()
            .recv()
            .map_err(|error| VmError::Operation(error.to_string()))?;
        Ok(VmGcStepOutcome {
            pause_micros: 1,
            root_count: 1,
            cross_boundary_reference_count: 0,
        })
    }
}

impl VmPluginInstance for GcRecordingInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn gc_step(&mut self, budget: VmGcBudget) -> Result<VmGcStepOutcome, VmError> {
        self.calls
            .lock()
            .unwrap()
            .push((self.slot, budget.max_micros_per_frame));
        if self.panic_once {
            self.panic_once = false;
            panic!("intentional GC backend panic");
        }
        Ok(VmGcStepOutcome {
            pause_micros: self.pause_micros,
            root_count: self.slot.get(),
            cross_boundary_reference_count: self.slot.get() + 10,
        })
    }
}

#[derive(Debug)]
struct RegistrationRetryBackend {
    load_count: AtomicUsize,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmBackend for RegistrationRetryBackend {
    fn backend_name(&self) -> &str {
        "registration-retry"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        let load_count = self
            .load_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        let caller = host
            .interface_caller()
            .map_err(|error| VmError::Operation(error.to_string()))?;
        host.host_interfaces
            .register_behavior_node(&caller, "script.retry", "Script Retry", "main", "tick")
            .map_err(|error| VmError::Operation(error.to_string()))?;
        if load_count == 2 {
            return Err(VmError::Operation(
                "intentional hot reload load failure".to_string(),
            ));
        }
        Ok(Box::new(PolicyRecordingInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
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
        Ok(crate::script::VmStateBlob::from_payload(b"saved".to_vec()))
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
fn failed_hot_reload_load_discards_generation_registrations_before_retry() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = RegistrationRetryBackend {
        load_count: AtomicUsize::new(0),
        events,
    };
    let coordinator = HotReloadCoordinator::new();
    let mut host = test_host_context();
    host.capabilities = CapabilitySet::default().with(crate::script::VM_BT_NODE_CAPABILITY);

    let slot = coordinator
        .load_package("registration-retry", &backend, test_package("0.1.0"), &host)
        .unwrap();
    assert!(coordinator
        .hot_reload(
            slot,
            "registration-retry",
            &backend,
            test_package("0.2.0"),
            &host,
        )
        .is_err());
    let rolled_back = coordinator.slot(slot).unwrap();
    assert_eq!(rolled_back.state, VmPluginSlotState::Active);
    assert_eq!(rolled_back.generation, 1);
    let registrations = host
        .host_interfaces
        .behavior_nodes(&coordinator.list_slots());
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].callback.generation, 1);

    coordinator
        .hot_reload(
            slot,
            "registration-retry",
            &backend,
            test_package("0.2.0"),
            &host,
        )
        .expect("retry should re-register the discarded generation");

    let registrations = host
        .host_interfaces
        .behavior_nodes(&coordinator.list_slots());
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].callback.generation, 2);
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

mod gc;
mod state_migration;

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

fn gc_test_package(
    name: &str,
    pause_micros: u64,
    mode: VmPluginGarbageCollectionMode,
    interval_frames: Option<u64>,
) -> VmPluginPackage {
    let mut package = test_package(&pause_micros.to_string());
    package.manifest.name = name.to_string();
    package.manifest.management.garbage_collection = VmPluginGarbageCollectionPolicy {
        mode,
        interval_frames,
    };
    package
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
        host_interfaces: VmHostInterfaceRegistry::default(),
        slot_lifecycle,
        vm_owner: None,
    }
}
