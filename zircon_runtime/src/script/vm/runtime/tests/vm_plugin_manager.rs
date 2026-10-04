use super::*;

#[test]
fn builtin_manager_retains_its_discovery_runtime_owner() {
    let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());

    assert!(manager.base_plugin_context().core.upgrade().is_some());
}

#[test]
fn stale_plugin_context_rejects_discovery_without_a_process_pool_fallback() {
    let runtime = CoreRuntime::new();
    let plugin_context = PluginContext {
        plugin_name: VM_PLUGIN_RUNTIME_NAME.to_string(),
        core: runtime.handle().downgrade(),
        package_root: None,
        source_root: None,
        data_root: None,
    };
    drop(runtime);
    let manager = VmPluginManager::with_plugin_context(plugin_context, HostRegistry::default());

    let error = manager
        .submit_package_discovery(".")
        .expect_err("stale runtime context must not fall back to a process I/O pool");

    assert!(error
        .to_string()
        .contains("runtime task owner is unavailable"));
}

#[test]
fn discovery_rejects_after_an_external_runtime_owner_expires() {
    let runtime = CoreRuntime::new();
    let plugin_context = PluginContext {
        plugin_name: VM_PLUGIN_RUNTIME_NAME.to_string(),
        core: runtime.handle().downgrade(),
        package_root: None,
        source_root: None,
        data_root: None,
    };
    let manager = VmPluginManager::with_plugin_context(plugin_context, HostRegistry::default());
    drop(runtime);

    let error = manager
        .submit_package_discovery(".")
        .expect_err("expired runtime owner must close discovery admission");

    assert!(error
        .to_string()
        .contains("runtime task owner is unavailable"));
}

#[test]
fn vm_discovery_worker_has_no_process_global_constructor() {
    let source = include_str!("../../plugin/vm_plugin_package_discovery/io.rs");

    for forbidden in [
        "TaskPools::process_default",
        "JobScheduler::process_io",
        "impl Default for VmPluginDiscoveryWorker",
        "pub(crate) fn new(limits: VmPluginDiscoveryLimits)",
        "pub(crate) fn with_io_pool",
    ] {
        assert!(
            !source.contains(forbidden),
            "VM discovery worker must not retain process fallback `{forbidden}`"
        );
    }
}

#[test]
fn callback_and_system_dispatch_avoid_wide_record_clones() {
    let source = include_str!("../vm_plugin_manager.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let callback = source.split("pub fn invoke_callback").nth(1).unwrap();
    let callback = callback
        .split("pub fn run_registered_systems")
        .next()
        .unwrap();
    let systems = source
        .split("pub fn run_registered_systems")
        .nth(1)
        .unwrap();
    let systems = systems.split("pub fn registered_systems").next().unwrap();

    assert!(callback.contains(".coordinator"));
    assert!(callback.contains(".generation(handle.slot)"));
    assert!(!callback.contains("self.slot(handle.slot)"));
    assert!(systems.contains("let system_count = systems.len();"));
    // BUG: [CR-SCRIPT-AUDIT-0004] 此断言仍要求旧的所有权循环文本，生产派发已改用借用迭代；截取的派发区间不含目标子串，因此测试必败。
    assert!(systems.contains("for mut system in systems"));
    assert!(!systems.contains("systems.iter().cloned()"));
}

#[test]
fn vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock() {
    let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
    let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut selected = manager.selected_backend.write().unwrap();
        *selected = Arc::from(DEFAULT_BACKEND_SELECTOR);
        panic!("poison vm plugin manager selected backend lock");
    }));
    assert!(poison.is_err());

    assert_eq!(manager.selected_backend_name(), DEFAULT_BACKEND_SELECTOR);
    manager
        .select_default_backend("builtin:mock")
        .expect("poisoned selected backend lock should recover for writes");
    assert_eq!(manager.selected_backend_name(), "builtin:mock");
}
