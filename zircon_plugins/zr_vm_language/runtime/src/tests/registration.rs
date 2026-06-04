use crate::{
    module_descriptor, plugin_registration, register_zr_vm_backend,
    ZrVmLanguageBackendRegistration, ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
    ZR_VM_LANGUAGE_MODULE_NAME, ZR_VM_PROJECT_BACKEND_SELECTOR,
};

#[test]
fn zr_vm_language_registration_reports_backend_capability() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == ZR_VM_LANGUAGE_MODULE_NAME));
    assert!(report
        .package_manifest
        .capabilities
        .contains(&"runtime.script.backend.zr_vm_project".to_string()));
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    for capability in [
        "runtime.plugin.zr_vm_language",
        "runtime.script.backend.zr_vm_project",
    ] {
        assert!(report
            .package_manifest
            .capabilities
            .contains(&capability.to_string()));
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == capability
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
    }
}

#[test]
fn zr_vm_backend_family_resolves_project_selector() {
    let manager = zircon_runtime::script::VmPluginManager::mock();
    register_zr_vm_backend(&manager);

    assert!(manager
        .backend_names()
        .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
    manager
        .select_default_backend(ZR_VM_PROJECT_BACKEND_SELECTOR)
        .unwrap();
    assert_eq!(
        manager.selected_backend_name(),
        ZR_VM_PROJECT_BACKEND_SELECTOR
    );
}

#[test]
fn zr_vm_runtime_module_registers_backend_with_vm_manager() {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime
        .register_module(zircon_runtime::script::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime
        .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
        .unwrap();
    runtime.activate_module(ZR_VM_LANGUAGE_MODULE_NAME).unwrap();

    let registration = runtime
        .handle()
        .resolve_plugin::<ZrVmLanguageBackendRegistration>(ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME)
        .unwrap();
    let manager = runtime
        .handle()
        .resolve_manager::<zircon_runtime::script::VmPluginManager>(
            zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
        )
        .unwrap();

    assert_eq!(registration.selector, "zr_vm");
    assert!(manager
        .backend_names()
        .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
}
