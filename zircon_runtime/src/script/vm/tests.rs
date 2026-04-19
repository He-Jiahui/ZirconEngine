#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_core::CoreRuntime;

    use super::super::{
        backend::MockVmBackend, module_descriptor, CapabilitySet, HostRegistry,
        HotReloadCoordinator, PluginHostDriver, UnavailableVmBackend, VmBackend, VmError,
        VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
        PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
        VM_PLUGIN_RUNTIME_NAME,
    };

    #[test]
    fn host_handles_are_stable_and_valid() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("RenderingManager");
        assert!(registry.is_valid(handle));
    }

    #[test]
    fn hot_reload_coordinator_tracks_slot_lifecycle_records() {
        let coordinator = HotReloadCoordinator::new(HostRegistry::default());
        let package_root = std::env::temp_dir().join("zircon-script-slot-lifecycle");
        let source = VmPluginPackageSource {
            package_root: Some(package_root.clone()),
            manifest_path: Some(package_root.join("plugin.toml")),
            bytecode_path: Some(package_root.join("plugin.bin")),
        };

        let slot = coordinator
            .load_package_with_source(
                "mock",
                &MockVmBackend,
                test_package("sample", "0.1.0"),
                source.clone(),
            )
            .unwrap();
        let initial = coordinator.slot(slot).unwrap();
        assert_eq!(initial.backend_name, "mock");
        assert_eq!(initial.source, source);
        assert_eq!(initial.manifest.version, "0.1.0");

        coordinator
            .hot_reload_with_source(
                slot,
                "mock",
                &MockVmBackend,
                test_package("sample", "0.2.0"),
                source.clone(),
            )
            .unwrap();

        let reloaded = coordinator.slot(slot).unwrap();
        assert_eq!(reloaded.manifest.version, "0.2.0");
        assert_eq!(coordinator.list_slots(), vec![reloaded.clone()]);

        let unloaded = coordinator.unload_slot(slot).unwrap();
        assert_eq!(unloaded.version, "0.2.0");
        assert!(matches!(
            coordinator.slot(slot),
            Err(VmError::MissingSlot(missing)) if missing == slot.get()
        ));
    }

    #[test]
    fn vm_plugin_manager_discovers_packages_selects_backends_and_loads_slots() {
        let fixture = PluginFixture::new("sample", "0.1.0", "mock", &[1, 2, 3]);
        let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
        let packages = manager.discover_packages(&fixture.root).unwrap();

        assert_eq!(packages.len(), 1);
        assert!(manager.backend_names().contains(&"mock".to_string()));
        assert!(manager.backend_names().contains(&"unavailable".to_string()));

        let discovered = &packages[0];
        assert_eq!(discovered.backend_name, "mock");
        assert_eq!(discovered.package.manifest.name, "sample");
        assert_eq!(
            discovered.source.manifest_path.as_deref(),
            Some(fixture.manifest_path.as_path())
        );
        assert_eq!(
            discovered.source.bytecode_path.as_deref(),
            Some(fixture.bytecode_path.as_path())
        );

        let slot = manager.load_discovered_package(discovered).unwrap();
        let loaded = manager.slot(slot).unwrap();
        assert_eq!(loaded.backend_name, "mock");
        assert_eq!(loaded.manifest.version, "0.1.0");
        assert_eq!(loaded.source, discovered.source);

        manager.select_default_backend("unavailable").unwrap();
        manager
            .hot_reload_slot(slot, test_package("sample", "0.2.0"))
            .unwrap();
        let reloaded = manager.slot(slot).unwrap();
        assert_eq!(reloaded.backend_name, "mock");
        assert_eq!(reloaded.manifest.version, "0.2.0");

        manager.unload_slot(slot).unwrap();
        assert!(manager.list_slots().is_empty());
    }

    #[test]
    fn unavailable_backend_reports_error() {
        let backend = UnavailableVmBackend;
        let error =
            match backend.load_package(&test_package("sample", "0.1.0"), HostRegistry::default()) {
                Ok(_) => panic!("expected unavailable backend to reject package"),
                Err(error) => error,
            };
        assert!(matches!(error, VmError::BackendUnavailable(_)));
    }

    #[test]
    fn script_module_descriptor_registers_vm_plugin_runtime_before_manager_facade() {
        let descriptor = module_descriptor();

        let plugin = descriptor
            .plugins
            .iter()
            .find(|plugin| plugin.name.as_str() == VM_PLUGIN_RUNTIME_NAME)
            .expect("vm plugin runtime descriptor");
        assert_eq!(plugin.startup_mode, zircon_core::StartupMode::Immediate);
        assert!(plugin
            .dependencies
            .iter()
            .any(|dependency| dependency.name.as_str() == PLUGIN_HOST_DRIVER_NAME));

        let manager = descriptor
            .managers
            .iter()
            .find(|manager| manager.name.as_str() == VM_PLUGIN_MANAGER_NAME)
            .expect("vm plugin manager descriptor");
        assert!(manager
            .dependencies
            .iter()
            .any(|dependency| dependency.name.as_str() == VM_PLUGIN_RUNTIME_NAME));
    }

    #[test]
    fn core_resolve_plugin_exposes_vm_plugin_runtime_and_manager_facade_shares_it() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        core.register_module(module_descriptor())
            .expect("register script module");
        core.activate_module(SCRIPT_MODULE_NAME)
            .expect("activate script module");

        let plugin = core
            .resolve_plugin::<VmPluginManager>(VM_PLUGIN_RUNTIME_NAME)
            .expect("resolve vm plugin runtime");
        let manager = core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .expect("resolve vm plugin manager facade");
        let driver = core
            .resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)
            .expect("resolve plugin host driver");

        assert!(Arc::ptr_eq(&plugin, &manager));

        let capability = driver.registry().register_capability("RenderingManager");
        assert!(plugin.host_registry().is_valid(capability));

        plugin.register_backend(Arc::new(MockVmBackend));
        plugin.select_default_backend("mock").unwrap();
        let slot = plugin.load_package(test_package("core", "0.1.0")).unwrap();
        assert_eq!(plugin.slot(slot).unwrap().backend_name, "mock");
    }

    #[test]
    fn vm_plugin_protocol_types_live_in_script_subsystem() {
        let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let manager_root = runtime_root.join("../zircon_manager");
        let script_mod_source = include_str!("../mod.rs");
        let vm_mod_source = include_str!("mod.rs");
        let manifest_source = include_str!("plugin/vm_plugin_manifest.rs");
        let host_registry_source = include_str!("host/host_registry.rs");
        let package_discovery_source = include_str!("plugin/vm_plugin_package_discovery.rs");
        let hot_reload_source = include_str!("runtime/hot_reload_coordinator.rs");
        let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
        let manager_handles_source =
            std::fs::read_to_string(manager_root.join("src/handles.rs")).unwrap_or_default();
        let manager_records_root = manager_root.join("src/records");

        for required in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
            assert!(
                script_mod_source.contains(required) || vm_mod_source.contains(required),
                "zircon_runtime::script should publicly export {required}"
            );
        }

        for source in [
            manifest_source,
            host_registry_source,
            package_discovery_source,
            hot_reload_source,
        ] {
            assert!(
                !source.contains("use zircon_manager::"),
                "vm runtime files should not source script protocol types from zircon_manager"
            );
        }

        for forbidden in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
            assert!(
                !manager_lib_source.contains(forbidden),
                "zircon_manager lib.rs should not re-export {forbidden} after vm plugin boundary cleanup"
            );
        }
        assert!(
            !manager_handles_source.contains("define_handle!(PluginSlotId);"),
            "zircon_manager handles should not define PluginSlotId after vm plugin boundary cleanup"
        );
        assert!(
            !manager_handles_source.contains("define_handle!(HostHandle);"),
            "zircon_manager handles should not define HostHandle after vm plugin boundary cleanup"
        );
        assert!(
            !manager_records_root.exists(),
            "zircon_manager should delete src/records after vm plugin boundary cleanup"
        );
    }

    #[test]
    fn vm_subsystem_is_grouped_by_module_backend_host_plugin_and_runtime() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("script")
            .join("vm");

        for relative in [
            "module/mod.rs",
            "module/script_module.rs",
            "module/module_descriptor.rs",
            "backend/mod.rs",
            "backend/backend_registry.rs",
            "backend/vm_backend.rs",
            "backend/unavailable_vm_backend.rs",
            "backend/mock_vm_backend.rs",
            "backend/vm_error.rs",
            "host/mod.rs",
            "host/host_registry.rs",
            "host/plugin_host_driver.rs",
            "host/constants.rs",
            "plugin/mod.rs",
            "plugin/vm_plugin_manifest.rs",
            "plugin/vm_plugin_package.rs",
            "plugin/vm_plugin_package_source.rs",
            "plugin/vm_plugin_package_discovery.rs",
            "plugin/vm_plugin_instance.rs",
            "plugin/vm_state_blob.rs",
            "runtime/mod.rs",
            "runtime/hot_reload_coordinator.rs",
            "runtime/vm_plugin_slot_record.rs",
            "runtime/vm_plugin_manager.rs",
        ] {
            assert!(
                root.join(relative).exists(),
                "expected vm module {relative} under {:?}",
                root
            );
        }
    }

    fn test_package(name: &str, version: &str) -> VmPluginPackage {
        VmPluginPackage {
            manifest: VmPluginManifest {
                name: name.to_string(),
                version: version.to_string(),
                entry: "main".to_string(),
                capabilities: CapabilitySet::default().with("render"),
            },
            bytecode: vec![1, 2, 3],
        }
    }

    struct PluginFixture {
        root: PathBuf,
        manifest_path: PathBuf,
        bytecode_path: PathBuf,
    }

    impl PluginFixture {
        fn new(name: &str, version: &str, backend: &str, bytecode: &[u8]) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("zircon-script-fixture-{nonce}"));
            let package_root = root.join(name);
            fs::create_dir_all(&package_root).unwrap();

            let manifest_path = package_root.join("plugin.toml");
            let bytecode_path = package_root.join("plugin.bin");
            fs::write(
                &manifest_path,
                format!(
                    "name = \"{name}\"\nversion = \"{version}\"\nentry = \"main\"\nbackend = \"{backend}\"\nbytecode = \"plugin.bin\"\n\n[capabilities]\ncapabilities = [\"render\"]\n"
                ),
            )
            .unwrap();
            fs::write(&bytecode_path, bytecode).unwrap();

            Self {
                root,
                manifest_path,
                bytecode_path,
            }
        }
    }

    impl Drop for PluginFixture {
        fn drop(&mut self) {
            let _ = remove_dir_all_if_exists(&self.root);
        }
    }

    fn remove_dir_all_if_exists(path: &Path) -> Result<(), std::io::Error> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}
