use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_asset_project_scan_import_sources_are_child_owner() {
    let parent = read_runtime_src("asset/project/manager/scan_and_import.rs");
    let sources = read_runtime_src("asset/project/manager/scan_and_import/sources.rs");
    let dependency_resolution =
        read_runtime_src("asset/project/manager/scan_and_import/dependency_resolution.rs");
    let metadata = read_runtime_src("asset/project/manager/scan_and_import/metadata.rs");
    let runtime_15_output_records = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");

    assert_contains_all(
        "asset project scan/import parent keeps import loop and delegates source collection",
        &parent,
        &[
            "mod sources;",
            "mod dependency_resolution;",
            "mod metadata;",
            "use self::dependency_resolution::{",
            "use self::metadata::{",
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
        "struct ResolvedDependencies",
        "pub(super) fn resolve_imported_dependencies(",
        "pub(super) fn dependencies_for_entry(",
        "pub(super) fn merge_handwritten_dependencies_into_meta(",
        "pub(super) fn clear_schema_migration_metadata(",
        "pub(super) fn validate_import_entries(",
        "pub(super) fn existing_entry_uuids_for_source(",
        "pub(super) fn existing_entry_tags_for_source(",
        "pub(super) fn failed_entries_for_source(",
        "pub(super) fn entry_uuid_for_import_entry(",
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
    assert_contains_all(
        "scan/import dependency child owns locator resolution and handwritten dependency merge",
        &dependency_resolution,
        &[
            "struct ResolvedDependencies",
            "pub(super) fn resolve_imported_dependencies(",
            "pub(super) fn dependencies_for_entry(",
            "pub(super) fn merge_handwritten_dependencies_into_meta(",
        ],
    );
    assert_contains_all(
        "scan/import metadata child owns import validation and stable entry identity projection",
        &metadata,
        &[
            "pub(super) fn clear_schema_migration_metadata(",
            "pub(super) fn apply_importer_metadata(",
            "pub(super) fn validate_import_entries(",
            "pub(super) fn existing_entry_uuids_for_source(",
            "pub(super) fn existing_entry_tags_for_source(",
            "pub(super) fn failed_entries_for_source(",
            "pub(super) fn remap_meta_entry_urls_to_source(",
            "pub(super) fn entry_uuid_for_import_entry(",
            "pub(super) fn importer_contract_matches(",
            "pub(super) fn config_hash_for_settings(",
        ],
    );

    for (path, source) in [
        ("asset/project/manager/scan_and_import.rs", parent.as_str()),
        (
            "asset/project/manager/scan_and_import/sources.rs",
            sources.as_str(),
        ),
        (
            "asset/project/manager/scan_and_import/dependency_resolution.rs",
            dependency_resolution.as_str(),
        ),
        (
            "asset/project/manager/scan_and_import/metadata.rs",
            metadata.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 output records own the completed scan/import split status",
        &runtime_15_output_records,
        &[
            "Runtime 15 M4 asset project scan/import source collection owner split",
            "runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred",
        ],
    );
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("importer doc", importer_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "asset/project/manager/scan_and_import.rs",
                "asset/project/manager/scan_and_import/sources.rs",
                "asset/project/manager/scan_and_import/dependency_resolution.rs",
                "asset/project/manager/scan_and_import/metadata.rs",
                "runtime_15_asset_project_scan_import_sources_are_child_owner",
            ],
        );
    }
}
