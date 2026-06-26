use super::support::*;
use super::*;

#[test]
fn builtin_backend_family_accepts_qualified_and_legacy_backend_names() {
    let registry = super::super::VmBackendRegistry::new();
    registry.register_family(Arc::new(BuiltinVmBackendFamily));

    assert!(registry.resolve("builtin:mock").is_ok());
    assert!(registry.resolve("mock").is_ok());
    assert!(registry.resolve("builtin:unavailable").is_ok());
    assert!(registry.resolve("unavailable").is_ok());
}

#[test]
fn hot_reload_coordinator_tracks_slot_lifecycle_records() {
    let coordinator = HotReloadCoordinator::new();
    let package_root = std::env::temp_dir().join("zircon-script-slot-lifecycle");
    let source = VmPluginPackageSource {
        package_root: Some(package_root.clone()),
        manifest_path: Some(package_root.join("plugin.toml")),
        bytecode_path: Some(package_root.join("plugin.bin")),
        zr_vm_project_path: None,
    };
    let package = test_package("sample", "0.1.0");
    let host = test_host_context(
        VM_PLUGIN_RUNTIME_NAME,
        "mock",
        source.clone(),
        package.manifest.capabilities.clone(),
    );

    let slot = coordinator
        .load_package("mock", &MockVmBackend, package, &host)
        .unwrap();
    let initial = coordinator.slot(slot).unwrap();
    assert_eq!(initial.backend_name, "mock");
    assert_eq!(initial.state, super::super::VmPluginSlotState::Active);
    assert_eq!(initial.generation, 1);
    assert_eq!(initial.source, source);
    assert_eq!(initial.manifest.version, "0.1.0");
    assert_eq!(initial.management, initial.manifest.management);

    coordinator
        .hot_reload(
            slot,
            "mock",
            &MockVmBackend,
            test_package("sample", "0.2.0"),
            &host,
        )
        .unwrap();

    let reloaded = coordinator.slot(slot).unwrap();
    assert_eq!(reloaded.manifest.version, "0.2.0");
    assert_eq!(reloaded.generation, 2);
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
    assert!(discovered.source.zr_vm_project_path.is_none());

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
fn vm_plugin_manager_calls_exports_by_loaded_package_name() {
    let manager = VmPluginManager::mock();
    let slot = manager
        .load_package(test_package("gameplay", "0.1.0"))
        .unwrap();

    let returned = manager
        .call_package_export(
            "gameplay",
            "player",
            "onUpdate",
            &[ScriptHostValue::Int(7), ScriptHostValue::Float(0.25)],
        )
        .unwrap();

    assert_eq!(returned, Some(ScriptHostValue::Null));
    assert_eq!(manager.slot_for_package_name("gameplay").unwrap(), slot);
}

#[test]
fn unavailable_backend_reports_error() {
    let backend = UnavailableVmBackend;
    let source = VmPluginPackageSource::default();
    let package = test_package("sample", "0.1.0");
    let host = test_host_context(
        VM_PLUGIN_RUNTIME_NAME,
        "builtin:unavailable",
        source,
        package.manifest.capabilities.clone(),
    );
    let error = match backend.load_package(&package, &host) {
        Ok(_) => panic!("expected unavailable backend to reject package"),
        Err(error) => error,
    };
    assert!(matches!(error, VmError::BackendUnavailable(_)));
}

#[test]
fn vm_plugin_manager_propagates_host_context_roots_and_backend_selector() {
    let fixture = PluginFixture::new("sample", "0.1.0", "recording:capture", &[1, 2, 3]);
    let observations = Arc::new(Mutex::new(Vec::<ObservedHostContext>::new()));
    let runtime = CoreRuntime::new();
    let base_plugin_context = PluginContext {
        plugin_name: VM_PLUGIN_RUNTIME_NAME.to_string(),
        core: runtime.handle().downgrade(),
        package_root: None,
        source_root: None,
        data_root: None,
    };
    let manager =
        VmPluginManager::with_plugin_context(base_plugin_context, HostRegistry::default());
    manager.register_family(Arc::new(RecordingVmBackendFamily::new(Arc::clone(
        &observations,
    ))));

    let packages = manager.discover_packages(&fixture.root).unwrap();
    let slot = manager.load_discovered_package(&packages[0]).unwrap();
    let record = manager.slot(slot).unwrap();
    let expected_data_root = fixture.package_root.join("data");

    assert_eq!(record.backend_name, "recording:capture");
    assert_eq!(
        record.source.manifest_path.as_deref(),
        Some(fixture.manifest_path.as_path())
    );
    assert_eq!(
        record.source.bytecode_path.as_deref(),
        Some(fixture.bytecode_path.as_path())
    );
    assert!(record.source.zr_vm_project_path.is_none());

    let observed = observations.lock().unwrap().clone();
    assert_eq!(observed.len(), 2);
    for host in observed {
        assert_eq!(host.plugin_name, VM_PLUGIN_RUNTIME_NAME);
        assert_eq!(host.backend_selector, "recording:capture");
        assert_eq!(
            host.package_root.as_deref(),
            Some(fixture.package_root.as_path())
        );
        assert_eq!(
            host.source_root.as_deref(),
            Some(fixture.package_root.as_path())
        );
        assert_eq!(
            host.data_root.as_deref(),
            Some(expected_data_root.as_path())
        );
        assert_eq!(host.package_source, record.source);
        assert_eq!(host.capabilities, record.manifest.capabilities);
    }
}

#[test]
fn vm_plugin_discovery_supports_zr_vm_project_packages_without_bytecode() {
    let fixture = ZrVmProjectFixture::new("sample_zr", "0.1.0");
    let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
    let packages = manager.discover_packages(&fixture.root).unwrap();

    assert_eq!(packages.len(), 1);
    let discovered = &packages[0];
    assert_eq!(discovered.backend_name, "zr_vm:project");
    assert!(discovered.package.bytecode.is_empty());
    assert_eq!(
        discovered.source.zr_vm_project_path.as_deref(),
        Some(fixture.project_path.as_path())
    );
    assert_eq!(
        discovered
            .package
            .zr_vm_project
            .as_ref()
            .unwrap()
            .entry_module,
        "main"
    );
    assert_eq!(
        discovered
            .package
            .zr_vm_project
            .as_ref()
            .unwrap()
            .project_path,
        fixture.project_path
    );
    assert!(discovered.source.bytecode_path.is_none());
}
