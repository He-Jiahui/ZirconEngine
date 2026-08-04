use super::*;

#[test]
fn runtime_15_gpu_model_embedded_primitive_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gpu_model_source = read_text(
        &manifest_root
            .join("src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs"),
        "GPU model resource from asset owner should be readable",
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
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");

    assert_contains_all(
        "GPU model embedded primitive source names",
        &gpu_model_source,
        &[
            "model_primitives_preferring_mesh_assets",
            "model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved",
            "let embedded = embedded_primitive(",
            "fn embedded_primitive(",
        ],
    );
    for retired_name in [
        "legacy_primitive",
        "keep_legacy_payload",
        "let legacy =",
        "vec![legacy]",
    ] {
        assert!(
            !gpu_model_source.contains(retired_name),
            "GPU model primitive fallback owner should not retain retired `{retired_name}` naming"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render assets doc", render_assets_doc),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 GPU model embedded primitive naming hard cutover",
                "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs",
                "embedded primitive",
                "runtime_15_gpu_model_embedded_primitive_uses_current_names",
            ],
        );
    }
}
