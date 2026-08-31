use std::sync::{Arc, Mutex};

use crate::core::framework::script::ScriptHostValue;
use crate::script::{
    CapabilitySet, HostRegistry, VmBackend, VmBackendFamily, VmError, VmPluginHostContext,
    VmPluginInstance, VmPluginManager, VmPluginManifest, VmPluginPackage, VmStateBlob,
};

#[derive(Debug)]
struct FallbackLifecycleBackend {
    events: Arc<Mutex<Vec<String>>>,
}

impl VmBackend for FallbackLifecycleBackend {
    fn backend_name(&self) -> &str {
        "lifecycle:fallback"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("load:{}", package.manifest.entry));
        Ok(Box::new(FallbackLifecycleInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
            state: VmStateBlob::default(),
        }))
    }
}

#[derive(Debug)]
struct FallbackLifecycleFamily {
    backend: Arc<FallbackLifecycleBackend>,
}

impl VmBackendFamily for FallbackLifecycleFamily {
    fn family_name(&self) -> &str {
        "lifecycle"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "lifecycle:fallback" | "fallback" => {
                let backend: Arc<dyn VmBackend> = self.backend.clone();
                Ok(backend)
            }
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor("lifecycle:fallback");
        visitor("fallback");
    }
}

#[derive(Debug)]
struct FallbackLifecycleInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<String>>>,
    state: VmStateBlob,
}

impl VmPluginInstance for FallbackLifecycleInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("activate:{}", self.manifest.entry));
        if self.manifest.entry == "bad_entry" {
            return Err(VmError::Operation(
                "fallback vm rejected bad entry module bad_entry".to_string(),
            ));
        }
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("deactivate:{}", self.manifest.name));
        Ok(())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        Ok(self.state.clone())
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        self.state = state.clone();
        Ok(())
    }

    fn call_export(
        &mut self,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        self.events.lock().unwrap().push(format!(
            "call:{module_name}.{export_name}:args={}",
            arguments.len()
        ));
        if export_name == "missingOptional" {
            return Ok(None);
        }

        let call = serde_json::json!({
            "module": module_name,
            "export": export_name,
            "arguments": arguments,
        });
        self.state.payload = serde_json::to_vec(&call)
            .map_err(|error| VmError::Operation(format!("fallback call encode failed: {error}")))?;
        Ok(Some(ScriptHostValue::Null))
    }
}

#[test]
fn vm_lifecycle_fallback_activate_bad_entry_module_surfaces_vm_error() {
    let (manager, events) = fallback_lifecycle_manager();

    let error = manager
        .load_package(test_package("sample", "0.1.0", "bad_entry"))
        .expect_err("bad entry package should fail during fallback activation");

    assert!(
        error
            .to_string()
            .contains("fallback vm rejected bad entry module bad_entry"),
        "bad entry failure should surface as an explicit VM error: {error}"
    );
    assert!(
        manager.list_slots().is_empty(),
        "failed fallback activation must not leave a live plugin slot"
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["load:bad_entry", "activate:bad_entry"]
    );
}

#[test]
fn vm_lifecycle_fallback_missing_optional_export_returns_none_not_error() {
    let (manager, events) = fallback_lifecycle_manager();
    let slot = manager
        .load_package(test_package("sample", "0.1.0", "main"))
        .unwrap();

    let result = manager
        .call_slot_export(slot, "main", "missingOptional", &[])
        .unwrap();

    assert_eq!(result, None);
    assert_eq!(
        manager.slot(slot).unwrap().manifest.name,
        "sample",
        "missing optional export should leave the fallback slot active"
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &[
            "load:main",
            "activate:main",
            "call:main.missingOptional:args=0"
        ]
    );
}

#[test]
fn vm_lifecycle_fallback_deactivate_is_idempotent_after_unload() {
    let (manager, events) = fallback_lifecycle_manager();
    let slot = manager
        .load_package(test_package("sample", "0.1.0", "main"))
        .unwrap();

    manager.unload_slot(slot).unwrap();
    let second_unload = manager
        .unload_slot(slot)
        .expect_err("second unload should be a stable missing-slot result");

    assert!(matches!(
        second_unload,
        VmError::MissingSlot(missing) if missing == slot.get()
    ));
    assert!(manager.list_slots().is_empty());
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["load:main", "activate:main", "deactivate:sample"]
    );
}

#[test]
fn vm_lifecycle_fallback_empty_arguments_do_not_require_real_backend() {
    let (manager, events) = fallback_lifecycle_manager();
    manager
        .load_package(test_package("sample", "0.1.0", "main"))
        .unwrap();

    let result = manager
        .call_package_export("sample", "main", "onUpdate", &[])
        .unwrap();

    assert_eq!(result, Some(ScriptHostValue::Null));
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["load:main", "activate:main", "call:main.onUpdate:args=0"]
    );
}

fn fallback_lifecycle_manager() -> (Arc<VmPluginManager>, Arc<Mutex<Vec<String>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
    manager.register_family(Arc::new(FallbackLifecycleFamily {
        backend: Arc::new(FallbackLifecycleBackend {
            events: Arc::clone(&events),
        }),
    }));
    manager
        .select_default_backend("lifecycle:fallback")
        .unwrap();
    (manager, events)
}

fn test_package(name: &str, version: &str, entry: &str) -> VmPluginPackage {
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: name.to_string(),
            version: version.to_string(),
            entry: entry.to_string(),
            capabilities: CapabilitySet::default(),
            management: crate::script::VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    }
}
