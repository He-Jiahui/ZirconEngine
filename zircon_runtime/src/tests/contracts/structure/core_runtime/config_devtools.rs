use super::*;

// TODO: [CR-RUNTIME-TESTS-STRUCT-0019] 运行时配置与开发工具诊断的锁恢复的静态源码锚点与当前归属不符；需追踪实际调用和新归属，判断契约回归还是守卫过时。
#[test]
fn runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store() {
    let config_store = read_runtime_src("core/runtime/config_store.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _config_store_doc = read_repo("docs/crates/zircon_runtime/core/runtime/config_store.md");

    assert_contains_all(
        "ConfigStore poison recovery helper",
        &config_store,
        &[
            "fn lock_values(&self) -> MutexGuard<'_, HashMap<String, Arc<Value>>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut values = self.lock_values();",
            "values.insert(key, value)",
            "self.lock_values().get(key).cloned()",
            "shared_snapshot_entries(&self.lock_values())",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("config store", &config_store);
    let recovery_tests = read_runtime_src("core/runtime/tests/config_store.rs");
    assert_contains_all(
        "mounted poison recovery regression",
        &recovery_tests,
        &["fn config_store_accessors_recover_poisoned_values_lock("],
    );
}

#[test]
fn runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot() {
    let devtools = read_runtime_src("core/runtime/diagnostics/devtools.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _diagnostics_doc = read_repo("docs/crates/zircon_runtime/core/diagnostics.md");

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
        ],
    );
    assert_no_direct_lock_unwrap_in_production("runtime devtools diagnostics", &devtools);
    let recovery_tests = read_runtime_src("core/runtime/diagnostics/tests/devtools.rs");
    assert_contains_all(
        "mounted poison recovery regression",
        &recovery_tests,
        &["fn devtools_snapshot_recovers_poisoned_runtime_registry_locks("],
    );
}
