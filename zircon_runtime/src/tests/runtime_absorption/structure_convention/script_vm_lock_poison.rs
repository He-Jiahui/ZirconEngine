use super::rust_source_view::production_code_view;
use super::{assert_contains_all_exact, runtime_src_path};

const READ_UNWRAP_CALL: &str = concat!(".read().", "unwrap()");
const WRITE_UNWRAP_CALL: &str = concat!(".write().", "unwrap()");

#[test]
fn runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector(
) {
    let vm_plugin_manager = read_runtime_src("script/vm/runtime/vm_plugin_manager.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");

    assert_contains_all_exact(
        "VM plugin manager selected-backend poison recovery",
        &vm_plugin_manager,
        &[
            "use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};",
            "fn selected_backend_read(&self) -> RwLockReadGuard<'_, String>",
            "fn selected_backend_write(&self) -> RwLockWriteGuard<'_, String>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.selected_backend_read().clone()",
            "*self.selected_backend_write() = backend_name.to_string();",
            "vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock",
        ],
    );
    assert_contains_all_exact(
        "script VM lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/script_vm_lock_poison.rs\"]",
            "mod script_vm_lock_poison;",
        ],
    );
    assert_no_direct_rwlock_unwrap_in_production("VM plugin manager", &vm_plugin_manager);
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
