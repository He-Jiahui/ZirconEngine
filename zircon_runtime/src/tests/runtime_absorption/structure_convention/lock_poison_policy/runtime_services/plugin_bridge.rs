use super::*;

#[test]
fn runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot() {
    let table = read_runtime_src("plugin/bridge/table.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "Plugin bridge table provider poison recovery helper",
        &table,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_provider(&self) -> MutexGuard<'_, Option<Arc<dyn Any + Send",
            "+ Sync>>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_provider().is_some()",
            ".lock_provider()",
            "*self.lock_provider() = None;",
            "*self.lock_provider() = Some(provider);",
            "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("plugin bridge table", &table);
    assert!(
        !production_section(&table).contains("lock poisoned"),
        "plugin bridge table production code should recover poisoned provider locks"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 plugin bridge table lock poison recovery",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred",
                "plugin/bridge/table.rs",
                "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot",
            ],
        );
    }
}
