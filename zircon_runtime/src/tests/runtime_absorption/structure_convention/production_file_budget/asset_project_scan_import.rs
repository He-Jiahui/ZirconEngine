use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_asset_project_scan_import_sources_are_child_owner() {
    let parent = read_runtime_src("asset/project/manager/scan_and_import.rs");
    let sources = read_runtime_src("asset/project/manager/scan_and_import/sources.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );

    assert_contains_all(
        "asset project scan/import parent keeps import loop and delegates source collection",
        &parent,
        &[
            "mod sources;",
            "use self::sources::{",
            "AssetImportSource",
            "source_bytes_for_import",
            "source_mtime_unix_ms_for_import",
            "pub fn scan_and_import(&mut self) -> Result<Vec<ResourceRecord>, AssetImportError>",
            "fn restore_imported_artifact(",
            "fn finish_successful_import(",
            "fn finish_failed_import(",
        ],
    );
    for moved_owner in [
        "pub(super) struct AssetImportSource",
        "pub(super) fn collect_import_sources(",
        "fn collect_import_sources_for_root(",
        "fn collect_compound_sources_for_root(",
        "fn source_uri_for_asset_root_path(",
        "fn collect_zmeta_files(",
        "fn compound_root_for_meta_path(",
        "pub(super) fn source_bytes_for_import(",
        "pub(super) fn source_mtime_unix_ms_for_import(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/project/manager/scan_and_import.rs should delegate {moved_owner} to scan_and_import/sources.rs"
        );
    }
    assert_contains_all(
        "scan/import sources child owns source enumeration and compound source byte assembly",
        &sources,
        &[
            "pub(super) struct AssetImportSource",
            "pub(super) fn collect_import_sources(",
            "fn collect_import_sources_for_root(",
            "fn collect_compound_sources_for_root(",
            "fn source_uri_for_asset_root_path(",
            "fn collect_zmeta_files(",
            "fn compound_root_for_meta_path(",
            "pub(super) fn source_bytes_for_import(",
            "pub(super) fn source_mtime_unix_ms_for_import(",
        ],
    );

    for (path, source) in [
        ("asset/project/manager/scan_and_import.rs", parent.as_str()),
        (
            "asset/project/manager/scan_and_import/sources.rs",
            sources.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("importer doc", importer_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 asset project scan/import source collection owner split",
                "runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred",
                "asset/project/manager/scan_and_import.rs",
                "asset/project/manager/scan_and_import/sources.rs",
                "runtime_15_asset_project_scan_import_sources_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 asset project scan/import source collection owner split",
            "runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 asset project scan/import source collection owner split",
            "2026-06-24",
        ],
    );
}
