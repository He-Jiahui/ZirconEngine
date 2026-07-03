use super::{assert_contains_all, repo_path, runtime_src_path};

const READ_UNWRAP_CALL: &str = concat!(".read().", "unwrap()");
const WRITE_UNWRAP_CALL: &str = concat!(".write().", "unwrap()");

#[test]
fn runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector(
) {
    let vm_plugin_manager = read_runtime_src("script/vm/runtime/vm_plugin_manager.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_vm_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
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
    assert_contains_all(
        "script VM lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/script_vm_lock_poison.rs\"]",
            "mod script_vm_lock_poison;",
        ],
    );
    assert_no_direct_rwlock_unwrap_in_production("VM plugin manager", &vm_plugin_manager);

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM doc", script_vm_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
                "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred",
                "script/vm/runtime/vm_plugin_manager.rs",
                "vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock",
                "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector",
            ],
        );
    }
}

fn assert_no_direct_rwlock_unwrap_in_production(label: &str, source: &str) {
    let production = production_section(source);
    assert!(
        !production.contains(READ_UNWRAP_CALL),
        "{label} production code should use poison-safe read helpers instead of {READ_UNWRAP_CALL}"
    );
    assert!(
        !production.contains(WRITE_UNWRAP_CALL),
        "{label} production code should use poison-safe write helpers instead of {WRITE_UNWRAP_CALL}"
    );
}

fn production_section(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
