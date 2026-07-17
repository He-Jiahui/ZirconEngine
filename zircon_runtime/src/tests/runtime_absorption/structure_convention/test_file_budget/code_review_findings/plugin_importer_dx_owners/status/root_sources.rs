use super::*;

pub(super) struct PluginImporterDxStatusDocSources {
    pub(super) runtime_15_plan: String,
    pub(super) runtime_index: String,
    pub(super) review_findings: String,
    pub(super) structure_convention: String,
    pub(super) module_doc: String,
    pub(super) status_rows: String,
    pub(super) status_maps: String,
}

pub(super) fn plugin_importer_dx_status_doc_sources() -> PluginImporterDxStatusDocSources {
    PluginImporterDxStatusDocSources {
        runtime_15_plan: read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        ),
        runtime_index: read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
        ),
        review_findings: read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
        ),
        structure_convention: read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
        ),
        module_doc: read_repo("docs/zircon_runtime/structure/module-convention.md"),
        status_rows: plugin_importer_dx_status_row_source(),
        status_maps: plugin_importer_dx_status_maps_source(),
    }
}

pub(super) fn plugin_importer_dx_status_row_source() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_ROW_PATH),
        read_runtime_src(REVIEW_GUARD_REVIEW_ROW_PATH),
        read_runtime_src(REVIEW_GUARD_SOURCE_INVENTORY_ROW_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_DOC_ROW_PATH),
        read_runtime_src(REVIEW_GUARD_STRUCTURE_ASSERTION_ROW_PATH),
    )
}

pub(super) fn plugin_importer_dx_status_maps_source() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
    )
}

pub(super) fn plugin_importer_dx_status_docs_child_sources() -> Vec<(&'static str, String)> {
    PLUGIN_IMPORTER_DX_STATUS_DOC_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_status_docs_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in plugin_importer_dx_status_docs_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
