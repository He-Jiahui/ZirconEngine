use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::core::framework::script::ScriptHostValue;

use super::super::backend::{VmBackend, VmError};
use super::super::handles::PluginSlotId;
use super::super::host::VmPluginHostContext;
use super::super::plugin::{
    VmPluginHotReloadPolicy, VmPluginInstance, VmPluginManifest, VmPluginPackage,
    VmPluginPackageSource,
};
use super::vm_plugin_slot_record::VmPluginSlotRecord;
use super::vm_plugin_slot_state::VmPluginSlotState;

pub struct HotReloadCoordinator {
    next_slot: AtomicU64,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
}

struct PluginSlot {
    backend_name: String,
    state: VmPluginSlotState,
    generation: u64,
    source: VmPluginPackageSource,
    package: VmPluginPackage,
    instance: Option<Box<dyn VmPluginInstance>>,
}

impl PluginSlot {
    fn active(
        backend_name: String,
        generation: u64,
        source: VmPluginPackageSource,
        package: VmPluginPackage,
        instance: Box<dyn VmPluginInstance>,
    ) -> Self {
        Self {
            backend_name,
            state: VmPluginSlotState::Active,
            generation,
            source,
            package,
            instance: Some(instance),
        }
    }

    fn record(&self, slot: PluginSlotId) -> VmPluginSlotRecord {
        VmPluginSlotRecord {
            slot,
            backend_name: self.backend_name.clone(),
            state: self.state,
            generation: self.generation,
            source: self.source.clone(),
            manifest: self.package.manifest.clone(),
            management: self.package.manifest.management.clone(),
        }
    }
}

impl fmt::Debug for HotReloadCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadCoordinator")
            .field("slot_count", &self.slots.lock().unwrap().len())
            .finish()
    }
}

impl HotReloadCoordinator {
    pub fn new() -> Self {
        Self {
            next_slot: AtomicU64::new(1),
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_package(
        &self,
        backend_name: impl Into<String>,
        backend: &dyn VmBackend,
        package: VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<PluginSlotId, VmError> {
        let mut instance = backend.load_package(&package, host)?;
        instance.activate(host)?;
        let slot = PluginSlotId::new(self.next_slot.fetch_add(1, Ordering::SeqCst));
        self.slots.lock().unwrap().insert(
            slot,
            PluginSlot::active(
                backend_name.into(),
                1,
                host.package_source.clone(),
                package,
                instance,
            ),
        );
        Ok(slot)
    }

    pub fn hot_reload(
        &self,
        slot: PluginSlotId,
        backend_name: impl Into<String>,
        backend: &dyn VmBackend,
        package: VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<(), VmError> {
        let backend_name = backend_name.into();
        let (policy, mut current_instance, next_generation) = {
            let mut slots = self.slots.lock().unwrap();
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let policy = slot_entry.package.manifest.management.hot_reload;
            if matches!(policy, VmPluginHotReloadPolicy::Disabled) {
                return Err(VmError::Operation(format!(
                    "vm plugin slot {} does not allow hot reload",
                    slot.get()
                )));
            }
            let current_instance = slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} is already {}",
                    slot.get(),
                    slot_entry.state.label()
                ))
            })?;
            slot_entry.state = VmPluginSlotState::Reloading;
            (policy, current_instance, slot_entry.generation + 1)
        };

        let state = match policy {
            VmPluginHotReloadPolicy::Disabled => unreachable!("disabled policy returned above"),
            VmPluginHotReloadPolicy::Stateless => None,
            VmPluginHotReloadPolicy::PreserveState => match current_instance.save_state() {
                Ok(state) => Some(state),
                Err(error) => {
                    self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Active);
                    return Err(error);
                }
            },
        };
        if let Err(error) = current_instance.deactivate() {
            self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
            return Err(error);
        }

        let mut next_instance = match backend.load_package(&package, host) {
            Ok(instance) => instance,
            Err(error) => {
                self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
                return Err(error);
            }
        };
        if let Err(error) = next_instance.activate(host) {
            self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
            return Err(error);
        }
        if let Some(state) = &state {
            if let Err(error) = next_instance.restore_state(state) {
                self.replace_slot(
                    slot,
                    PluginSlot {
                        backend_name,
                        state: VmPluginSlotState::Failed,
                        generation: next_generation,
                        source: host.package_source.clone(),
                        package,
                        instance: Some(next_instance),
                    },
                );
                return Err(error);
            }
        }

        self.replace_slot(
            slot,
            PluginSlot::active(
                backend_name,
                next_generation,
                host.package_source.clone(),
                package,
                next_instance,
            ),
        );
        Ok(())
    }

    fn restore_slot_instance(
        &self,
        slot: PluginSlotId,
        instance: Box<dyn VmPluginInstance>,
        state: VmPluginSlotState,
    ) {
        let mut slots = self.slots.lock().unwrap();
        if let Some(slot_entry) = slots.get_mut(&slot) {
            slot_entry.instance = Some(instance);
            slot_entry.state = state;
        }
    }

    fn replace_slot(&self, slot: PluginSlotId, slot_entry: PluginSlot) {
        self.slots.lock().unwrap().insert(slot, slot_entry);
    }

    pub fn unload_slot(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        let mut slot_entry = {
            let mut slots = self.slots.lock().unwrap();
            if let Some(slot_entry) = slots.get(&slot) {
                if slot_entry.instance.is_none() {
                    return Err(VmError::Operation(format!(
                        "vm plugin slot {} cannot unload while {}",
                        slot.get(),
                        slot_entry.state.label()
                    )));
                }
            }
            slots
                .remove(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?
        };
        let manifest = slot_entry.package.manifest.clone();
        if let Some(mut instance) = slot_entry.instance.take() {
            instance.deactivate()?;
        }
        Ok(manifest)
    }

    pub fn manifest(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        Ok(self.slot(slot)?.manifest)
    }

    pub fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        let slots = self.slots.lock().unwrap();
        let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
        Ok(slot_entry.record(slot))
    }

    pub fn slot_for_package_name(&self, package_name: &str) -> Result<PluginSlotId, VmError> {
        let slots = self.slots.lock().unwrap();
        slots
            .iter()
            .filter(|(_, entry)| {
                entry.state == VmPluginSlotState::Active
                    && entry.package.manifest.name == package_name
            })
            .map(|(slot, _)| *slot)
            .min_by_key(|slot| slot.get())
            .ok_or_else(|| {
                VmError::Operation(format!("vm plugin package {package_name} is not loaded"))
            })
    }

    pub fn call_slot_export(
        &self,
        slot: PluginSlotId,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let mut instance = {
            let mut slots = self.slots.lock().unwrap();
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            if slot_entry.state != VmPluginSlotState::Active {
                return Err(VmError::Operation(format!(
                    "vm plugin slot {} cannot call export while {}",
                    slot.get(),
                    slot_entry.state.label()
                )));
            }
            slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} cannot call export while active instance is unavailable",
                    slot.get()
                ))
            })?
        };

        let result = instance.call_export(module_name, export_name, arguments);
        self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
        result
    }

    pub fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        let mut records = self
            .slots
            .lock()
            .unwrap()
            .iter()
            .map(|(slot, entry)| entry.record(*slot))
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.slot.get());
        records
    }
}

#[cfg(test)]
mod tests {
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
}
