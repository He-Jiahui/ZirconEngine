use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_moved_guard_absence_root_inventory_is_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, child_path, anchor) in TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"moved_guard_absence/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error moved-guard absence parent mounts root-inventory child",
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
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_PATHS_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_STATUSES_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_SOURCES_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_INVENTORY_CHILD,
        "Cargo gate deferred",
    ];
}
