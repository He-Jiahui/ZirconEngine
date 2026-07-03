use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_asset_artifact_cache_ui_documents_are_child_owner() {
    let parent = read_runtime_src("asset/artifact/cache_payload.rs");
    let ui = read_runtime_src("asset/artifact/cache_payload/ui.rs");
    let material_shader = read_runtime_src("asset/artifact/cache_payload/material_shader.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let artifact_doc = read_repo("docs/zircon_runtime/asset/artifact.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "artifact cache parent keeps variant dispatch and imports the UI cache owner",
        &parent,
        &[
            "mod ui;",
            "mod material_shader;",
            "use ui::{ArtifactCacheUiAssetDocument, ArtifactCacheUiV2AssetDocument};",
            "use material_shader::{ArtifactCacheMaterialAsset, ArtifactCacheShaderAsset};",
            "UiLayout(ArtifactCacheUiAssetDocument)",
            "UiV2View(ArtifactCacheUiV2AssetDocument)",
            "Material(ArtifactCacheMaterialAsset)",
            "Shader(ArtifactCacheShaderAsset)",
            "ArtifactCacheUiAssetDocument::from_document(&asset.document)?",
            "ArtifactCacheUiV2AssetDocument::from_document(&asset.document)?",
            "ArtifactCacheMaterialAsset::from(asset)",
            "ArtifactCacheShaderAsset::from(asset)",
            "ImportedAsset::UiLayout(asset.into_layout_asset()?)",
            "ImportedAsset::UiV2Component(asset.into_component_asset()?)",
            "ImportedAsset::Material(asset.into_asset()?)",
            "ImportedAsset::Shader(asset.into_asset()?)",
        ],
    );
    for moved_owner in [
        "pub(super) struct ArtifactCacheUiAssetDocument",
        "impl ArtifactCacheUiAssetDocument",
        "pub(super) struct ArtifactCacheUiV2AssetDocument",
        "impl ArtifactCacheUiV2AssetDocument",
        "UiLayoutAsset::from_toml_str",
        "UiV2ViewAsset::from_toml_str",
        "pub(super) struct ArtifactCacheMaterialAsset",
        "pub(super) struct ArtifactCacheShaderAsset",
        "RenderShaderPipelineLayoutDescriptor",
        "ArtifactCacheShaderRenderStateDescriptor",
        "ArtifactCacheRenderShaderDefinitionValue",
        "cache_table_like_to_toml",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/artifact/cache_payload.rs should delegate {moved_owner} to cache_payload child owners"
        );
    }
    assert_contains_all(
        "UI cache child owns v1/v2 document TOML normalization and typed restore paths",
        &ui,
        &[
            "pub(super) struct ArtifactCacheUiAssetDocument",
            "pub(super) fn from_document(",
            "UiLayoutAsset::from_toml_str",
            "UiWidgetAsset::from_toml_str",
            "UiStyleAsset::from_toml_str",
            "pub(super) struct ArtifactCacheUiV2AssetDocument",
            "UiV2ViewAsset::from_toml_str",
            "UiV2ComponentAsset::from_toml_str",
            "UiV2StyleAsset::from_toml_str",
        ],
    );
    assert_contains_all(
        "material/shader cache child owns material and shader wire conversions",
        &material_shader,
        &[
            "pub(super) struct ArtifactCacheMaterialAsset",
            "pub(super) struct ArtifactCacheShaderAsset",
            "pub(super) fn into_asset(self) -> Result<MaterialAsset, AssetImportError>",
            "pub(super) fn into_asset(self) -> Result<ShaderAsset, AssetImportError>",
            "RenderShaderPipelineLayoutDescriptor",
            "ArtifactCacheShaderRenderStateDescriptor",
            "ArtifactCacheRenderShaderDefinitionValue",
            "cache_table_like_to_toml",
            "cache_table_to_toml",
        ],
    );

    for (path, source) in [
        ("asset/artifact/cache_payload.rs", parent.as_str()),
        ("asset/artifact/cache_payload/ui.rs", ui.as_str()),
        (
            "asset/artifact/cache_payload/material_shader.rs",
            material_shader.as_str(),
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
        ("artifact cache doc", artifact_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 asset artifact cache material/shader owner split",
                "runtime_15_asset_artifact_cache_material_shader_owner_split_static_passed_cargo_deferred",
                "asset/artifact/cache_payload.rs",
                "asset/artifact/cache_payload/material_shader.rs",
                "runtime_15_asset_artifact_cache_ui_documents_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 asset artifact cache material/shader owner split",
            "runtime_15_asset_artifact_cache_material_shader_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 asset artifact cache material/shader owner split",
            "2026-07-03",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("artifact cache doc", artifact_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 asset artifact cache UI document owner split",
                "runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred",
                "asset/artifact/cache_payload.rs",
                "asset/artifact/cache_payload/ui.rs",
                "runtime_15_asset_artifact_cache_ui_documents_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 asset artifact cache UI document owner split",
            "runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 asset artifact cache UI document owner split",
            "2026-06-24",
        ],
    );
}
