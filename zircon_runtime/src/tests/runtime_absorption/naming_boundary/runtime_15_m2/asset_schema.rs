use std::path::Path;

use super::super::support::{assert_contains_all, read_repo_text, read_text};

#[path = "asset_schema/material_asset_schema_v1.rs"]
mod material_asset_schema_v1;

#[test]
fn runtime_15_font_ui_asset_schema_names_use_current_policy_terms() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let font_asset = read_text(
        &manifest_root.join("src/asset/assets/font.rs"),
        "font asset owner should be readable",
    );
    let ui_document_loader = read_text(
        &manifest_root.join("src/asset/assets/ui/document_loader.rs"),
        "UI document codec owner should be readable",
    );
    let ui_document_importer = read_repo_text(
        manifest_root,
        "zircon_plugins/ui_document_importer/runtime/src/lib.rs",
    );
    let ui_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/ui.md");
    let font_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/font.md");
    let frameworks_05_acceptance = read_repo_text(
        manifest_root,
        "tests/acceptance/frameworks-05-asset-ui-loader-hard-cutover.md",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );

    assert_contains_all(
        "font asset schema-v1 render mode helper",
        &font_asset,
        &[
            "schema_v1_render_mode: Option<UiTextRenderMode>",
            "let mode = schema_v1_render_mode.or(self.default_mode);",
        ],
    );
    assert!(
        !font_asset.contains(concat!("legacy", "_render_mode")),
        "font asset owner should not keep retired generic render-mode parameter name"
    );

    assert_contains_all(
        "ZUI document codec owner",
        &ui_document_loader,
        &[
            "pub(super) fn load_zui_document(",
            "UiV2AssetKind::Component => validate_zui_component_profile(document)",
        ],
    );
    for retired in ["UiV2DocumentImportProfile", "LegacyToml"] {
        assert!(
            !ui_document_loader.contains(retired),
            "ZUI document import owner should not keep retired profile branch {retired}"
        );
    }
    assert_contains_all(
        "ZUI plugin importer owner",
        &ui_document_importer,
        &[
            "UiZuiAssetLoader::load_zui_str",
            "ImportedAsset::UiV2Component",
        ],
    );
    assert!(
        !ui_document_importer.contains("UiV2DocumentImportProfile::Zui"),
        "ZUI importer should call the current document conversion owner directly"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI assets doc", ui_assets_doc.as_str()),
        ("font assets doc", font_assets_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 font/UI asset schema naming hard cutover",
                "runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/assets/font.rs",
                "schema_v1_render_mode",
                "runtime_15_font_ui_asset_schema_names_use_current_policy_terms",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 05 current ZUI importer ownership",
        &frameworks_05_acceptance,
        &[
            "asset 内 built-in `.zui` importer、旧 importer owner 和转换 owner 已删除",
            "zircon_plugins/ui_document_importer/runtime/src/plugin.rs",
        ],
    );
}

#[test]
fn runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer_font_asset = read_text(
        &manifest_root.join("src/graphics/scene/scene_renderer/ui/font_asset.rs"),
        "renderer font asset owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let font_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/font.md");

    assert_contains_all(
        "renderer font asset schema-v1 priority fixture",
        &renderer_font_asset,
        &["schema_v1_render_mode_takes_priority_over_strategy_default_mode"],
    );
    assert!(
        !renderer_font_asset.contains(concat!("legacy", "_render_mode_takes_priority")),
        "renderer font asset fixture should not keep retired generic render-mode test name"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("font assets doc", font_assets_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 font render-mode priority fixture naming hard cutover",
                "runtime_15_font_render_mode_priority_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/ui/font_asset.rs",
                "schema_v1_render_mode_takes_priority_over_strategy_default_mode",
                "runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name",
                "module_convention_gate classified-and-clear",
            ],
        );
    }
}
