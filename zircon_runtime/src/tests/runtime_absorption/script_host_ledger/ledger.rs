use super::catalog::{
    combined_fixed_sources_contain_module, count_occurrences, fixed_sources_contain_function,
    missing_documented_functions, FIXED_HOST_FUNCTIONS, FIXED_HOST_MODULES, HOST_CAPABILITIES,
};

#[test]
fn host_function_registry_matches_documented_ledger() {
    let builtin_source = include_str!("../../../script/vm/host/builtin_host_modules.rs");
    let gameplay_source = include_str!("../../../script/vm/gameplay_host.rs");
    let bridge_source = include_str!("../../../script/vm/host/bridge_host_module.rs");
    let ledger =
        include_str!("../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let plan = concat!(
        include_str!(
            "../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
        ),
        include_str!(
            "../../../../../docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md"
        )
    );

    assert_eq!(
        count_occurrences(builtin_source, "HostExportFunction::new("),
        20,
        "builtin host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(gameplay_source, "HostExportFunction::new("),
        39,
        "gameplay host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(builtin_source, "#[crate::zircon_host_function("),
        2,
        "macro host function count changed; update function_ledger.md and Runtime 13 status"
    );

    for module in FIXED_HOST_MODULES {
        assert!(
            combined_fixed_sources_contain_module(builtin_source, gameplay_source, module),
            "fixed host module `{module}` should exist in host registration sources"
        );
        assert!(
            ledger.contains(module),
            "function ledger should document fixed host module `{module}`"
        );
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
        assert!(
            ledger.contains(capability),
            "function ledger should document host capability `{capability}`"
        );
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
        "6 host modules, 61 fixed host functions, and 2 fixed script type descriptors",
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

    for required_plan_anchor in [
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
        "builtin_callbacks=11",
        "gameplay_callbacks=39",
        "macro_host_functions=2",
    ] {
        assert!(
            plan.contains(required_plan_anchor),
            "Runtime 13 plan should record `{required_plan_anchor}`"
        );
    }
}

#[test]
fn host_function_registry_ledger_guard_rejects_missing_entry() {
    let ledger =
        include_str!("../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md")
            .replace("| `time_unix_millis` |", "| `time_unix_millis_removed` |");

    let missing_entries = missing_documented_functions(&ledger);

    assert!(
        missing_entries
            .iter()
            .any(|entry| entry == "zr.zircon.foundation.time_unix_millis"),
        "ledger guard negative self-check should reject a missing fixed host function"
    );
}
