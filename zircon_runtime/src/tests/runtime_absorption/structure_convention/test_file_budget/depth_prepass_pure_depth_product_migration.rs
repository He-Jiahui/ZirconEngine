use super::super::rust_source_view::{production_code_view, production_section};
use super::*;

const STATUS: &str = "render_plan08_runtime_depth_prepass_pure_depth_product_migration_static_passed_cargo_check_renderdoc_deferred";

#[test]
fn runtime_15_depth_prepass_pure_depth_product_migration_is_wired() {
    let variant_registry = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
    );
    let shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let shader_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
    );
    let ensure_depth_prepass = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs",
    );
    let depth_pipeline = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs",
    );
    let depth_pipeline_production = production_section(&depth_pipeline);
    let depth_pipeline_code = production_code_view(&depth_pipeline);
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    assert_contains_all(
        "depth prepass variant identity uses the depth pass type",
        &variant_registry,
        &[
            "MeshPassPipelineKind::DepthPrepass => ShaderPassType::DepthPrepass",
            "mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type",
        ],
    );
    assert_contains_all(
        "runtime depth prepass source uses the depth-only template pass",
        &format!("{shader_source}{shader_source_tests}"),
        &[
            "pub(crate) fn mesh_pipeline_depth_prepass_template_source_for_geometry",
            "ShaderPassType::DepthPrepass",
            "mesh_pipeline_depth_prepass_template_source_uses_depth_only_template",
            "zr_template_depth_alpha.wgsl",
            "!source.wgsl_source.contains(\"surface.normal_ws * 0.5\")",
        ],
    );
    assert_contains_all(
        "depth prepass cache key carries depth prepass identity",
        &ensure_depth_prepass,
        &[
            "DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX",
            "ShaderPassType::DepthPrepass",
            "variant_key.canonical_string()",
        ],
    );
    assert_contains_all(
        "WGPU depth prepass pipeline has no color target",
        &depth_pipeline_production,
        &[
            "depth_write_enabled: Some(true)",
            "key.is_alpha_mask()",
            "entry_point: Some(\"fs_main\")",
            "targets: &[]",
            "None",
        ],
    );
    assert!(
        !depth_pipeline_code.contains("NORMAL_FORMAT"),
        "depth prepass production pipeline must not write the prepass normal target"
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            shader_source.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
            variant_registry.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs",
            depth_pipeline.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs",
            include_str!("depth_prepass_pure_depth_product_migration.rs"),
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
        ("shader doc", shader_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime pure-depth DepthPrepass product migration",
                STATUS,
                "mesh_pipeline_depth_prepass_template_source_uses_depth_only_template",
                "runtime_15_depth_prepass_pure_depth_product_migration_is_wired",
            ],
        );
    }
}
