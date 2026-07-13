use super::support::*;
use super::*;

#[test]
fn builtin_host_modules_register_gameplay_capabilities() {
    let exports = HostExportRegistry::default();
    super::super::register_builtin_host_modules(&exports, &HostRegistry::default()).unwrap();

    let gameplay = exports.module("zr.zircon.gameplay").unwrap();
    assert!(gameplay
        .descriptor
        .capabilities
        .contains(&"gameplay.input".to_string()));
    assert!(gameplay
        .descriptor
        .capabilities
        .contains(&"gameplay.entity".to_string()));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "key_pressed"));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "nav_next_point_json"));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "nearest_by_script_property"));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "damage_entity"));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "set_world_hud_bar"));
    assert!(gameplay
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "set_animation_bool"));
    assert!(
        !gameplay
            .descriptor
            .functions
            .iter()
            .any(|function| function.name == "vampire_start" || function.name == "vampire_tick"),
        "gameplay host should expose generic gameplay calls, not vampire-specific Rust delegates"
    );
}

#[test]
fn script_module_descriptor_registers_vm_plugin_runtime_before_manager_facade() {
    let descriptor = module_descriptor();

    let plugin = descriptor
        .plugins
        .iter()
        .find(|plugin| plugin.name.as_str() == VM_PLUGIN_RUNTIME_NAME)
        .expect("vm plugin runtime descriptor");
    assert_eq!(plugin.startup_mode, crate::core::StartupMode::Immediate);
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

    let capability = driver
        .registry()
        .register_capability("RenderingManager")
        .unwrap();
    assert!(plugin.host_registry().is_valid(capability));
    assert!(driver.host_exports().module("zr.zircon.math").is_some());
    assert!(plugin
        .host_exports()
        .module("zr.zircon.foundation")
        .is_some());
    assert_eq!(
        plugin.base_plugin_context().plugin_name,
        VM_PLUGIN_RUNTIME_NAME
    );

    plugin.select_default_backend("builtin:mock").unwrap();
    let slot = plugin.load_package(test_package("core", "0.1.0")).unwrap();
    assert_eq!(plugin.slot(slot).unwrap().backend_name, "builtin:mock");
}

#[test]
fn vm_plugin_protocol_types_live_in_script_subsystem() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manager_root = runtime_root.join("src/core/manager");
    let script_mod_source = include_str!("../../mod.rs");
    let vm_mod_source = include_str!("../mod.rs");
    let manifest_source = include_str!("../plugin/vm_plugin_manifest.rs");
    let host_registry_source = include_str!("../host/host_registry.rs");
    let package_discovery_source = include_str!("../plugin/vm_plugin_package_discovery.rs");
    let hot_reload_source = include_str!("../runtime/hot_reload_coordinator.rs");
    let manager_mod_source = include_str!("../../../core/manager/mod.rs");
    let manager_resolver_source = include_str!("../../../core/manager/resolver.rs");
    let manager_records_root = manager_root.join("records");

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
            !source.contains("use crate::core::manager::"),
            "vm runtime files should not source script protocol types from zircon_manager"
        );
    }

    for forbidden in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
        assert!(
            !manager_mod_source.contains(forbidden),
            "core manager mod.rs should not re-export {forbidden} after vm plugin boundary cleanup"
        );
        assert!(
            !manager_resolver_source.contains(forbidden),
            "core manager resolver should not re-export {forbidden} after vm plugin boundary cleanup"
        );
    }
    assert!(
        !runtime_root.join("src/manager").exists(),
        "runtime root should not keep a legacy manager module after vm plugin boundary cleanup"
    );
    assert!(
        !manager_records_root.exists(),
        "core manager should not grow a records subtree after vm plugin boundary cleanup"
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
        "backend/builtin_vm_backend_family.rs",
        "backend/vm_backend.rs",
        "backend/vm_backend_family.rs",
        "backend/unavailable_vm_backend.rs",
        "backend/mock_vm_backend.rs",
        "backend/vm_error.rs",
        "gc_bridge/mod.rs",
        "gc_bridge/host_handle.rs",
        "gc_bridge/vm_object_ref.rs",
        "gc_bridge/budget.rs",
        "host/mod.rs",
        "host/bridge_host_module.rs",
        "host/builtin_host_modules.rs",
        "host/host_export_registry.rs",
        "host/host_registry.rs",
        "host/plugin_host_driver.rs",
        "host/constants.rs",
        "host/script_call_table.rs",
        "host/vm_plugin_host_context.rs",
        "host/vm_plugin_slot_lifecycle.rs",
        "gameplay_host.rs",
        "gameplay_host/script_bindings.rs",
        "plugin/mod.rs",
        "plugin/management_policy/mod.rs",
        "plugin/management_policy/policy.rs",
        "plugin/management_policy/hot_reload.rs",
        "plugin/management_policy/garbage_collection.rs",
        "plugin/management_policy/memory.rs",
        "plugin/vm_plugin_manifest.rs",
        "plugin/vm_plugin_package.rs",
        "plugin/vm_plugin_package_source.rs",
        "plugin/vm_plugin_package_discovery.rs",
        "plugin/vm_plugin_instance.rs",
        "plugin/vm_state_blob.rs",
        "runtime/mod.rs",
        "runtime/hot_reload_coordinator.rs",
        "runtime/vm_plugin_slot_record.rs",
        "runtime/vm_plugin_slot_state.rs",
        "runtime/vm_plugin_manager.rs",
        "runtime_context.rs",
        "scene_hook.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected vm module {relative} under {:?}",
            root
        );
    }

    for forbidden in [
        "backend/zr_vm_project_fallback_backend.rs",
        "backend/zr_vm_project_fallback_backend",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "vm runtime must not keep project-specific fallback backend path {forbidden}"
        );
    }
}
