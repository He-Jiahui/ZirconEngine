use super::*;

#[test]
fn runtime_15_code_review_findings_structure_guard_children_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for forbidden in [
        "pub(super) const STRUCTURE_GUARD_PARENT:",
        "pub(super) const STRUCTURE_GUARD_CHILD_OWNER:",
        "pub(super) const STRUCTURE_GUARD_CHILDREN:",
        "pub(super) fn structure_guard_child_sources",
        "pub(super) fn structure_guard_child_source_blob",
    ] {
        assert!(
            !parent.contains(forbidden),
            "structure guard children parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        STRUCTURE_GUARD_CHILD_OWNER,
        STRUCTURE_GUARD_ROOT_PATHS_CHILD,
        STRUCTURE_GUARD_ROOT_STATUSES_CHILD,
        STRUCTURE_GUARD_ROOT_CHILD_ROWS_CHILD,
        STRUCTURE_GUARD_ROOT_SOURCES_CHILD,
        STRUCTURE_GUARD_ROOT_INVENTORY_CHILD,
        "Cargo gate deferred",
    ];
}
