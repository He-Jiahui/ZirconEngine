use super::*;

#[test]
fn runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let camera_source = read_text(
        &manifest_root.join("src/core/framework/render/camera.rs"),
        "render camera source should be readable",
    );
    let scene_render_files = [
        "src/scene/world/render.rs",
        "src/scene/world/render/lights.rs",
        "src/scene/world/render_particles.rs",
        "src/scene/world/render_post_process.rs",
        "src/scene/world/render_visibility.rs",
    ];
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
    let camera_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/camera.md",
    );

    assert_contains_all(
        "RenderLayerSet scene schema v1 mask API",
        &camera_source,
        &[
            "pub fn from_scene_schema_v1_mask(mask: u32) -> Self",
            "pub fn to_scene_schema_v1_mask_lossy(&self) -> u32",
            "pub fn intersects_scene_schema_v1_mask(&self, mask: u32) -> bool",
        ],
    );

    for relative_path in scene_render_files {
        let source = read_text(
            &manifest_root.join(relative_path),
            "scene render source should be readable",
        );
        assert!(
            !source.contains("legacy"),
            "{relative_path} should not keep legacy scene schema/render layer naming"
        );
        assert!(
            !source.contains("from_legacy_mask")
                && !source.contains("to_legacy_mask_lossy")
                && !source.contains("intersects_legacy_mask"),
            "{relative_path} should use scene_schema_v1 render layer mask APIs"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render camera doc", camera_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover",
                "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
                "from_scene_schema_v1_mask",
                "runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names",
            ],
        );
    }
}
