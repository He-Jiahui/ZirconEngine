use super::super::assert_contains_all;
use super::helpers::{assert_not_contains, read_repo, CORE_DOCS_WITH_SESSION};

const STATUS: &str =
    "runtime_15_module_convention_gate_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_module_convention_gate_guard_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 module convention gate guard folder-backed split";
const GUARD: &str = "runtime_15_module_convention_gate_guard_is_folder_backed";

const PARENT_PATH: &str =
    "zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs";
const ROW_PARENT_PATH: &str = "structure_convention/module_convention_gate.rs";
const ROW_CHILD_PATHS: &[&str] = &[
    "structure_convention/module_convention_gate/helpers.rs",
    "structure_convention/module_convention_gate/split_layout.rs",
];

#[test]
fn runtime_15_module_convention_gate_guard_is_folder_backed() {
    let parent = read_repo(PARENT_PATH);
    assert_contains_all(
        "module convention gate parent mounts children",
        &parent,
        &[
            "mod audit_status;",
            "mod debt_boundary;",
            "mod helpers;",
            "mod module_doc_frontmatter;",
            "mod output_contract;",
            "mod split_layout;",
        ],
    );
    assert_not_contains(
        "module convention gate parent",
        &parent,
        &[
            "fn read_repo",
            "runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit",
            "runtime_15_module_convention_gate_reports_non_render_debt_boundary",
            "frontmatter_section_entries",
        ],
    );

    assert_contains_all(
        "module doc frontmatter child",
        &read_repo(CHILD_PATHS[1]),
        &["runtime_15_module_convention_module_doc_frontmatter_has_unique_entries"],
    );
    assert_contains_all(
        "output contract child",
        &read_repo(CHILD_PATHS[2]),
        &["runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit"],
    );
    assert_contains_all(
        "debt boundary child",
        &read_repo(CHILD_PATHS[3]),
        &[
            "runtime_15_module_convention_gate_reports_non_render_debt_boundary",
            "runtime_15_render_scoped_migration_debt_handoff_is_status_locked",
            "runtime_15_hard_cutover_allowed_hyper_policy_does_not_report_risk",
        ],
    );
    assert_contains_all("audit status child", &read_repo(CHILD_PATHS[4]), &[]);

    for (path, max_lines) in [
        (PARENT_PATH, 20usize),
        (CHILD_PATHS[0], 100),
        (CHILD_PATHS[1], 120),
        (CHILD_PATHS[2], 120),
        (CHILD_PATHS[3], 220),
        (CHILD_PATHS[4], 250),
        (CHILD_PATHS[5], 160),
    ] {
        let line_count = read_repo(path).lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    for doc in CORE_DOCS_WITH_SESSION {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[SLICE, STATUS, GUARD, "module_convention_gate"],
        );
    }

    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    assert_contains_all(
        "frameworks plan records module convention gate split",
        &frameworks_plan,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}
