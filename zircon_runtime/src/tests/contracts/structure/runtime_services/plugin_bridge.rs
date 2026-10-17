use super::*;

// TODO: [CR-RUNTIME-TESTS-STRUCT-0024] 插件桥接提供方槽位的锁恢复的静态源码锚点与当前归属不符；需追踪实际调用和新归属，判断契约回归还是守卫过时。
#[test]
fn runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot() {
    let table = read_runtime_src("plugin/bridge/table.rs");
    let _runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let _runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let _structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _plugin_bridge_doc = read_repo("docs/crates/zircon_runtime/plugin/bridge.md");

    let tests = read_runtime_src("plugin/bridge/tests/table.rs");
    assert_contains_all(
        "bridge generation and provider share one atomic state",
        &table,
        &[
            "use arc_swap::ArcSwap;",
            "state: ArcSwap<BridgeEntryState>",
            "struct BridgeEntryState",
            "generation: u32",
            "provider: Option<Arc<dyn Any + Send + Sync>>",
            "ArcSwap::from_pointee(BridgeEntryState",
        ],
    );
    let provider = table
        .split("fn provider<T>(")
        .nth(1)
        .and_then(|tail| tail.split("fn set_enabled(").next())
        .expect("typed bridge provider owner");
    assert_contains_all(
        "bridge provider observes generation and provider together",
        provider,
        &[
            "let state = self.state.load();",
            "if state.generation % 2 != 0",
            "let provider = state",
            "Ok((state.generation, (*provider).clone()))",
        ],
    );
    for (method, next) in [
        ("fn deactivate(", "fn replace_provider<"),
        ("fn replace_erased_provider(", "fn restore_provider("),
        ("fn restore_provider(", "fn snapshot_state("),
    ] {
        let body = table
            .split(method)
            .nth(1)
            .and_then(|tail| tail.split(next).next())
            .expect("bridge atomic transition owner");
        assert_contains_all(
            "bridge transition publishes one complete state",
            body,
            &[
                "self.state.rcu(|current|",
                "Arc::new(BridgeEntryState",
                "generation,",
                "provider:",
            ],
        );
    }
    assert_contains_all(
        "bridge lifecycle retains original observable provider transitions",
        &tests,
        &[
            "fn bridge_entry_publishes_generation_and_provider_as_one_state(",
            "erased_provider(7)",
            "entry.deactivate();",
            "assert!(!entry.provider_installed());",
            "entry.restore_provider(erased_provider(11));",
            "assert_eq!(provider.sample(), 7);",
            "assert_eq!(provider.sample(), 11);",
            "assert_eq!(provider.sample(), 13);",
        ],
    );
    assert!(
        !table.contains("lock_provider") && !table.contains("Mutex<"),
        "bridge provider must not regain a separately poisoned provider authority"
    );
    assert_no_direct_lock_unwrap_in_production("plugin bridge table", &table);
    assert!(
        !production_section(&table).contains("lock poisoned"),
        "plugin bridge table production code should recover poisoned provider locks"
    );
}
