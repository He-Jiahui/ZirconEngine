use super::*;

#[test]
fn runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/native_plugin_loader.rs");
    let real_fixture =
        read_runtime_src("tests/plugin_extensions/native_plugin_loader/real_fixture.rs");
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
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let asset_importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");

    assert_contains_all(
        "native plugin loader parent mounts real fixture child owner",
        &parent,
        &[
            "#[path = \"native_plugin_loader/real_fixture.rs\"]",
            "mod real_fixture;",
        ],
    );

    for moved_test in [
        "fn native_loader_loads_real_fixture_from_export_load_manifest_payload",
        "fn native_loader_calls_real_fixture_descriptor_and_entries",
        "fn native_loader_rejects_unknown_abi_version_with_explicit_report",
        "fn native_loader_fixture_can_import_data_asset_through_native_importer_handler",
    ] {
        assert!(
            !parent.contains(moved_test),
            "native plugin loader parent should mount the real fixture child owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "real fixture child owns native dynamic fixture and importer contracts",
        &real_fixture,
        &[
            "use super::*;",
            "fn native_loader_loads_real_fixture_from_export_load_manifest_payload",
            "fn native_loader_calls_real_fixture_descriptor_and_entries",
            "fn native_loader_rejects_unknown_abi_version_with_explicit_report",
            "fn native_loader_fixture_can_import_data_asset_through_native_importer_handler",
            "NativeAssetImporterHandler::new",
            "ImportedAsset::Data",
            "ZIRCON_NATIVE_PLUGIN_STATUS_DENIED",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count() + real_fixture.matches("#[test]").count(),
        11,
        "native plugin loader parent plus real fixture child should preserve the original 11 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/native_plugin_loader.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/native_plugin_loader/real_fixture.rs",
            real_fixture.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
