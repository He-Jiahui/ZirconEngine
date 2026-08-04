use super::*;

#[test]
fn runtime_15_material_asset_schema_v1_defaults_use_versioned_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let material_asset = read_text(
        &manifest_root.join("src/asset/assets/material/material_asset.rs"),
        "material asset owner should be readable",
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
    let zmeta_material_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/asset/zmeta-shader-material.md",
    );
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");

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
