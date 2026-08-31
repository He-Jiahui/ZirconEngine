use super::*;

#[derive(Debug)]
struct PanicOnceExportBackend;

impl VmBackend for PanicOnceExportBackend {
    fn backend_name(&self) -> &str {
        "panic-once-export"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(PanicOnceExportInstance {
            manifest: package.manifest.clone(),
            panic_once: true,
        }))
    }
}

#[derive(Debug)]
struct PanicOnceExportInstance {
    manifest: VmPluginManifest,
    panic_once: bool,
}

impl VmPluginInstance for PanicOnceExportInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn call_export(
        &mut self,
        _module_name: &str,
        _export_name: &str,
        _arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        if self.panic_once {
            self.panic_once = false;
            panic!("intentional export panic");
        }
        Ok(None)
    }
}

#[test]
fn panicked_export_restores_the_active_instance() {
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    let slot = coordinator
        .load_package(
            "panic-once-export",
            &PanicOnceExportBackend,
            test_package("0.1.0"),
            &host,
        )
        .unwrap();

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = coordinator.call_slot_export(slot, "main", "tick", &[]);
    }));
    assert!(panic.is_err());

    let record = coordinator.slot(slot).unwrap();
    assert_eq!(record.state, VmPluginSlotState::Active);
    assert_eq!(record.generation, 1);
    assert!(coordinator
        .call_slot_export(slot, "main", "tick", &[])
        .unwrap()
        .is_none());
}
