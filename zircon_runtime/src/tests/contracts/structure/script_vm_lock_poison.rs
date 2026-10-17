use super::rust_source_view::production_code_view;
use super::{assert_contains_all, assert_contains_all_exact, runtime_src_path};

const READ_UNWRAP_CALL: &str = concat!(".read().", "unwrap()");
const WRITE_UNWRAP_CALL: &str = concat!(".write().", "unwrap()");

// TODO: [CR-RUNTIME-TESTS-STRUCT-0138] 脚本虚拟机后端选择器的锁中毒策略的静态源码锚点与当前归属不符；需追踪实际调用和新归属，判断契约回归还是守卫过时。
#[test]
fn runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector(
) {
    let vm_plugin_manager = read_runtime_src("script/vm/runtime/vm_plugin_manager.rs");
    let structure_parent = read_runtime_src("tests/contracts/structure.rs");

    assert_contains_all_exact(
        "VM plugin manager selected-backend poison recovery",
        &vm_plugin_manager,
        &[
            "use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};",
            "fn selected_backend_read(&self) -> RwLockReadGuard<'_, Arc<str>>",
            "fn selected_backend_write(&self) -> RwLockWriteGuard<'_, Arc<str>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "Arc::clone(&self.selected_backend_read())",
            "*self.selected_backend_write() = Arc::from(backend_name);",
        ],
    );
    assert_contains_all_exact(
        "script VM lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure/script_vm_lock_poison.rs\"]",
            "mod script_vm_lock_poison;",
        ],
    );
    assert_no_direct_rwlock_unwrap_in_production("VM plugin manager", &vm_plugin_manager);
    let recovery_tests = read_runtime_src("script/vm/runtime/tests/vm_plugin_manager.rs");
    assert_contains_all(
        "mounted poison recovery regression",
        &recovery_tests,
        &["fn vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock("],
    );
}

fn assert_no_direct_rwlock_unwrap_in_production(label: &str, source: &str) {
    let production = production_code_view(source);
    assert!(
        !production.contains(READ_UNWRAP_CALL),
        "{label} production code should use poison-safe read helpers instead of {READ_UNWRAP_CALL}"
    );
    assert!(
        !production.contains(WRITE_UNWRAP_CALL),
        "{label} production code should use poison-safe write helpers instead of {WRITE_UNWRAP_CALL}"
    );
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}
