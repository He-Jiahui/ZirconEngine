use super::*;

#[test]
fn runtime_15_asset_pipeline_manager_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/pipeline/manager.rs");
    let model_import = read_runtime_src("asset/tests/pipeline/manager/model_import.rs");
    let project_open = read_runtime_src("asset/tests/pipeline/manager/project_open.rs");
    let resource_records = read_runtime_src("asset/tests/pipeline/manager/resource_records.rs");
    let resource_revisions = read_runtime_src("asset/tests/pipeline/manager/resource_revisions.rs");
    let runtime_leases = read_runtime_src("asset/tests/pipeline/manager/runtime_leases.rs");
    let service_capabilities =
        read_runtime_src("asset/tests/pipeline/manager/service_capabilities.rs");
    let watcher = read_runtime_src("asset/tests/pipeline/manager/watcher.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_asset_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );

    assert_contains_all(
        "asset pipeline manager parent test module mounts",
        &parent,
        &[
            "mod model_import;",
            "mod project_open;",
            "mod resource_records;",
            "mod resource_revisions;",
            "mod runtime_leases;",
            "mod service_capabilities;",
            "mod watcher;",
            "fn project_asset_manager_with_first_wave_plugin_fixtures",
        ],
    );
    for moved_test in [
        "fn asset_manager_opens_project_reports_assets_and_publishes_changes",
        "fn asset_manager_imports_model_toml_with_virtual_geometry_payload",
        "fn asset_manager_watcher_reimports_modified_assets",
        "fn resource_server_reports_resource_records_for_project_assets",
        "fn asset_manager_service_reports_importer_capabilities_before_and_after_project_open",
        "fn resource_server_reimport_bumps_revision_and_publishes_updated_event",
        "fn importing_one_asset_does_not_bump_unrelated_resource_revisions",
        "fn watcher_ignores_meta_sidecar_updates_for_revision_tracking",
        "fn watcher_reimports_modified_asset_once_without_revision_loop",
        "fn asset_manager_acquire_release_unloads_and_rehydrates_runtime_resources",
        "fn sample_virtual_geometry_model_asset",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/pipeline/manager.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/pipeline/manager.rs should not keep executable tests in the parent module"
    );

    let migrated_child_sources = [
        project_open.as_str(),
        model_import.as_str(),
        watcher.as_str(),
        resource_records.as_str(),
        service_capabilities.as_str(),
        resource_revisions.as_str(),
        runtime_leases.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        12,
        "asset pipeline manager child modules should preserve the current 12 parent tests"
    );

    assert_contains_all(
        "asset pipeline manager project-open child owns project scan contracts",
        &project_open,
        &[
            "use super::*;",
            "fn asset_manager_opens_project_reports_assets_and_publishes_changes",
        ],
    );
    assert_contains_all(
        "asset pipeline manager model child owns virtual geometry model import",
        &model_import,
        &[
            "use super::*;",
            "fn asset_manager_imports_model_toml_with_virtual_geometry_payload",
            "fn sample_virtual_geometry_model_asset",
        ],
    );
    assert_contains_all(
        "asset pipeline manager watcher child owns source watch contracts",
        &watcher,
        &[
            "use super::*;",
            "fn asset_manager_watcher_reports_changes_from_the_second_manifest_root",
            "fn asset_manager_watcher_reimports_modified_assets",
            "fn watcher_ignores_meta_sidecar_updates_for_revision_tracking",
            "fn watcher_reimports_modified_asset_once_without_revision_loop",
        ],
    );
    assert_contains_all(
        "asset pipeline manager resource-record child owns ResourceManager records",
        &resource_records,
        &[
            "use super::*;",
            "fn resource_server_reports_resource_records_for_project_assets",
        ],
    );
    assert_contains_all(
        "asset pipeline manager service child owns importer capability service contracts",
        &service_capabilities,
        &[
            "use super::*;",
            "fn asset_manager_service_reports_importer_capabilities_before_and_after_project_open",
        ],
    );
    assert_contains_all(
        "asset pipeline manager revision child owns resource revision contracts",
        &resource_revisions,
        &[
            "use super::*;",
            "fn resource_server_reimport_bumps_revision_and_publishes_updated_event",
            "fn importing_one_asset_does_not_bump_unrelated_resource_revisions",
        ],
    );
    assert_contains_all(
        "asset pipeline manager runtime lease child owns runtime resource lifecycle",
        &runtime_leases,
        &[
            "use super::*;",
            "fn asset_manager_acquire_release_unloads_and_rehydrates_runtime_resources",
        ],
    );

    for (path, source) in [
        ("asset/tests/pipeline/manager.rs", parent.as_str()),
        (
            "asset/tests/pipeline/manager/model_import.rs",
            model_import.as_str(),
        ),
        (
            "asset/tests/pipeline/manager/project_open.rs",
            project_open.as_str(),
        ),
        (
            "asset/tests/pipeline/manager/resource_records.rs",
            resource_records.as_str(),
        ),
        (
            "asset/tests/pipeline/manager/resource_revisions.rs",
            resource_revisions.as_str(),
        ),
        (
            "asset/tests/pipeline/manager/runtime_leases.rs",
            runtime_leases.as_str(),
        ),
        (
            "asset/tests/pipeline/manager/service_capabilities.rs",
            service_capabilities.as_str(),
        ),
        ("asset/tests/pipeline/manager/watcher.rs", watcher.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render asset doc", render_asset_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset pipeline manager test folder split",
                "runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred",
                "asset/tests/pipeline/manager.rs",
                "asset/tests/pipeline/manager/model_import.rs",
                "asset/tests/pipeline/manager/watcher.rs",
                "runtime_15_asset_pipeline_manager_tests_are_folder_backed",
            ],
        );
    }
}
