#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_manager::CapabilitySet;

    use super::super::{
        host_registry::HostRegistry, hot_reload_coordinator::HotReloadCoordinator,
        mock_vm_backend::MockVmBackend, unavailable_vm_backend::UnavailableVmBackend,
        vm_backend::VmBackend, vm_error::VmError, vm_plugin_manifest::VmPluginManifest,
        vm_plugin_package::VmPluginPackage,
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
}
