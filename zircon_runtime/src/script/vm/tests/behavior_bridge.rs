use super::*;
use crate::script::{
    CapabilitySet, VmInterfaceCaller, VmPluginManagementPolicy, VmPluginManifest, VmPluginPackage,
    VM_BT_NODE_CAPABILITY,
};

#[test]
fn successful_callback_only_rewrites_cache_when_the_handle_changes() {
    let source = include_str!("../behavior_bridge.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("let resolved_handle = handle;"));
    assert!(source.contains("if result.is_ok() && handle != resolved_handle"));
}

#[test]
fn behavior_callback_cache_only_clones_a_missing_key() {
    let manager = VmPluginManager::mock();
    let package = package("cache");
    let slot = manager.load_package(package).unwrap();
    register_same_node(&manager, slot, 1);
    let bridge = VmScriptBehaviorBridge::new();
    bridge.bind_manager(&manager);
    let callback = ScriptBehaviorCallbackRef::parse("cache::shared.task").unwrap();
    let first = bridge.resolve_callback(&manager, &callback).unwrap();
    let refreshed = VmCallbackHandle {
        generation: first.generation + 1,
        ..first
    };

    assert!(!bridge.cache_callback(&callback, refreshed));
    assert_eq!(bridge.lock_callbacks().get(&callback), Some(&refreshed));

    let missing = ScriptBehaviorCallbackRef::parse("cache::missing.task").unwrap();
    assert!(bridge.cache_callback(&missing, first));
    assert_eq!(bridge.lock_callbacks().get(&missing), Some(&first));
}

#[test]
fn provider_qualified_callback_selects_slot_and_refreshes_generation() {
    let manager = VmPluginManager::mock();
    let first_package = package("first");
    let second_package = package("second");
    let first_slot = manager.load_package(first_package.clone()).unwrap();
    let second_slot = manager.load_package(second_package).unwrap();
    register_same_node(&manager, first_slot, 1);
    register_same_node(&manager, second_slot, 1);
    let bridge = VmScriptBehaviorBridge::new();
    bridge.bind_manager(&manager);

    let first_ref = ScriptBehaviorCallbackRef::parse("first::shared.task").unwrap();
    let second_ref = ScriptBehaviorCallbackRef::parse("second::shared.task").unwrap();
    let first = bridge.resolve_callback(&manager, &first_ref).unwrap();
    let second = bridge.resolve_callback(&manager, &second_ref).unwrap();
    assert_eq!(first.slot, first_slot);
    assert_eq!(second.slot, second_slot);

    manager.hot_reload_slot(first_slot, first_package).unwrap();
    register_same_node(&manager, first_slot, 2);
    let reloaded = bridge.resolve_callback(&manager, &first_ref).unwrap();
    assert_eq!(reloaded.slot, first_slot);
    assert_eq!(reloaded.generation, 2);
}

#[test]
fn duplicate_active_package_names_are_rejected_as_ambiguous() {
    let manager = VmPluginManager::mock();
    manager.load_package(package("duplicate")).unwrap();
    manager.load_package(package("duplicate")).unwrap();
    let bridge = VmScriptBehaviorBridge::new();
    bridge.bind_manager(&manager);

    let error = bridge
        .resolve_callback(
            &manager,
            &ScriptBehaviorCallbackRef::parse("duplicate::shared.task").unwrap(),
        )
        .unwrap_err();
    assert!(error.message.contains("ambiguous"), "{}", error.message);
}

fn register_same_node(
    manager: &VmPluginManager,
    slot: super::super::PluginSlotId,
    generation: u32,
) {
    let caller = VmInterfaceCaller::new(
        slot,
        generation,
        CapabilitySet::default().with(VM_BT_NODE_CAPABILITY),
    );
    manager
        .host_interfaces()
        .register_behavior_node(&caller, "shared.task", "Shared", "ai", "tick")
        .unwrap();
}

fn package(name: &str) -> VmPluginPackage {
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            entry: "main".to_string(),
            capabilities: CapabilitySet::default().with(VM_BT_NODE_CAPABILITY),
            management: VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    }
}
