use super::*;

#[test]
fn runtime_15_asset_project_zmeta_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/project/zmeta.rs");
    let metadata_lifecycle = read_runtime_src("asset/tests/project/zmeta/metadata_lifecycle.rs");
    let package_roots = read_runtime_src("asset/tests/project/zmeta/package_roots.rs");
    let compound_shader = read_runtime_src("asset/tests/project/zmeta/compound_shader.rs");
    let shader_diagnostics_fixture =
        read_runtime_src("asset/tests/project/zmeta/shader_diagnostics_fixture.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "asset project zmeta parent test module mounts",
        &parent,
        &[
            "mod compound_shader;",
            "mod metadata_lifecycle;",
            "mod package_roots;",
            "mod shader_diagnostics_fixture;",
            "fn multi_asset_importer",
            "fn material_for_shader",
            "fn import_multi_asset_bundle_with_text",
        ],
    );

    for moved_test in [
        "fn project_manager_writes_zmeta_schema_and_ignores_old_meta_toml_sidecars",
        "fn project_manager_restore_refreshes_zmeta_entry_urls_after_source_rename",
        "fn project_manager_scans_package_asset_roots_as_package_uris",
        "fn project_manager_imports_compound_zshader_package_with_subassets",
        "fn documented_zmeta_shader_material_fixture_parses",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/project/zmeta.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/project/zmeta.rs should not keep executable tests in the parent module"
    );
    let migrated_test_count = [
        metadata_lifecycle.as_str(),
        package_roots.as_str(),
        compound_shader.as_str(),
        shader_diagnostics_fixture.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 15,
        "asset project zmeta child modules should preserve the current 15 tests"
    );

    assert_contains_all(
        "asset project zmeta metadata child owns schema/reference lifecycle contracts",
        &metadata_lifecycle,
        &[
            "use super::*;",
            "fn project_manager_writes_zmeta_schema_and_ignores_old_meta_toml_sidecars",
            "fn project_manager_watch_remove_commits_resource_and_asset_registries_together",
            "fn project_manager_watch_rename_preserves_guid_and_replaces_both_registry_paths",
            "fn project_manager_registry_commit_failure_keeps_both_live_registries_unchanged",
            "fn project_manager_restore_refreshes_zmeta_entry_urls_after_source_rename",
            "fn project_manager_preserves_zmeta_subasset_uuids_across_failed_reimport",
        ],
    );
    assert_contains_all(
        "asset project zmeta package child owns package URI contracts",
        &package_roots,
        &[
            "use super::*;",
            "fn project_manager_scans_package_asset_roots_as_package_uris",
            "package://com.zircon.navigation/nav/agent.json",
        ],
    );
    assert_contains_all(
        "asset project zmeta compound shader child owns shader package import contracts",
        &compound_shader,
        &[
            "use super::*;",
            "fn project_manager_imports_compound_zshader_package_with_subassets",
            "fn project_manager_derives_include_shader_import_path_from_project_and_package_path",
            "fn project_manager_reports_redundant_explicit_shader_import_path",
            "fn project_manager_reports_duplicate_shader_import_path_conflicts",
            "assert_eq!(shader.pipeline_layout, Default::default());",
            "material_for_shader(&shader_uri)",
        ],
    );
    assert_contains_all(
        "asset project zmeta diagnostics child owns zshader diagnostics and fixture contracts",
        &shader_diagnostics_fixture,
        &[
            "use super::*;",
            "fn project_manager_imports_zshader_with_wgsl_capture_diagnostics",
            "fn zshader_v2_options_replace_legacy_user_shader_definition_rows",
            "fn documented_zmeta_shader_material_fixture_parses",
        ],
    );

    for (path, source) in [
        ("asset/tests/project/zmeta.rs", parent.as_str()),
        (
            "asset/tests/project/zmeta/metadata_lifecycle.rs",
            metadata_lifecycle.as_str(),
        ),
        (
            "asset/tests/project/zmeta/package_roots.rs",
            package_roots.as_str(),
        ),
        (
            "asset/tests/project/zmeta/compound_shader.rs",
            compound_shader.as_str(),
        ),
        (
            "asset/tests/project/zmeta/shader_diagnostics_fixture.rs",
            shader_diagnostics_fixture.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_asset_project_manager_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/project/manager.rs");
    let artifact_cache_imports =
        read_runtime_src("asset/tests/project/manager/artifact_cache_imports.rs");
    let restore_failure_migration =
        read_runtime_src("asset/tests/project/manager/restore_failure_migration.rs");
    let subassets_errors = read_runtime_src("asset/tests/project/manager/subassets_errors.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "asset project manager parent test module mounts",
        &parent,
        &[
            "mod artifact_cache_imports;",
            "mod restore_failure_migration;",
            "mod subassets_errors;",
            "fn project_manager_with_first_wave_plugin_fixtures",
            "fn import_counted_data",
            "fn import_multi_asset_bundle",
        ],
    );

    for moved_test in [
        "fn project_manager_scans_assets_imports_artifact_cache_and_loads_artifacts",
        "fn project_manager_imports_physics_and_animation_assets_into_runtime_artifact_cache",
        "fn project_manager_restores_ready_artifacts_from_meta_after_restart",
        "fn project_manager_records_ui_schema_migration_in_meta",
        "fn project_manager_imports_labeled_subassets_as_separate_artifacts",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/project/manager.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/project/manager.rs should not keep executable tests in the parent module"
    );
    let migrated_test_count = [
        artifact_cache_imports.as_str(),
        restore_failure_migration.as_str(),
        subassets_errors.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 11,
        "asset project manager child modules should preserve the current 11 tests"
    );

    assert_contains_all(
        "asset project manager artifact-cache child owns first-wave import contracts",
        &artifact_cache_imports,
        &[
            "use super::*;",
            "fn project_manager_scans_assets_imports_artifact_cache_and_loads_artifacts",
            "fn project_manager_imports_physics_and_animation_assets_into_runtime_artifact_cache",
            "fn project_manager_imports_sound_assets_into_runtime_artifact_cache",
        ],
    );
    assert_contains_all(
        "asset project manager restore child owns failed import and migration contracts",
        &restore_failure_migration,
        &[
            "use super::*;",
            "fn project_manager_restores_ready_artifacts_from_meta_after_restart",
            "fn project_manager_records_failed_imports_and_continues_scanning",
            "fn project_manager_records_import_dependency_ids_and_missing_dependency_diagnostics",
        ],
    );
    assert_contains_all(
        "asset project manager subasset child owns label/error contracts",
        &subassets_errors,
        &[
            "use super::*;",
            "fn project_manager_imports_labeled_subassets_as_separate_artifacts",
            "fn project_manager_records_duplicate_imported_asset_label_as_failed_import",
            "fn project_manager_returns_structured_error_for_unknown_label_load",
        ],
    );

    for (path, source) in [
        ("asset/tests/project/manager.rs", parent.as_str()),
        (
            "asset/tests/project/manager/artifact_cache_imports.rs",
            artifact_cache_imports.as_str(),
        ),
        (
            "asset/tests/project/manager/restore_failure_migration.rs",
            restore_failure_migration.as_str(),
        ),
        (
            "asset/tests/project/manager/subassets_errors.rs",
            subassets_errors.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
