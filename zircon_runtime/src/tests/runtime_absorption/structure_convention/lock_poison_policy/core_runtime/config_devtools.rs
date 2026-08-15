use super::*;

#[test]
fn runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store() {
    let config_store = read_runtime_src("core/runtime/config_store.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let config_store_doc = read_repo("docs/zircon_runtime/core/runtime/config_store.md");

    assert_contains_all(
        "ConfigStore poison recovery helper",
        &config_store,
        &[
            "fn lock_values(&self) -> MutexGuard<'_, HashMap<String, Value>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_values().insert(key.into(), value)",
            "self.lock_values().get(key).cloned()",
            "self.lock_values().clone()",
            "config_store_accessors_recover_poisoned_values_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("config store", &config_store);
}

#[test]
fn runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot() {
    let devtools = read_runtime_src("core/runtime/diagnostics/devtools.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");

    assert_contains_all(
        "runtime devtools poison recovery helper",
        &devtools,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let modules = lock_poison_recovered(&core.inner.modules);",
            "let services = lock_poison_recovered(&core.inner.services);",
            "lock_poison_recovered(&core.inner.devtools_plugin_catalog_entries).clone()",
            "devtools_snapshot_recovers_poisoned_runtime_registry_locks",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("runtime devtools diagnostics", &devtools);
}
