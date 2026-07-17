use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_root_inventory_is_child_owned() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let status_rows = plugin_importer_dx_structure_status_row_source();
    let status_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_PLUGIN_IMPORTER_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_PLUGIN_IMPORTER_DATE_MAP)
    );
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

    for (module_name, child_path, anchor) in PLUGIN_IMPORTER_DX_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"plugin_importer_dx_owners/{module_name}.rs\"]");
        assert_contains_all(
            "plugin-importer DX structure guard parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < PLUGIN_IMPORTER_DX_ROOT_CHILD_LINE_BUDGET,
            "{child_path} should stay below its root inventory child budget"
        );
    }

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
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_SLICE,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_STATUS,
        PLUGIN_IMPORTER_DX_STRUCTURE_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_PATHS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_STATUSES_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_CHILD_ROWS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_SOURCES_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_GUARD,
        "target-server direct binary passed",
    ];
    for (label, source) in [
        ("plugin-importer DX row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records plugin-importer DX structure guard root inventory split",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_ROOT_INVENTORY_SLICE,
            PLUGIN_IMPORTER_DX_ROOT_INVENTORY_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer DX structure guard root inventory split",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_ROOT_INVENTORY_SLICE,
            PLUGIN_IMPORTER_DX_ROOT_INVENTORY_DATE,
        ],
    );
}
