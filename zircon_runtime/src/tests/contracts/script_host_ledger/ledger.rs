use super::catalog::{
    count_occurrences, fixed_sources_contain_function, missing_documented_functions,
    FIXED_HOST_FUNCTIONS, FIXED_HOST_MODULES, HOST_CAPABILITIES,
};
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;

#[test]
fn host_function_registry_matches_documented_ledger() {
    let builtin_source = include_str!("../../../script/vm/host/builtin_host_modules.rs");
    let gameplay_source = include_str!("../../../script/vm/gameplay_host.rs");
    let bridge_source = include_str!("../../../script/vm/host/bridge_host_module.rs");
    let ledger =
        include_str!("../../../../../docs/crates/zircon_runtime/script/vm/host/function_ledger.md");
    let historical = history("docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md");
    let current_inventory = ledger
        .lines()
        .find(|line| line.starts_with("Runtime13 current source contract inventory:"))
        .expect("the canonical ledger must have a current source inventory");
    for required in [
        "fixed_host_function_count = 74",
        "builtin_callback_count = 21",
        "gameplay_callback_count = 51",
        "host_capability_count = 14",
    ] {
        assert!(current_inventory.contains(required), "inventory {required}");
    }

    assert_eq!(
        count_occurrences(builtin_source, "HostExportFunction::new("),
        21,
        "builtin host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(gameplay_source, "HostExportFunction::new("),
        51,
        "gameplay host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(builtin_source, "#[crate::zircon_host_function("),
        2,
        "macro host function count changed; update function_ledger.md and Runtime 13 status"
    );

    for module in FIXED_HOST_MODULES {
        assert!(
            builtin_source.contains(module) || gameplay_source.contains(module),
            "fixed host module `{module}` should exist in host registration sources"
        );
        assert!(ledger.contains(module), "module {module}");
    }

    for (module, function) in FIXED_HOST_FUNCTIONS {
        assert!(
            fixed_sources_contain_function(builtin_source, gameplay_source, function),
            "fixed host function `{module}.{function}` should exist in registration sources"
        );
        assert!(
            ledger.contains(&format!("| `{function}` |"))
                || ledger.contains(&format!("| Type `{function}` |")),
            "function ledger should document fixed host function `{module}.{function}`"
        );
    }

    for capability in HOST_CAPABILITIES {
        assert!(ledger.contains(capability), "capability {capability}");
    }

    for required_bridge_anchor in [
        "pub const BRIDGE_HOST_MODULE: &str = \"zr.zircon.bridge\";",
        "pub const BRIDGE_HOST_CAPABILITY: &str = \"bridge.call\";",
        "ScriptBridgeMethodDescriptor",
        "register_bridge_host_module",
    ] {
        assert!(
            bridge_source.contains(required_bridge_anchor),
            "bridge host source should keep dynamic module anchor `{required_bridge_anchor}`"
        );
    }

    for required_ledger_anchor in [
        "6 host modules, 74 fixed host functions, and 2 fixed script type descriptors",
        "Version 0.3.0 deterministic value and scalar math ABI.",
        "`zr.zircon.bridge`",
        "dynamic module shape contract",
        "Value descriptors",
        "Host handles",
        "Serialized payloads",
        "ZrHostEcsApiV1",
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
    ] {
        assert!(
            ledger.contains(required_ledger_anchor),
            "function ledger should record `{required_ledger_anchor}`"
        );
    }

    for required_history_anchor in [
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
        "builtin_callbacks=11",
        "gameplay_callbacks=39",
        "macro_host_functions=2",
    ] {
        assert!(
            historical.contains(required_history_anchor),
            "fixed Runtime13 history should record `{required_history_anchor}`"
        );
    }
}

#[test]
fn host_function_registry_ledger_guard_rejects_missing_entry() {
    let ledger =
        include_str!("../../../../../docs/crates/zircon_runtime/script/vm/host/function_ledger.md")
            .replace("| `time_unix_millis` |", "| `time_unix_millis_removed` |");

    let missing_entries = missing_documented_functions(&ledger);

    assert!(
        missing_entries
            .iter()
            .any(|entry| entry == "zr.zircon.foundation.time_unix_millis"),
        "ledger guard negative self-check should reject a missing fixed host function"
    );
}
