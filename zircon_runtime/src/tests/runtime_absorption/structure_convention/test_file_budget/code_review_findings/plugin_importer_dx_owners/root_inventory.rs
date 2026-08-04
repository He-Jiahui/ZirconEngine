use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_root_inventory_is_child_owned() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for forbidden in [
        "pub(super) const STRUCTURE_GUARD_PARENT:",
        "pub(super) const FOLDER_BACKED_CHILDREN:",
        "pub(super) fn folder_backed_child_sources",
        "pub(super) fn folder_backed_child_source_blob",
    ] {
        assert!(
            !parent.contains(forbidden),
            "plugin-importer DX structure guard parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        PLUGIN_IMPORTER_DX_STRUCTURE_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_PATHS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_STATUSES_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_CHILD_ROWS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_SOURCES_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_CHILD,
        "target-server direct binary passed",
    ];
}
