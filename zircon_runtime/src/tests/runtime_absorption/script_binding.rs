use std::fs;
use std::path::Path;

const SCRIPT_LEDGER_TEST_MAX_LINES: usize = 700;
const GAMEPLAY_TEST_MAX_LINES: usize = 1000;
const GAMEPLAY_HOST_OWNER_MAX_LINES: usize = 400;
const EXPECTED_RUNTIME_13_SOURCE_FILES: &[&str] = &[
    "src/script/vm/host/builtin_host_modules.rs",
    "src/script/vm/gameplay_host.rs",
    "src/script/vm/gameplay_host/combat.rs",
    "src/script/vm/gameplay_host/components.rs",
    "src/script/vm/gameplay_host/error.rs",
    "src/script/vm/gameplay_host/input.rs",
    "src/script/vm/gameplay_host/lifecycle.rs",
    "src/script/vm/gameplay_host/navigation.rs",
    "src/script/vm/gameplay_host/script_bindings.rs",
    "src/script/vm/gameplay_host/transform.rs",
    "src/script/vm/gameplay_host/values.rs",
    "src/script/vm/host/bridge_host_module.rs",
    "src/script/vm/host/host_export_registry.rs",
    "src/script/vm/host/script_call_table.rs",
    "src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs",
    "src/core/framework/script.rs",
    "src/script/vm/capability_set.rs",
    "src/script/vm/handles.rs",
    "src/script/vm/runtime_context.rs",
];
const EXPECTED_RUNTIME_13_TEST_FILES: &[&str] = &[
    "src/tests/runtime_absorption/script_host_ledger.rs",
    "src/tests/runtime_absorption/script_binding.rs",
    "src/script/vm/gameplay_host/tests.rs",
];
const GAMEPLAY_HOST_OWNER_FILES: &[&str] = &[
    "src/script/vm/gameplay_host.rs",
    "src/script/vm/gameplay_host/combat.rs",
    "src/script/vm/gameplay_host/components.rs",
    "src/script/vm/gameplay_host/error.rs",
    "src/script/vm/gameplay_host/input.rs",
    "src/script/vm/gameplay_host/lifecycle.rs",
    "src/script/vm/gameplay_host/navigation.rs",
    "src/script/vm/gameplay_host/script_bindings.rs",
    "src/script/vm/gameplay_host/transform.rs",
    "src/script/vm/gameplay_host/values.rs",
];

#[test]
fn runtime_13_script_binding_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_13_SOURCE_FILES.len(), 19);
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

    let builtin_host = include_str!("../../script/vm/host/builtin_host_modules.rs");
    let gameplay_host = include_str!("../../script/vm/gameplay_host.rs");
    assert_eq!(
        count_occurrences(builtin_host, "HostExportFunction::new("),
        11,
        "Runtime 13 builtin callback count should match script_binding_boundary"
    );
    assert_eq!(
        count_occurrences(gameplay_host, "HostExportFunction::new("),
        39,
        "Runtime 13 gameplay callback count should match script_binding_boundary"
    );
    assert_eq!(
        count_occurrences(builtin_host, "#[crate::zircon_host_function("),
        2,
        "Runtime 13 macro host-function count should match script_binding_boundary"
    );

    let script_host_guard = include_str!("script_host_ledger.rs");
    let script_binding_guard = include_str!("script_binding.rs");
    let gameplay_tests = include_str!("../../script/vm/gameplay_host/tests.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/late.rs");
    for guard_anchor in [
        "host_function_registry_matches_documented_ledger",
        "host_function_registry_ledger_guard_rejects_missing_entry",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
        "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
        "script_held_entity_handle_reports_invalid_after_despawn",
        "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
        "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
        "runtime_13_gameplay_host_owner_split_keeps_domain_files",
    ] {
        assert!(
            script_host_guard.contains(guard_anchor)
                || script_binding_guard.contains(guard_anchor)
                || gameplay_tests.contains(guard_anchor)
                || cargo_gate_guard.contains(guard_anchor),
            "Runtime 13 guard anchor `{guard_anchor}` should stay visible to script_binding_boundary"
        );
    }

    let mirror_docs = [
        (
            "Runtime 13 function ledger",
            include_str!("../../../../docs/zircon_runtime/script/vm/host/function_ledger.md"),
        ),
        (
            "Runtime 13 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "script_binding_boundary",
            "expected_source_file_count = 19",
            "expected_test_file_count = 3",
            "fixed_host_module_count = 6",
            "fixed_host_function_count = 52",
            "type_descriptor_count = 2",
            "builtin_callback_count = 11",
            "gameplay_callback_count = 39",
            "macro_host_function_count = 2",
            "host_capability_count = 11",
            "guard_anchor_count = 9",
            "native_ecs_abi_references = []",
            "oversized_test_files = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 13 script-binding audit anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_13_gameplay_host_owner_split_keeps_domain_files() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_files_exist(
        runtime_root,
        GAMEPLAY_HOST_OWNER_FILES,
        "Runtime 13 gameplay host owner",
    );
    for file in GAMEPLAY_HOST_OWNER_FILES {
        assert_file_line_budget(
            runtime_root,
            file,
            GAMEPLAY_HOST_OWNER_MAX_LINES,
            "Runtime 13 gameplay host owner",
        );
    }

    let gameplay_host = include_str!("../../script/vm/gameplay_host.rs");
    for module_anchor in [
        "mod combat;",
        "mod components;",
        "mod error;",
        "mod input;",
        "mod lifecycle;",
        "mod navigation;",
        "mod script_bindings;",
        "mod transform;",
        "mod values;",
    ] {
        assert!(
            gameplay_host.contains(module_anchor),
            "gameplay_host.rs should keep domain owner module `{module_anchor}`"
        );
    }
    for registration_anchor in [
        "HostExportFunction::new(\"key_pressed\"",
        "HostExportFunction::new(\"position_json\"",
        "HostExportFunction::new(\"component_json\"",
        "HostExportFunction::new(\"damage_entity\"",
        "HostExportFunction::new(\"spawn_empty\"",
        "HostExportFunction::new(\"nav_next_point_json\"",
    ] {
        assert!(
            gameplay_host.contains(registration_anchor),
            "gameplay_host.rs should keep callback registration anchor `{registration_anchor}`"
        );
    }

    let values = include_str!("../../script/vm/gameplay_host/values.rs");
    for value_anchor in [
        "expect_vec3_json",
        "resource_handle_from_script_ref",
        "script_core_error",
    ] {
        assert!(
            values.contains(value_anchor),
            "gameplay_host/values.rs should own shared value helper `{value_anchor}`"
        );
    }
    let transform = include_str!("../../script/vm/gameplay_host/transform.rs");
    assert!(
        transform.contains("navigation_next_point"),
        "gameplay_host/transform.rs should keep navigation-aware movement dependency explicit"
    );
    let navigation = include_str!("../../script/vm/gameplay_host/navigation.rs");
    assert!(
        navigation.contains("NavMeshAgentDescriptor"),
        "gameplay_host/navigation.rs should own nav-agent mutation"
    );
}

fn assert_files_exist(runtime_root: &Path, files: &[&str], label: &str) {
    for file in files {
        let path = runtime_root.join(file);
        assert!(
            path.exists(),
            "{label} file `{}` is missing; update script_binding_boundary before changing Runtime 13 ownership",
            path.display()
        );
    }
}

fn assert_file_line_budget(runtime_root: &Path, file: &str, max_lines: usize, label: &str) {
    let path = runtime_root.join(file);
    let line_count = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count();
    assert!(
        line_count <= max_lines,
        "{label} `{file}` has {line_count} lines, exceeding the {max_lines}-line owner budget"
    );
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}
