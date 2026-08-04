use super::*;

const STATUS: &str =
    "runtime_15_runtime_07_owner_budget_mirror_docs_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split";
const GUARD: &str = "runtime_15_runtime_07_owner_budget_mirror_docs_is_child_owner";

#[test]
fn runtime_15_runtime_07_owner_budget_mirror_docs_is_child_owner() {
    let parent = read_runtime_src("tests/runtime_absorption/performance_hotspots/owner_budget.rs");
    let mirror_docs_child = read_runtime_src(
        "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
    );
    let mirror_docs_source = [
        mirror_docs_child.clone(),
        read_runtime_src(
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/audit_wiring.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/doc_mirrors.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/performance_guard.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/source_inventory.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/split_layout.rs",
        ),
    ]
    .join("\n");
    let sources_load = read_runtime_src(
        "tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
    );
    let source_inventory = read_repo(
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py",
    );

    assert_contains_all(
        "Runtime 07 owner-budget parent mounts mirror-doc child owner",
        &parent,
        &[
            "#[path = \"owner_budget/large_file_gate.rs\"]",
            "#[path = \"owner_budget/mirror_docs.rs\"]",
            "#[path = \"owner_budget/virtual_geometry_debug_snapshot.rs\"]",
            "mod large_file_gate;",
            "mod mirror_docs;",
            "mod virtual_geometry_debug_snapshot;",
            "fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
        ],
    );
    assert!(
        !parent
            .contains("fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts"),
        "performance_hotspots/owner_budget.rs should delegate mirror-doc audit checks"
    );

    assert_contains_all(
        "mirror-docs child owns Runtime 07 audit mirror contract",
        &mirror_docs_source,
        &[
            "fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "performance_hotpath_boundary",
            "EXPECTED_TEST_FILE_COUNT = 91",
            "expected_test_file_count = 91",
            "large_file_m1_gate_status = classified-and-clear",
            "owner_budget/large_file_gate.rs",
            "owner_budget/mirror_docs.rs",
            "owner_budget/virtual_geometry_debug_snapshot.rs",
            "mirror_docs_guard_present = true",
            "risks = []",
        ],
    );
    assert_contains_all(
        "Runtime 07 performance source inventory tracks owner-budget child owners",
        &source_inventory,
        &[
            "EXPECTED_TEST_FILE_COUNT = 91",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
        ],
    );
    assert_contains_all(
        "owner-budget source loader owns mirror-doc child source include",
        &sources_load,
        &[
            "owner_budget_large_file_gate: include_str!(\"../large_file_gate.rs\")",
            "owner_budget_mirror_docs: include_str!(\"../mirror_docs.rs\")",
            "owner_budget_virtual_geometry_debug_snapshot: include_str!(",
            "\"../virtual_geometry_debug_snapshot.rs\"",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            mirror_docs_child.as_str(),
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
            sources_load.as_str(),
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
}
