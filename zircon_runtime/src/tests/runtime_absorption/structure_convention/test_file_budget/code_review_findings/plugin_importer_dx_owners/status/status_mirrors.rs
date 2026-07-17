use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_status_docs_folder_backed_status_is_current() {
    let status_rows = plugin_importer_dx_status_row_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
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

    for (label, source) in [
        ("plugin-importer DX row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_SLICE,
                PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_STATUS,
                PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER,
                PLUGIN_IMPORTER_DX_STATUS_DOC_DELEGATION_OWNER,
                PLUGIN_IMPORTER_DX_STATUS_DOC_DOC_MIRRORS_OWNER,
                PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS_MAPS_OWNER,
                PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS_MIRRORS_OWNER,
                PLUGIN_IMPORTER_DX_STATUS_DOC_GUARD,
                PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_GUARD,
                PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records plugin-importer DX status-doc folder-backed split",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer DX status-doc folder-backed split",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_DATE,
        ],
    );

    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER);
    for (path, source) in [(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER, parent)]
        .into_iter()
        .chain(plugin_importer_dx_status_docs_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_STATUS_DOC_CHILD_LINE_BUDGET,
            "{path} should stay below the focused plugin-importer DX status-doc budget; got {line_count} lines"
        );
    }
}
