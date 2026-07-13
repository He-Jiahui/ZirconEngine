use super::*;

const STATUS: &str = "render_plan08_virtual_geometry_page_cluster_shader_bindings_direct_binary_wgpu_layout_passed_renderdoc_deferred";
const CARGO_WRAPPER_STATUS: &str = "render_plan08_virtual_geometry_page_cluster_shader_bindings_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred";

#[test]
fn runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired() {
    let geometry_source = read_runtime_src("core/framework/render/shader/geometry_source.rs");
    let virtual_geometry_plugin =
        read_repo("zircon_plugins/virtual_geometry/runtime/src/plugin.rs");
    let virtual_geometry_static_manifest = read_repo("zircon_plugins/virtual_geometry/plugin.toml");
    let gpu_scene_binding = read_runtime_src("graphics/scene/gpu_scene/binding.rs");
    let gpu_scene_runtime = read_runtime_src("graphics/scene/gpu_scene/gpu_scene.rs");
    let mesh_pipeline_ensure = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
    );
    let mesh_pipeline_shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let gpu_scene_wgsl =
        read_runtime_src("graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
    let virtual_geometry_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl");
    let permutation_registry =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let shader_prewarm_tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let package_manifest_tests =
        read_runtime_src("tests/plugin_extensions/package_manifest_declarations.rs");
    let build_tool_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let virtual_geometry_descriptor_sources =
        format!("{virtual_geometry_plugin}{virtual_geometry_static_manifest}");
    let gpu_scene_sources = format!("{gpu_scene_binding}{gpu_scene_runtime}");
    let virtual_geometry_shader_sources = format!("{gpu_scene_wgsl}{virtual_geometry_wgsl}");
    let prewarm_binding_sources = format!(
        "{permutation_registry}{shader_prewarm_tests}{package_manifest_tests}{build_tool_tests}{mesh_pipeline_ensure}{mesh_pipeline_shader_source}"
    );

    assert_contains_all(
        "framework geometry binding contract exposes both VG resources",
        &geometry_source,
        &[
            "VirtualGeometryPages",
            "VirtualGeometryClusters",
            "GeometrySourceBindingKind",
        ],
    );
    assert_contains_all(
        "virtual geometry plugin descriptor declares page and cluster resource slots",
        &virtual_geometry_descriptor_sources,
        &[
            "GeometrySourceBindingKind::VirtualGeometryPages",
            "GeometrySourceBindingKind::VirtualGeometryClusters",
            "\"virtual_geometry.pages\"",
            "\"virtual_geometry.clusters\"",
            "virtual_geometry_clusters",
        ],
    );
    assert_contains_all(
        "GPUScene layout reserves non-morph storage slots for VG pages and clusters",
        &gpu_scene_sources,
        &[
            "GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING: u32 = 9",
            "GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING: u32 = 10",
            "virtual_geometry_pages_buffer",
            "virtual_geometry_clusters_buffer",
            "zircon-gpu-scene-virtual-geometry-pages-fallback",
            "zircon-gpu-scene-virtual-geometry-clusters-fallback",
        ],
    );
    assert_contains_all(
        "WGSL declares and fetches resident VG page and cluster data",
        &virtual_geometry_shader_sources,
        &[
            "@group(3) @binding(9) var<storage, read> zr_virtual_geometry_pages",
            "@group(3) @binding(10) var<storage, read> zr_virtual_geometry_clusters",
            "fn zr_virtual_geometry_payload_slot(",
            "fn zr_virtual_geometry_vertex_word_index(",
            "fn zr_virtual_geometry_has_vertex(",
            "zr_virtual_geometry_clusters[word_index + word_offset]",
            "return v.position;",
            "return v.normal;",
        ],
    );
    assert_contains_all(
        "prewarm and build tests carry both VG required binding records",
        &prewarm_binding_sources,
        &[
            "virtual_geometry_pages",
            "virtual_geometry_clusters",
            "virtual_geometry.pages",
            "virtual_geometry.clusters",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/gpu_scene/binding.rs",
            gpu_scene_binding.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/gpu_scene.rs",
            gpu_scene_runtime.as_str(),
        ),
        (
            "graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl",
            virtual_geometry_wgsl.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_page_cluster_shader_bindings.rs",
            include_str!("virtual_geometry_page_cluster_shader_bindings.rs"),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            mesh_pipeline_shader_source.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 900,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
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
                "VirtualGeometry page/cluster shader bindings",
                STATUS,
                "VirtualGeometry page/cluster shader bindings Cargo-wrapper WGPU-layout backfill",
                CARGO_WRAPPER_STATUS,
                "1/1",
                "runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired",
            ],
        );
    }
}
