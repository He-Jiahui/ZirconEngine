#[path = "mirror_docs/current_inventory.rs"]
mod current_inventory;
use self::current_inventory::unlabelled_host_inventories;

use std::path::Path;

use super::inventory::{
    EXPECTED_RUNTIME_13_SOURCE_FILES, EXPECTED_RUNTIME_13_TEST_FILES, GAMEPLAY_TEST_MAX_LINES,
    RUNTIME_13_GUARD_ANCHORS, SCRIPT_BINDING_MIRROR_DOC_ANCHORS, SCRIPT_LEDGER_TEST_MAX_LINES,
};
use super::support::{assert_file_line_budget, assert_files_exist, count_occurrences};

#[test]
fn runtime_13_script_binding_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_13_SOURCE_FILES.len(), 31);
    assert_eq!(EXPECTED_RUNTIME_13_TEST_FILES.len(), 3);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_13_SOURCE_FILES,
        "Runtime 13 script binding source",
    );
    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_13_TEST_FILES,
        "Runtime 13 script binding guard/test",
    );
    assert_file_line_budget(
        runtime_root,
        "src/tests/contracts/script_host_ledger.rs",
        SCRIPT_LEDGER_TEST_MAX_LINES,
        "Runtime 13 ledger guard",
    );
    assert_file_line_budget(
        runtime_root,
        "src/script/vm/gameplay_host/tests/cases.rs",
        GAMEPLAY_TEST_MAX_LINES,
        "Runtime 13 gameplay host tests",
    );

    let script_framework_facade = include_str!("../../../core/framework/script.rs");
    for declaration in [
        "#[path = \"script/argument_views.rs\"]",
        "#[path = \"script/call_frame.rs\"]",
        "#[path = \"script/descriptors.rs\"]",
        "#[path = \"script/hot_path_metrics.rs\"]",
        "#[path = \"script/value_contracts.rs\"]",
    ] {
        assert!(
            script_framework_facade.contains(declaration),
            "Runtime 13 framework facade must retain the split declaration `{declaration}`"
        );
    }

    let builtin_host = include_str!("../../../script/vm/host/builtin_host_modules.rs");
    let gameplay_host = include_str!("../../../script/vm/gameplay_host.rs");
    assert_eq!(
        count_occurrences(builtin_host, "HostExportFunction::new("),
        21,
        "Runtime 13 builtin callback count should match script_binding_boundary"
    );
    assert_eq!(
        count_occurrences(gameplay_host, "HostExportFunction::new("),
        51,
        "Runtime 13 gameplay callback count should match script_binding_boundary"
    );
    assert_eq!(
        count_occurrences(builtin_host, "#[crate::zircon_host_function("),
        2,
        "Runtime 13 macro host-function count should match script_binding_boundary"
    );

    let guard_sources = [
        include_str!("../script_host_ledger.rs"),
        include_str!("../script_host_ledger/ledger.rs"),
        include_str!("../script_host_ledger/capability.rs"),
        include_str!("../script_host_ledger/ecs_facade.rs"),
        include_str!("../script_binding.rs"),
        include_str!("mirror_docs.rs"),
        include_str!("gameplay_host.rs"),
        include_str!("../../../script/vm/gameplay_host/tests/combat_lifecycle.rs"),
    ]
    .join("\n");
    for guard_anchor in RUNTIME_13_GUARD_ANCHORS {
        assert!(
            guard_sources.contains(guard_anchor),
            "Runtime 13 guard anchor `{guard_anchor}` should stay visible to script_binding_boundary"
        );
    }

    current_inventory::assert_current_inventory(SCRIPT_BINDING_MIRROR_DOC_ANCHORS);
}

#[test]
fn runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass() {
    let contract =
        include_str!("../../../../../docs/crates/zircon_runtime/script/vm/host/function_ledger.md");
    assert!(contract.contains("cargo test -p zircon_runtime --lib script --locked -- --nocapture"));
    assert!(contract.contains("code_static_pending_cargo"));
}

#[test]
fn runtime_13_script_binding_inventory_guard_requires_explicit_history() {
    let current = "Runtime13 current source contract inventory: fixed_host_function_count = 74, builtin_callback_count = 21, gameplay_callback_count = 51, host_capability_count = 14";
    let unlabelled = [
        "Current evidence reports fixed_host_function_count = 52",
        "Current owner hard-cut sync: builtin_callback_count = 11",
        "- Runtime 13 binding inventory reports gameplay_callback_count = 39",
        "  - python script_binding_boundary_audit (2026-06-14: host_capability_count = 11)",
    ];
    let source = format!("{current}\n{}", unlabelled.join("\n"));
    assert_eq!(unlabelled_host_inventories(&source), unlabelled);

    let labelled = [
        "Historical Runtime13 source snapshot: fixed_host_function_count = 52",
        "Historical Runtime13 source snapshot (2026-07-14): builtin_callback_count = 11",
        "- Historical Runtime13 source snapshot: gameplay_callback_count = 39",
        "  - python script_binding_boundary_audit (Historical Runtime13 source snapshot, 2026-06-14: host_capability_count = 11)",
    ];
    let source = format!("{current}\n{}", labelled.join("\n"));
    assert!(unlabelled_host_inventories(&source).is_empty());
}
