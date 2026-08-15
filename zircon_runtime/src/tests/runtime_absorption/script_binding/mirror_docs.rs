use std::path::Path;

use super::inventory::{
    EXPECTED_RUNTIME_13_SOURCE_FILES, EXPECTED_RUNTIME_13_TEST_FILES, GAMEPLAY_TEST_MAX_LINES,
    RUNTIME_13_GUARD_ANCHORS, SCRIPT_BINDING_MIRROR_DOC_ANCHORS, SCRIPT_LEDGER_TEST_MAX_LINES,
};
use super::support::{assert_file_line_budget, assert_files_exist, count_occurrences};

#[test]
fn runtime_13_script_binding_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_13_SOURCE_FILES.len(), 24);
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
        "src/tests/runtime_absorption/script_host_ledger.rs",
        SCRIPT_LEDGER_TEST_MAX_LINES,
        "Runtime 13 ledger guard",
    );
    assert_file_line_budget(
        runtime_root,
        "src/script/vm/gameplay_host/tests.rs",
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
        11,
        "Runtime 13 builtin callback count should match script_binding_boundary"
    );
    assert_eq!(
        count_occurrences(gameplay_host, "HostExportFunction::new("),
        40,
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

    let mirror_docs = [
        (
            "Runtime 13 function ledger",
            include_str!("../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md"),
        ),
        (
            "Runtime 13 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(doc_name, doc_source, SCRIPT_BINDING_MIRROR_DOC_ANCHORS);
    }
}

#[test]
fn runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass() {
    let plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );
    assert!(plan.contains("cargo test -p zircon_runtime --lib script --locked -- --nocapture"));
    assert!(plan.contains("code_static_pending_cargo"));
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should mirror Runtime 13 script-binding audit anchor `{anchor}`"
        );
    }
}
