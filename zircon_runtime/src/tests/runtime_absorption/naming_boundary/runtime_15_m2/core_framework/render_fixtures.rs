use super::*;

#[test]
fn runtime_15_core_framework_render_fixtures_use_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_queue = read_text(
        &manifest_root.join("src/core/framework/render/core_pipeline/render_queue.rs"),
        "render queue fixture source should be readable",
    );
    let effect_stack = read_text(
        &manifest_root.join("src/core/framework/render/post_process/effect_stack_settings.rs"),
        "post-process effect stack fixture source should be readable",
    );
    let relevance = read_text(
        &manifest_root.join("src/core/framework/render/relevance.rs"),
        "render relevance fixture source should be readable",
    );
    let light_readiness = read_text(
        &manifest_root.join("src/core/framework/render/light/readiness.rs"),
        "render light readiness fixture source should be readable",
    );
    let scene_extract = read_text(
        &manifest_root.join("src/core/framework/render/scene_extract.rs"),
        "render scene extract fixture source should be readable",
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
    let common_render_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/common_api.md",
    );

    assert_contains_all(
        "core framework render fixture current names",
        &render_queue,
        &["authored_queue_offsets_are_clamped_to_material_window"],
    );
    assert!(!render_queue.contains("authored_legacy_offsets"));
    assert_contains_all(
        "post-process effect stack fixture current names",
        &effect_stack,
        &["extended_effect_stack_settings_enable_product_node_without_retired_fields"],
    );
    assert!(!effect_stack.contains("without_legacy_fields"));
    assert_contains_all(
        "primitive relevance fixture current names",
        &relevance,
        &["primitive_relevance_preserves_layers_above_scene_schema_v1_mask_width"],
    );
    assert!(!relevance.contains("above_legacy_mask_width"));
    assert_contains_all(
        "typed scene-schema-v1 mask fixtures",
        &(light_readiness + "\n" + &scene_extract),
        &[
            "RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)",
            "RenderLayerSet::from_scene_schema_v1_mask(u32::MAX)",
        ],
    );
    assert!(!scene_extract.contains("RenderLayerSet::from_legacy_mask(u32::MAX)"));

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("common render API doc", common_render_doc),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 core framework render fixture naming hard cutover",
                "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/render/core_pipeline/render_queue.rs",
                "scene_schema_v1_mask",
                "runtime_15_core_framework_render_fixtures_use_current_names",
            ],
        );
    }
}
