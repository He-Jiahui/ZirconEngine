use super::*;

const PRODUCT_STAGED_PREWARM_OWNER_SPLIT_VELOCITY_STATUS: &str = "render_plan08_product_material_mesh_staged_prewarm_owner_split_velocity_static_passed_cargo_deferred_active_lanes";

#[test]
fn runtime_15_product_base_mesh_staged_prewarm_is_wired() {
    let product_parent = read_runtime_src("graphics/tests/render_product_mesh_cache.rs");
    let product_staged_dir = "graphics/tests/render_product_mesh_cache/staged_prewarm";
    assert!(
        !repo_path("zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm.rs")
            .exists(),
        "product staged prewarm owner should stay folder-backed instead of returning to one oversized file"
    );
    let product_staged_mod = read_runtime_src(&format!("{product_staged_dir}/mod.rs"));
    let product_staged_material_passes =
        read_runtime_src(&format!("{product_staged_dir}/material_passes.rs"));
    let product_staged = [
        product_staged_mod.as_str(),
        product_staged_material_passes.as_str(),
    ]
    .join("\n");
    let mesh_cache = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
    );
    let renderer_submit = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs",
    );
    let wgpu_framework = read_runtime_src(
        "graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let product_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "render product mesh cache parent mounts staged prewarm child",
        &product_parent,
        &["mod staged_prewarm;"],
    );
    assert_contains_all(
        "product staged prewarm test covers second-launch Base mesh cache hits",
        &product_staged,
        &[
            "render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss",
            "submit_base_mesh_with_staged_cache",
            "base_mesh_shader_cache_product_manifest",
            "base_mesh_shader_cache_product_pipeline",
            "base_mesh_shader_cache_product_feature",
            "prewarm_shader_variants",
            "builtin_fallback_shader_prewarm_manifest",
            "builtin_standard_material_shader_prewarm_manifest_for_geometry",
            "ShaderVariantCacheDisk::with_fallback_roots",
            "replace_shader_variant_disk_cache_for_tests",
            "static_cache_skinned_extract",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
            "DisplayMode::Shaded",
            ".with_executor_id(\"mesh.opaque\")",
            ".with_side_effects()",
            "last_shader_variant_miss_report",
            "dimension_summary",
            "assert_staged_prewarm_runtime_dimension_correlation",
            "ShaderVariantPrewarmDimensionCount",
            "ShaderVariantRuntimeDimensionCount",
            "last_mesh_replay_state_change_count",
            "last_mesh_skinned_draw_count",
            "last_graph_executed_executor_ids",
            "compile_miss_count",
            "disk_hit_count",
            "disk_write_count",
            "disk_error_count",
            "mod material_passes;",
            "render_product_material_mesh_passes_second_launch_use_staged_prewarm_without_compile_miss",
            "temporal.velocity-object",
            "last_mesh_previous_velocity_transform_draw_count",
            "last_mesh_missing_velocity_transform_draw_count, 0",
            "report.dimension_summary.pass_types.get(\"velocity\")",
            "velocity pass",
            "previous velocity transform",
        ],
    );

    for (label, source) in [
        ("mesh pipeline cache", mesh_cache.as_str()),
        ("scene renderer submit seam", renderer_submit.as_str()),
        ("wgpu framework seam", wgpu_framework.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "replace_shader_variant_disk_cache_for_tests",
                "ShaderVariantCacheDisk",
            ],
        );
    }

    for (path, source) in [
        (
            "graphics/tests/render_product_mesh_cache.rs",
            product_parent.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs",
            product_staged_mod.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/staged_prewarm/material_passes.rs",
            product_staged_material_passes.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("template doc", template_doc.as_str()),
        ("mesh pipeline cache doc", mesh_cache_doc.as_str()),
        ("render product submit doc", product_submit_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Product Base mesh second-launch staged prewarm",
                "render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred",
                "render_plan08_runtime_shader_variant_dimension_correlation_product_passed_renderdoc_deferred",
                "render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss",
                "graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs",
                "runtime_15_product_base_mesh_staged_prewarm_is_wired",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Product material mesh staged prewarm owner split + Velocity runtime contract",
                PRODUCT_STAGED_PREWARM_OWNER_SPLIT_VELOCITY_STATUS,
                "staged_prewarm/material_passes.rs",
                "temporal.velocity-object",
                "velocity pass",
                "previous velocity transform",
            ],
        );
    }
}
