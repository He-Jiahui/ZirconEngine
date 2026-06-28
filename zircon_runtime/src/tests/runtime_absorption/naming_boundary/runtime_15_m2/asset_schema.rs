use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_material_asset_schema_v1_defaults_use_versioned_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let material_asset = read_text(
        &manifest_root.join("src/asset/assets/material/material_asset.rs"),
        "material asset owner should be readable",
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
    let zmeta_material_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/asset/zmeta-shader-material.md",
    );
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
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
        "material asset schema-v1 defaults owner",
        &material_asset,
        &[
            "property_overrides_with_schema_v1_defaults",
            "texture_slots_with_schema_v1_defaults",
            "schema_v1_pbr_texture_slots",
        ],
    );
    for retired in [
        concat!("property_overrides_with_", "legacy", "_defaults"),
        concat!("texture_slots_with_", "legacy", "_defaults"),
        concat!("legacy_", "texture_slots"),
    ] {
        assert!(
            !material_asset.contains(retired),
            "material asset owner should not keep retired generic migration helper name {retired}"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("zmeta material doc", zmeta_material_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover",
                "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/assets/material/material_asset.rs",
                "property_overrides_with_schema_v1_defaults",
                "naming_boundary/runtime_15_m2/asset_schema.rs",
                "runtime_15_material_asset_schema_v1_defaults_use_versioned_names",
            ],
        );
    }
}
