use std::sync::Arc;

use zircon_runtime::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostValue,
};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::script::{
    CapabilitySet, VmInterfaceCaller, VmPluginManagementPolicy, VmPluginManager, VmPluginManifest,
    VmPluginPackage, VmScriptBehaviorBridge, SCRIPT_MODULE_NAME, VM_BT_NODE_CAPABILITY,
    VM_PLUGIN_MANAGER_NAME,
};

#[test]
fn unbound_bridge_rejects_invocation() {
    let error = VmScriptBehaviorBridge::new()
        .invoke(&callback(), &[])
        .unwrap_err();
    assert!(error.message.contains("not bound"), "{}", error.message);
}

#[test]
fn deactivation_and_reactivation_require_a_new_manager_binding() {
    // Keep an external Arc alive: service admission, rather than Arc liveness,
    // must decide whether this manager can receive another invocation.
    let manager = manager_with_package(true);
    let runtime = runtime_with_manager(&manager);
    let bridge = VmScriptBehaviorBridge::new();
    let first = runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    assert_eq!(
        first,
        runtime
            .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .unwrap()
    );
    bridge.bind_manager(first.clone());
    assert_eq!(
        bridge.invoke(&callback(), &[]).unwrap(),
        Some(ScriptHostValue::Null)
    );
    bridge.bind_manager(first.clone());
    assert!(bridge.invoke(&callback(), &[]).is_ok());

    // This also verifies that an idle bridge holds no call guard that would
    // prevent deactivation from draining.
    runtime.deactivate_module(SCRIPT_MODULE_NAME).unwrap();
    assert!(bridge.invoke(&callback(), &[]).is_err());
    runtime.activate_module(SCRIPT_MODULE_NAME).unwrap();
    assert!(bridge.invoke(&callback(), &[]).is_err());

    let next = runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    assert_ne!(first, next);
    bridge.bind_manager(next);
    assert!(bridge.invoke(&callback(), &[]).is_ok());
}

#[test]
fn idle_bridge_retains_neither_the_runtime_nor_the_manager() {
    let manager = manager_with_package(true);
    let weak_manager = Arc::downgrade(&manager);
    let runtime = runtime_with_manager(&manager);
    let weak_runtime = runtime.handle().downgrade();
    let bridge = VmScriptBehaviorBridge::new();
    bridge.bind_manager(
        runtime
            .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .unwrap(),
    );
    assert!(bridge.invoke(&callback(), &[]).is_ok());

    drop(manager);
    drop(runtime);
    assert!(weak_runtime.upgrade().is_none());
    assert!(weak_manager.upgrade().is_none());
    assert!(bridge.invoke(&callback(), &[]).is_err());
}

#[test]
fn switching_runtimes_does_not_reuse_the_previous_managers_callback_cache() {
    let first_manager = manager_with_package(true);
    let next_manager = manager_with_package(false);
    let first_runtime = runtime_with_manager(&first_manager);
    let next_runtime = runtime_with_manager(&next_manager);
    let first = first_runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    let next = next_runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    // The registry name, slot and generation match; runtime identity differs.
    assert_ne!(first, next);
    let bridge = VmScriptBehaviorBridge::new();
    bridge.bind_manager(first.clone());
    assert!(bridge.invoke(&callback(), &[]).is_ok());

    bridge.bind_manager(next);
    let error = bridge.invoke(&callback(), &[]).unwrap_err();
    assert!(
        error.message.contains("is not registered"),
        "the new manager must resolve its own registrations: {}",
        error.message
    );
    bridge.bind_manager(first);
    assert!(bridge.invoke(&callback(), &[]).is_ok());
}

fn callback() -> ScriptBehaviorCallbackRef {
    ScriptBehaviorCallbackRef::parse("bridge-test::shared.task").unwrap()
}

fn runtime_with_manager(manager: &Arc<VmPluginManager>) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    let manager = Arc::clone(manager);
    runtime
        .register_module(
            ModuleDescriptor::new(SCRIPT_MODULE_NAME, "behavior bridge lifecycle regression")
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        SCRIPT_MODULE_NAME,
                        ServiceKind::Manager,
                        "VmPluginManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| Ok(Arc::clone(&manager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime.activate_module(SCRIPT_MODULE_NAME).unwrap();
    runtime
}

fn manager_with_package(register_node: bool) -> Arc<VmPluginManager> {
    let manager = VmPluginManager::mock();
    let capabilities = CapabilitySet::default().with(VM_BT_NODE_CAPABILITY);
    let slot = manager
        .load_package(VmPluginPackage {
            manifest: VmPluginManifest {
                name: "bridge-test".to_string(),
                version: "1.0.0".to_string(),
                entry: "main".to_string(),
                capabilities: capabilities.clone(),
                management: VmPluginManagementPolicy::default(),
            },
            zr_vm_project: None,
            bytecode: vec![1, 2, 3],
        })
        .unwrap();
    if register_node {
        let caller = VmInterfaceCaller::new(slot, 1, capabilities);
        manager
            .host_interfaces()
            .register_behavior_node(&caller, "shared.task", "Shared", "ai", "tick")
            .unwrap();
    }
    manager
}
