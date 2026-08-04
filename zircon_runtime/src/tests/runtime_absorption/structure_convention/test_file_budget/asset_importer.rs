use super::*;

#[test]
fn runtime_15_asset_importer_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/importer.rs");
    let structure = read_runtime_src("asset/tests/assets/importer/structure.rs");
    let typed_toml_ui = read_runtime_src("asset/tests/assets/importer/typed_toml_ui.rs");
    let builtin_data = read_runtime_src("asset/tests/assets/importer/builtin_data.rs");
    let registry_priority = read_runtime_src("asset/tests/assets/importer/registry_priority.rs");
    let registry_errors = read_runtime_src("asset/tests/assets/importer/registry_errors.rs");
    let shader_model = read_runtime_src("asset/tests/assets/importer/shader_model.rs");
    let physics_animation = read_runtime_src("asset/tests/assets/importer/physics_animation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "asset importer parent test module mounts",
        &parent,
        &[
            "mod structure;",
            "mod typed_toml_ui;",
            "mod builtin_data;",
            "mod registry_priority;",
            "mod registry_errors;",
            "mod shader_model;",
            "mod physics_animation;",
            "fn valid_wgsl",
            "fn test_data_outcome",
            "fn assert_cooked_virtual_geometry",
        ],
    );
    for moved_test in [
        "fn importer_subtree_uses_ingest_namespace_without_service_shell",
        "fn importer_registry_uses_full_suffix_before_plain_extension_fallback",
        "fn importer_default_decodes_builtin_png_texture_without_plugin_backend",
        "fn importer_registry_priority_overrides_duplicate_extension",
        "fn asset_import_error_preserves_registry_error_source",
        "fn importer_validates_wgsl_and_reports_errors",
        "fn importer_decodes_physics_material_and_animation_sequence_assets",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/importer.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/importer.rs should not keep executable tests in the parent module"
    );
    let child_sources = [
        structure.as_str(),
        typed_toml_ui.as_str(),
        builtin_data.as_str(),
        registry_priority.as_str(),
        registry_errors.as_str(),
        shader_model.as_str(),
        physics_animation.as_str(),
    ];
    assert_eq!(
        child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        21,
        "asset importer child modules should preserve the current 21 tests"
    );

    assert_contains_all(
        "asset importer structure child owns hard-cutover contracts",
        &structure,
        &[
            "use super::*;",
            "fn importer_subtree_uses_ingest_namespace_without_service_shell",
            "mod ingest;",
        ],
    );
    assert_contains_all(
        "asset importer typed-TOML child owns UI and typed suffix contracts",
        &typed_toml_ui,
        &[
            "use super::*;",
            "fn importer_registry_routes_zui_to_document_backend",
            "fn importer_default_rejects_ui_toml_without_source_fixture_backend",
            "fn importer_registry_rejects_unknown_typed_toml_instead_of_plain_data_fallback",
        ],
    );
    assert_contains_all(
        "asset importer builtin-data child owns default importer contracts",
        &builtin_data,
        &[
            "use super::*;",
            "fn importer_default_decodes_builtin_png_texture_without_plugin_backend",
            "fn importer_default_decodes_txt_as_text_data",
            "fn importer_capability_report_marks_diagnostic_only_backends",
        ],
    );
    assert_contains_all(
        "asset importer registry-priority child owns matcher ordering contracts",
        &registry_priority,
        &[
            "use super::*;",
            "fn importer_registry_priority_overrides_duplicate_extension",
            "fn importer_registry_prefers_available_extension_importer_over_higher_priority_diagnostic",
            "fn importer_registry_rejects_same_priority_duplicate_matcher",
        ],
    );
    assert_contains_all(
        "asset importer registry-error child owns typed error conversion",
        &registry_errors,
        &[
            "use super::*;",
            "fn asset_import_error_preserves_registry_error_source",
            "AssetImporterRegistryError::DuplicateMatcher",
        ],
    );
    assert_contains_all(
        "asset importer shader/model child owns shader and model import contracts",
        &shader_model,
        &[
            "use super::*;",
            "fn importer_validates_wgsl_and_reports_errors",
            "fn importer_emits_mesh_subassets_for_model_imports",
            "fn importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh",
        ],
    );
    assert_contains_all(
        "asset importer physics-animation child owns first-wave asset contracts",
        &physics_animation,
        &[
            "use super::*;",
            "fn importer_decodes_physics_material_and_animation_sequence_assets",
            "ImportedAsset::AnimationSequence",
        ],
    );

    for source in [parent.as_str()].into_iter().chain(child_sources) {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset importer parent and child test owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
