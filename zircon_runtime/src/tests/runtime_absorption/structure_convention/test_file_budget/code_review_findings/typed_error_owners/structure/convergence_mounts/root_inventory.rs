use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_convergence_mounts_root_inventory_is_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD);
    let status_rows = super::super::typed_error_structure_assertion_status_row_source();
    let status_map = super::super::typed_error_structure_assertion_status_map_source();
    let date_map = super::super::typed_error_structure_assertion_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, child_path, anchor) in TYPED_ERROR_CONVERGENCE_MOUNT_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"convergence_mounts/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error convergence mounts parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < 120,
            "{child_path} should stay below its root inventory child budget"
        );
    }

    let status_anchors = [
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_SLICE,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_STATUS,
        TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_PATHS_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_STATUSES_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_SOURCES_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("typed-error structure row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records typed-error convergence mounts root inventory split",
        &status_map,
        &[
            TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_SLICE,
            TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error convergence mounts root inventory split",
        &date_map,
        &[
            TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_SLICE,
            TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_DATE,
        ],
    );
}
