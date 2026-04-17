#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::{
        backend::MockVmBackend, CapabilitySet, HostRegistry, HotReloadCoordinator,
        UnavailableVmBackend, VmBackend, VmError, VmPluginManifest, VmPluginPackage,
    };

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

    #[test]
    fn vm_plugin_protocol_types_live_in_script_subsystem() {
        let script_lib_source = include_str!("../lib.rs");
        let vm_mod_source = include_str!("mod.rs");
        let manifest_source = include_str!("plugin/vm_plugin_manifest.rs");
        let host_registry_source = include_str!("host/host_registry.rs");
        let hot_reload_source = include_str!("runtime/hot_reload_coordinator.rs");
        let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
        let manager_records_mod_source =
            include_str!("../../../zircon_manager/src/records/mod.rs");
        let manager_handles_source = include_str!("../../../zircon_manager/src/handles.rs");
        let manager_capability_set_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../zircon_manager/src/records/capability_set.rs");

        for required in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
            assert!(
                script_lib_source.contains(required) || vm_mod_source.contains(required),
                "zircon_script should publicly export {required}"
            );
        }

        for source in [manifest_source, host_registry_source, hot_reload_source] {
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
            !manager_records_mod_source.contains("CapabilitySet"),
            "zircon_manager records mod should not re-export CapabilitySet after vm plugin boundary cleanup"
        );
        assert!(
            !manager_handles_source.contains("define_handle!(PluginSlotId);"),
            "zircon_manager handles should not define PluginSlotId after vm plugin boundary cleanup"
        );
        assert!(
            !manager_handles_source.contains("define_handle!(HostHandle);"),
            "zircon_manager handles should not define HostHandle after vm plugin boundary cleanup"
        );
        assert!(
            !manager_capability_set_path.exists(),
            "zircon_manager should delete src/records/capability_set.rs after vm plugin boundary cleanup"
        );
    }

    #[test]
    fn vm_subsystem_is_grouped_by_module_backend_host_plugin_and_runtime() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("vm");

        for relative in [
            "module/mod.rs",
            "module/script_module.rs",
            "module/module_descriptor.rs",
            "backend/mod.rs",
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
            "plugin/vm_plugin_instance.rs",
            "plugin/vm_state_blob.rs",
            "runtime/mod.rs",
            "runtime/vm_plugin_manager.rs",
            "runtime/hot_reload_coordinator.rs",
        ] {
            assert!(
                root.join(relative).exists(),
                "expected vm module {relative} under {:?}",
                root
            );
        }
    }
}
