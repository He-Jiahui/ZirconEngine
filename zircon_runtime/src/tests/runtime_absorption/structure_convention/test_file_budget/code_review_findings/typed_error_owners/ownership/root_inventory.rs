use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_child_ownership_root_inventory_is_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_CHILD);
    let status_rows = typed_error_child_ownership_status_row_source();
    let status_map = typed_error_child_ownership_status_map_source();
    let date_map = typed_error_child_ownership_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, child_path, anchor) in TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"ownership/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error child-ownership parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILD_LINE_BUDGET,
            "{child_path} should stay below its root inventory child budget"
        );
    }

    for forbidden in [
        "pub(super) struct TypedErrorChildOwnershipSources",
        "pub(super) const TYPED_ERROR_CHILD_OWNERSHIP_CHILD:",
        "const MOVED_GUARD_ABSENCE_PRESERVED_GUARDS_CHILD:",
        "pub(super) const TYPED_ERROR_CHILD_OWNERSHIP_CHILDREN:",
        "pub(super) fn typed_error_child_ownership_sources",
        "pub(super) fn typed_error_child_ownership_child_sources",
    ] {
        assert!(
            !parent.contains(forbidden),
            "typed-error child-ownership parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_SLICE,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_STATUS,
        TYPED_ERROR_CHILD_OWNERSHIP_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_PATHS_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_STATUSES_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_SOURCES_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("typed-error row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records typed-error child-ownership root inventory split",
        &status_map,
        &[
            TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_SLICE,
            TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error child-ownership root inventory split",
        &date_map,
        &[
            TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_SLICE,
            TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_DATE,
        ],
    );
}
