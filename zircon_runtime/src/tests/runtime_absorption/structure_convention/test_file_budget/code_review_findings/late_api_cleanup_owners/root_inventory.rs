use super::*;

#[test]
fn runtime_15_late_api_cleanup_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for forbidden in [
        "pub(super) struct LateApiCleanupSources",
        "pub(super) const STRUCTURE_GUARD_OWNER:",
        "pub(super) const REVIEW_GUARDS:",
        "pub(super) const FOLDER_BACKED_CHILDREN:",
        "pub(super) fn read_late_api_cleanup_sources",
        "pub(super) fn folder_backed_child_sources",
    ] {
        assert!(
            !parent.contains(forbidden),
            "late API cleanup parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        STRUCTURE_GUARD_OWNER,
        LATE_API_CLEANUP_ROOT_PATHS_CHILD,
        LATE_API_CLEANUP_ROOT_STATUSES_CHILD,
        LATE_API_CLEANUP_ROOT_CHILD_ROWS_CHILD,
        LATE_API_CLEANUP_ROOT_SOURCES_CHILD,
        LATE_API_CLEANUP_ROOT_INVENTORY_CHILD,
        "Cargo gate deferred",
    ];
}
