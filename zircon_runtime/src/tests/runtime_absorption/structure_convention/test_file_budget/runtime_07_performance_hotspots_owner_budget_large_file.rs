use super::*;

const STATUS: &str =
    "runtime_15_runtime_07_owner_budget_large_file_gate_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split";
const GUARD: &str = "runtime_15_runtime_07_owner_budget_large_file_gate_is_child_owner";

#[test]
fn runtime_15_runtime_07_owner_budget_large_file_gate_is_child_owner() {
    let parent = read_runtime_src("tests/runtime_absorption/performance_hotspots/owner_budget.rs");
    let large_file_child = read_runtime_src(
        "tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
    );
    let mirror_docs_child = read_runtime_src(
        "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
    );
    let virtual_geometry_child = read_runtime_src(
        "tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
    );

    assert_contains_all(
        "Runtime 07 owner-budget parent mounts child owners",
        &parent,
        &[
            "#[path = \"owner_budget/large_file_gate.rs\"]",
            "#[path = \"owner_budget/mirror_docs.rs\"]",
            "#[path = \"owner_budget/virtual_geometry_debug_snapshot.rs\"]",
            "mod large_file_gate;",
            "mod mirror_docs;",
            "mod virtual_geometry_debug_snapshot;",
            "fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
            "include_str!(\"owner_budget/large_file_gate.rs\")",
            "include_str!(\"owner_budget/mirror_docs.rs\")",
            "include_str!(\"owner_budget/virtual_geometry_debug_snapshot.rs\")",
        ],
    );
    assert!(
        !parent
            .contains("fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts"),
        "performance_hotspots/owner_budget.rs should delegate mirror-doc audit checks"
    );
    assert!(
        !parent.contains(
            "fn runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit"
        ),
        "performance_hotspots/owner_budget.rs should delegate large-file owner-budget gate checks"
    );

    assert_contains_all(
        "large-file owner-budget child owns Runtime 07 audit contract",
        &large_file_child,
        &[
            "fn runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
            "`hotspot_count = 0`",
            "large_file_ownership_gate",
            "classified-and-clear",
            "threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots",
        ],
    );
    assert_contains_all(
        "mirror-docs child remains sibling owner",
        &mirror_docs_child,
        &[
            "fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "EXPECTED_TEST_FILE_COUNT = 14",
        ],
    );
    assert_contains_all(
        "virtual-geometry child remains sibling owner",
        &virtual_geometry_child,
        &[
            "fn runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
            "virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
            large_file_child.as_str(),
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            mirror_docs_child.as_str(),
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
            virtual_geometry_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
                "tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record Runtime 07 owner-budget large-file child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-07-01"],
    );
}
