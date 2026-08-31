use super::*;

#[test]
fn runtime_15_shader_prewarm_manifest_tests_are_folder_backed() {
    let parent = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let errors = read_runtime_src("bin/zircon_shader_prewarm/error.rs");
    let material_sources =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/material_sources.rs");
    let pass_types = read_runtime_src("bin/zircon_shader_prewarm/manifest/pass_types.rs");
    let paths = read_runtime_src("bin/zircon_shader_prewarm/manifest/paths.rs");
    let revision = read_runtime_src("bin/zircon_shader_prewarm/manifest/revision.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let asset_scan_error_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/asset_scan_errors.rs");
    let geometry_source_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/geometry_sources.rs");
    let io_tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/io.rs");
    let raw_revision_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/raw_revision.rs");
    let registry_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");

    assert_contains_all(
        "shader prewarm manifest parent mounts test child",
        &parent,
        &[
            "#[cfg(test)]\nmod tests;",
            "mod pass_types;",
            "mod paths;",
            "mod revision;",
            "mod tests;",
            "pub fn read_manifest",
            "pub fn asset_root_manifest_for_quality_tiers",
            "pub fn asset_root_manifest_for_quality_tiers_and_geometry_sources",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest path child owns source path helpers",
        &paths,
        &[
            "fn content_hash",
            "fn primary_zshader_path",
            "fn wgsl_files_for_document",
            "ShaderSourceOutsidePackageDir",
        ],
    );
    assert_contains_all(
        "raw WGSL prewarm remains a generic module",
        &parent,
        &[
            "fn shader_source_from_wgsl",
            "ShaderAssetKind::Module",
            "if !source.kind.participates_in_material_variants()",
        ],
    );
    assert!(
        !parent.contains("asset_scan_full_material_passes(),\n        ShaderAssetKind::Surface"),
        "raw WGSL prewarm must not regain implicit material passes or Surface identity"
    );
    assert_contains_all(
        "material prewarm rejects non-material shader kinds",
        &material_sources,
        &[
            "fn prewarm_manifest_for_material_source",
            "if !shader_source.kind.participates_in_material_variants()",
            "ShaderPrewarmAssetScanError::MaterialShaderKindMismatch",
        ],
    );
    assert_contains_all(
        "shader prewarm owns a typed material kind mismatch",
        &errors,
        &[
            "MaterialShaderKindMismatch",
            "material prewarm requires kind surface",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "bin/zircon_shader_prewarm/manifest.rs should mount child tests instead of defining executable tests"
    );
    assert!(
        !parent.contains("fn shader_prewarm_asset_root_manifest_reads_compound_zshader_package"),
        "bin/zircon_shader_prewarm/manifest.rs should mount the asset-root manifest test owner"
    );
    assert_contains_all(
        "shader prewarm manifest test child owns asset-root manifest contract",
        &(tests.clone() + &raw_revision_tests + &geometry_source_tests),
        &[
            "mod io;",
            "mod asset_scan_errors;",
            "mod geometry_sources;",
            "mod resource_registry;",
            "asset_root_manifest(&root)",
            "fn shader_prewarm_asset_root_manifest_reads_compound_zshader_package",
            "fn shader_prewarm_asset_root_manifest_uses_sparse_material_option_keys",
            "fn shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source",
            "fn shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids",
            "fn shader_prewarm_asset_root_manifest_expands_requested_geometry_sources",
            "fn shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision",
            "fn shader_prewarm_asset_root_manifest_does_not_promote_raw_wgsl_module",
            "material_revision",
            "SHADING_MODEL_ID_BLINN_PHONG",
            "SHADING_MODEL_ID_UNLIT",
            "SHADING_MODEL_PLUGIN_ID_START",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
        ],
    );
    assert_eq!(
        tests.matches("#[test]").count()
            + raw_revision_tests.matches("#[test]").count()
            + geometry_source_tests.matches("#[test]").count(),
        10,
        "shader prewarm manifest child should own builtin-fallback, asset-root, sparse-option, builtin-template, custom-shading, revision, and geometry-source tests"
    );
    assert_contains_all(
        "shader prewarm manifest IO child owns file read and merge schema errors",
        &io_tests,
        &[
            "fn shader_prewarm_read_manifest_reports_typed_read_error",
            "fn shader_prewarm_read_manifest_reports_typed_parse_error",
            "fn shader_prewarm_merge_manifest_reports_typed_schema_error",
            "ShaderPrewarmManifestError::Read",
            "ShaderPrewarmManifestError::Parse",
            "ShaderPrewarmManifestError::UnsupportedSchema",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest asset-scan error child owns typed scan errors",
        &asset_scan_error_tests,
        &[
            "fn shader_prewarm_asset_root_scan_reports_typed_read_root_error",
            "fn shader_prewarm_asset_root_scan_reports_typed_zshader_parse_error",
            "fn shader_prewarm_asset_root_scan_reports_typed_empty_wgsl_error",
            "fn shader_prewarm_asset_root_scan_reports_typed_zmaterial_parse_error",
            "fn shader_prewarm_asset_root_scan_rejects_material_bound_to_raw_wgsl_module",
            "ShaderPrewarmAssetScanError::ReadAssetRoot",
            "ShaderPrewarmAssetScanError::ParseZShader",
            "ShaderPrewarmAssetScanError::EmptyShaderSource",
            "ShaderPrewarmAssetScanError::ParseZMaterial",
            "ShaderPrewarmAssetScanError::MaterialShaderKindMismatch",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest resource-registry child owns registry revision tests",
        &registry_tests,
        &[
            "fn shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay",
            "fn shader_prewarm_asset_root_exports_raw_module_records_without_material_variants",
            "fn shader_prewarm_project_and_plugin_asset_roots_use_exported_registry_revisions",
            "fn shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
            "shader_resource_records_from_asset_root",
            "shader_resource_records_from_manager",
        ],
    );
    assert_eq!(
        registry_tests.matches("#[test]").count(),
        4,
        "shader prewarm manifest resource-registry child should own the four registry tests"
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/manifest.rs", parent.as_str()),
        ("bin/zircon_shader_prewarm/error.rs", errors.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/material_sources.rs",
            material_sources.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/pass_types.rs",
            pass_types.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/paths.rs",
            paths.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/revision.rs",
            revision.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/asset_scan_errors.rs",
            asset_scan_error_tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/geometry_sources.rs",
            geometry_source_tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/io.rs",
            io_tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/raw_revision.rs",
            raw_revision_tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs",
            registry_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
