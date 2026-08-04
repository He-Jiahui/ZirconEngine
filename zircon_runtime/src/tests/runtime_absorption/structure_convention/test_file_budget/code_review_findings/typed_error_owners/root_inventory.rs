use super::*;

#[test]
fn runtime_15_typed_error_structure_guard_root_inventory_is_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for forbidden in [
        "pub(super) const STRUCTURE_GUARD_PARENT:",
        "pub(super) const FOLDER_BACKED_CHILDREN:",
        "pub(super) fn folder_backed_child_sources",
        "pub(super) fn folder_backed_child_source_blob",
    ] {
        assert!(
            !parent.contains(forbidden),
            "typed-error structure guard parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        TYPED_ERROR_STRUCTURE_CHILD,
        TYPED_ERROR_ROOT_PATHS_CHILD,
        TYPED_ERROR_ROOT_STATUSES_CHILD,
        TYPED_ERROR_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_ROOT_SOURCES_CHILD,
        TYPED_ERROR_ROOT_INVENTORY_CHILD,
        "Cargo gate deferred",
    ];
}
