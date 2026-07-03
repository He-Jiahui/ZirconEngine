use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[path = "asset_schema/material_asset_schema_v1.rs"]
mod material_asset_schema_v1;

#[test]
fn runtime_15_font_ui_asset_schema_names_use_current_policy_terms() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let font_asset = read_text(
        &manifest_root.join("src/asset/assets/font.rs"),
        "font asset owner should be readable",
    );
    let ui_v2_document_import = read_text(
        &manifest_root.join("src/asset/importer/ingest/ui_v2_document_import.rs"),
        "ui v2 document import owner should be readable",
    );
    let import_ui_zui_asset = read_text(
        &manifest_root.join("src/asset/importer/ingest/import_ui_zui_asset.rs"),
        "ZUI importer should be readable",
    );
    let ui_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/ui.md");
    let font_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/font.md");
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
        ),
        "Runtime 15 M2 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
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
        "ZUI document import owner",
        &ui_v2_document_import,
        &[
            "pub(crate) fn imported_asset_from_ui_v2_document(",
            "UiV2AssetKind::Component => ImportedAsset::UiV2Component",
        ],
    );
    for retired in ["UiV2DocumentImportProfile", "LegacyToml"] {
        assert!(
            !ui_v2_document_import.contains(retired),
            "ZUI document import owner should not keep retired profile branch {retired}"
        );
    }
    assert_contains_all(
        "ZUI importer caller",
        &import_ui_zui_asset,
        &["imported_asset_from_ui_v2_document(parsed)"],
    );
    assert!(
        !import_ui_zui_asset.contains("UiV2DocumentImportProfile::Zui"),
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
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 font/UI asset schema naming hard cutover",
                "runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/assets/font.rs",
                "asset/importer/ingest/ui_v2_document_import.rs",
                "schema_v1_render_mode",
                "runtime_15_font_ui_asset_schema_names_use_current_policy_terms",
            ],
        );
    }
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
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let font_assets_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/assets/font.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
        ),
        "Runtime 15 M2 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

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
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
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
