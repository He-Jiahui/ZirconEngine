use super::*;

#[derive(Debug)]
struct DeactivateFailureBackend {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmBackend for DeactivateFailureBackend {
    fn backend_name(&self) -> &str {
        "deactivate-failure"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.events.lock().unwrap().push("load");
        Ok(Box::new(DeactivateFailureInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
}

#[derive(Debug)]
struct DeactivateFailureInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmPluginInstance for DeactivateFailureInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        self.events.lock().unwrap().push("activate");
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        self.events.lock().unwrap().push("deactivate");
        Err(VmError::Operation(
            "intentional unload deactivate failure".to_string(),
        ))
    }
}

#[test]
fn deactivate_failure_retains_slot_and_catalog_revision() {
    let coordinator = HotReloadCoordinator::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = DeactivateFailureBackend {
        events: Arc::clone(&events),
    };
    let host = test_host_context();
    let slot = coordinator
        .load_package(
            backend.backend_name(),
            &backend,
            test_package("0.1.0"),
            &host,
        )
        .expect("test package should load");
    let revision = host.reflection_catalog.revision();

    let error = coordinator
        .unload_slot(slot)
        .expect_err("deactivate failure must abort the unload transaction");

    assert!(error
        .to_string()
        .contains("intentional unload deactivate failure"));
    assert_eq!(
        coordinator
            .slot(slot)
            .expect("failed unload must retain the slot")
            .state,
        VmPluginSlotState::Failed
    );
    assert_eq!(host.reflection_catalog.revision(), revision);
    assert_eq!(
        &*events.lock().unwrap(),
        &["load", "activate", "deactivate"]
    );
}
