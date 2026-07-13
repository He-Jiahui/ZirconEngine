use super::*;

const STATUS: &str = "render_plan08_staged_prewarm_product_sweep_wgpu_passed_renderdoc_deferred";
const DEFAULT_FEATURES_DIRECT_BINARY_STATUS: &str = "render_plan08_staged_prewarm_product_sweep_default_features_direct_binary_wgpu_passed_renderdoc_deferred";
const DEFAULT_FEATURES_CARGO_WRAPPER_STATUS: &str = "render_plan08_staged_prewarm_product_sweep_default_features_cargo_wrapper_wgpu_passed_renderdoc_deferred";
const DEFAULT_FEATURES_CURRENT_REFRESH_STATUS: &str = "render_plan08_staged_prewarm_product_sweep_default_features_current_wgpu_refresh_passed_renderdoc_deferred";
const PRODUCT_READBACK_PNG_STATUS: &str = "render_plan08_project_plugin_registry_material_passes_product_readback_png_passed_renderdoc_deferred";

#[test]
fn runtime_15_render_plan08_staged_prewarm_product_sweep_is_wired() {
    let mesh_cache_root = read_runtime_src("graphics/tests/render_product_mesh_cache.rs");
    let base_product =
        read_runtime_src("graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs");
    let material_product = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/staged_prewarm/material_passes.rs",
    );
    let registry_product = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_staged_cache.rs",
    );
    let registry_material_root = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/mod.rs",
    );
    let registry_material_manifest = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/manifest.rs",
    );
    let registry_material_second = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/second_launch.rs",
    );
    let registry_material_custom = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_shading_model.rs",
    );
    let registry_material_custom_second = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_second_launch.rs",
    );
    let registry_material_product_png = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/product_png.rs",
    );
    let ensure_pipeline = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "Plan 08 staged-prewarm product/runtime sweep covers base, registry, material-pass, custom shading, and runtime consumers",
        &(mesh_cache_root
            + &base_product
            + &material_product
            + &registry_product
            + &registry_material_root
            + &registry_material_manifest
            + &registry_material_second
            + &registry_material_custom
            + &registry_material_custom_second
            + &registry_material_product_png
            + &ensure_pipeline),
        &[
            "runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss",
            "runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss",
            "runtime_custom_geometry_descriptor_pipeline_uses_staged_prewarm_without_compile_miss",
            "runtime_project_plugin_registry_shader_keys_use_staged_prewarm_without_compile_miss",
            "render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss",
            "render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss",
            "render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss",
            "render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss",
            "render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss",
            "render_product_material_mesh_passes_second_launch_use_staged_prewarm_without_compile_miss",
            "render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss",
            "export_project_plugin_registry_material_passes_product_png",
            "render_plan08_project_plugin_registry_material_passes_product_readback_png_passed_renderdoc_deferred",
            "runtime_render_plan08_project_plugin_registry_material_passes_20260703.png",
            "submit_registry_material_passes_with_staged_cache_capture",
            "save_side_by_side_product_frames",
            "registry_staged_cache_runtime_surface_source",
            "standard_material_surface_source_for_features",
            "fn zr_material_surface(",
            "MATERIAL_SHADER_TEMPLATE_REVISION",
            ".include_content_hashes",
            ".contains(&request_source_hash)",
            "super::super::registry_staged_cache_runtime_surface_source()",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 08 staged-prewarm product sweep",
                STATUS,
                "Plan 08 staged-prewarm product sweep default-feature direct-binary WGPU backfill",
                DEFAULT_FEATURES_DIRECT_BINARY_STATUS,
                "Plan 08 staged-prewarm product sweep default-feature Cargo-wrapper WGPU backfill",
                DEFAULT_FEATURES_CARGO_WRAPPER_STATUS,
                "Plan 08 staged-prewarm product sweep default-feature current WGPU refresh",
                DEFAULT_FEATURES_CURRENT_REFRESH_STATUS,
                "Plan 08 project/plugin registry material-pass product readback PNG",
                PRODUCT_READBACK_PNG_STATUS,
                "export_project_plugin_registry_material_passes_product_png",
                "runtime_render_plan08_project_plugin_registry_material_passes_20260703.png",
                "2FF919F50FDFFBAEB1544CAD9C14B7748FA8234C784175195AF3E550FB6151BB",
                "6290 filtered",
                "6.58s",
                "4794 non-black pixels",
                "staged_prewarm_without_compile_miss",
                "11 passed",
                "11/11",
                "5924 filtered",
                "56.74s",
                "5925 filtered",
                "56.99s",
                "10m 04s",
                "6187 filtered",
                "48.90s",
                "7m 36s",
                "6197 filtered",
                "5.65s",
                "raw project/plugin registry",
                "fn zr_material_surface(",
                "RenderDoc/product capture",
                "runtime_15_render_plan08_staged_prewarm_product_sweep_is_wired",
            ],
        );
    }
}
