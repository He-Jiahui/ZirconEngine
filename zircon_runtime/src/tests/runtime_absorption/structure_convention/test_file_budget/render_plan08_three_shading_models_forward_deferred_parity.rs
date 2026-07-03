use super::*;

const STATUS: &str =
    "render_plan08_three_shading_models_forward_deferred_parity_wgpu_passed_light_grid_fallback_renderdoc_deferred";
const DEFAULT_FEATURES_STATUS: &str =
    "render_plan08_three_shading_models_forward_deferred_parity_default_features_wgpu_passed_renderdoc_deferred";
const DEFERRED_PROBE_STATUS: &str =
    "render_plan08_deferred_project_shader_gbuffer_probe_wgpu_passed_renderdoc_deferred";
const DEFERRED_PROBE_DEFAULT_STATUS: &str =
    "render_plan08_deferred_project_shader_gbuffer_probe_default_features_wgpu_refresh_passed_renderdoc_deferred";
const PRODUCT_READBACK_PNG_STATUS: &str =
    "render_plan08_three_shading_models_forward_deferred_product_readback_png_passed_renderdoc_deferred";

#[test]
fn runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired() {
    let product_parent = read_runtime_src("graphics/tests/render_product_mesh_cache.rs");
    let product_test =
        read_runtime_src("graphics/tests/render_product_mesh_cache/shading_model_parity.rs");
    let project_render = read_runtime_src("graphics/tests/project_render.rs");
    let render_quality = read_runtime_src("graphics/tests/project_render/render_quality.rs");
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let mesh_shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let mesh_shader_source_runtime_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs",
    );
    let execution_resources = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs",
    );
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "Plan 08 three shading-model render_product parity test is wired",
        &product_test,
        &[
            "render_product_three_shading_models_forward_deferred_parity",
            STATUS,
            "RenderMaterialLightingModel::Pbr",
            "RenderMaterialLightingModel::BlinnPhong",
            "RenderMaterialLightingModel::Unlit",
            "RenderPipelineHandle::new(1)",
            "RenderPipelineHandle::new(2)",
            "deferred.gbuffer",
            "lighting.deferred",
            "assert_rgba_frames_nearly_equal",
            "export_three_shading_models_forward_deferred_product_png",
            PRODUCT_READBACK_PNG_STATUS,
            "runtime_render_plan08_three_shading_models_forward_deferred_product_20260703.png",
            "save_side_by_side_product_frames",
            "ImageBuffer::<Rgba<u8>, _>::from_raw",
            "receive_shadows",
            "with_clustered_lighting(false)",
        ],
    );
    assert_contains_all(
        "render product mesh cache parent declares shading-model parity child",
        &product_parent,
        &["mod shading_model_parity;"],
    );
    assert_contains_all(
        "Plan 08 light-grid disabled clustered fallback is bound",
        &execution_resources,
        &[
            "bind_light_grid_external_buffers",
            "LightGridParams::disabled()",
            "LIGHT_GRID_EMPTY_ZBIN_HEADER",
            ":light-grid-execution-fallback",
            "light_grid_external_fallback_buffers_satisfy_materialization_report",
        ],
    );
    assert_contains_all(
        "Plan 08 Deferred project-shader probe samples the covered region",
        &project_render,
        &[
            "CapturedFrame",
            "fn average_channel_in_region(",
            "origin: UVec2",
            "frame.rgba[index]",
        ],
    );
    assert_contains_all(
        "Plan 08 Deferred project-shader probe separates project WGSL from GBuffer material shading",
        &render_quality,
        &[
            "deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path",
            "write_flat_color_wgsl",
            "[0.0, 1.0, 0.0]",
            "let sample_origin = UVec2::new(viewport_size.x / 4, viewport_size.y / 4);",
            "average_channel_in_region(&forward_frame, sample_origin, sample_size, 0)",
            "average_channel_in_region(&deferred_frame, sample_origin, sample_size, 1)",
        ],
    );
    assert_contains_all(
        "Plan 08 Deferred project-shader probe keeps full-pass project WGSL off material template path",
        &resource_streamer_accessors,
        &[
            "shader_uses_material_surface_source",
            "shader.runtime.kind.participates_in_material_variants()",
            "shader.runtime.source.contains(\"fn zr_material_surface\")",
        ],
    );
    assert_contains_all(
        "Plan 08 Deferred project-shader probe uses the narrow runtime material-surface predicate",
        &mesh_shader_source,
        &[
            "shader_source_uses_runtime_material_surface",
            "streamer.shader_uses_material_surface_source(&key.shader_id)",
            "MeshPipelineShaderSource::from_raw_wgsl",
        ],
    );
    assert_contains_all(
        "Plan 08 Deferred project-shader probe low-level regression covers full-pass Surface assets",
        &mesh_shader_source_runtime_tests,
        &[
            "runtime_surface_shader_with_full_pass_entry_points_uses_raw_wgsl_source",
            "FULL_PASS_WGSL",
            "assert!(!streamer.shader_uses_material_surface_source(&key.shader_id))",
            "assert_eq!(source.template_revision, MESH_SHADER_TEMPLATE_REVISION)",
            "zircon-test-full-pass-project-raw-wgsl",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs",
            product_parent.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/shading_model_parity.rs",
            product_test.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs",
            execution_resources.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/project_render.rs",
            project_render.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/project_render/render_quality.rs",
            render_quality.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            mesh_shader_source.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs",
            mesh_shader_source_runtime_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_plan08_three_shading_models_forward_deferred_parity.rs",
            include_str!("render_plan08_three_shading_models_forward_deferred_parity.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 08 three shading-model forward/deferred product parity",
                "Plan 08 three shading-model forward/deferred product parity default-feature WGPU backfill",
                STATUS,
                DEFAULT_FEATURES_STATUS,
                DEFERRED_PROBE_STATUS,
                DEFERRED_PROBE_DEFAULT_STATUS,
                PRODUCT_READBACK_PNG_STATUS,
                "render_product_three_shading_models_forward_deferred_parity",
                "export_three_shading_models_forward_deferred_product_png",
                "runtime_render_plan08_three_shading_models_forward_deferred_product_20260703.png",
                "runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired",
                "PBR/Blinn-Phong/Unlit",
                "Forward + Deferred",
                "light_grid_external_fallback_buffers_satisfy_materialization_report",
                "deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path",
                "average_channel_in_region",
                "runtime_surface_shader_with_full_pass_entry_points_uses_raw_wgsl_source",
                "shader_uses_material_surface_source",
                "default-feature",
                "5876 filtered",
                "11.81s",
                "6202 filtered",
                "3.56s",
                "Base pass",
                "product readback PNG",
                "RenderDoc/product capture",
            ],
        );
    }

    assert_contains_all(
        "Plan 08 three shading-model default-feature backfill is recorded in material docs",
        &material_doc,
        &[
            "Plan 08 three shading-model forward/deferred product parity default-feature WGPU backfill",
            DEFAULT_FEATURES_STATUS,
            "render_product_three_shading_models_forward_deferred_parity",
            "PBR/Blinn-Phong/Unlit",
            "Forward + Deferred",
            "default-feature",
            "5876 filtered",
            "11.81s",
        ],
    );
}
