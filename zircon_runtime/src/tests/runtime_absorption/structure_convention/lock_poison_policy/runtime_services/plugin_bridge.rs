use super::*;

#[test]
fn runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot() {
    let table = read_runtime_src("plugin/bridge/table.rs");
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");

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
}
