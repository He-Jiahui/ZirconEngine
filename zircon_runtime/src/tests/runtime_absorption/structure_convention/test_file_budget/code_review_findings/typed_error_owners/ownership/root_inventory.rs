use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_child_ownership_root_inventory_is_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_CHILD);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

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
        TYPED_ERROR_CHILD_OWNERSHIP_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_PATHS_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_STATUSES_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_SOURCES_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_CHILD,
        "Cargo gate deferred",
    ];
}
