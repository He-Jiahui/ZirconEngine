use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_status_docs_root_inventory_is_child_owned() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER);
    let status_rows = plugin_importer_dx_status_row_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, child_path, anchor) in PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"status_docs/{module_name}.rs\"]");
        assert_contains_all(
            "plugin-importer DX status-doc parent mounts root-inventory child",
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
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_SLICE,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_STATUS,
        PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_PATHS_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_STATUSES_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILD_ROWS_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_SOURCES_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("plugin-importer DX row data", status_rows.as_str()),
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
        "M3 review status map records plugin-importer DX status-doc root inventory split",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer DX status-doc root inventory split",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_DATE,
        ],
    );
}
