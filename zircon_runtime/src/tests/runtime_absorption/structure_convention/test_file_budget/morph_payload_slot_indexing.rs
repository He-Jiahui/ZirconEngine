use super::*;

const STATUS: &str = "render_plan08_morph_payload_slot_indexing_check_passed_wgpu_deferred";

#[test]
fn runtime_15_morph_payload_slot_indexing_is_wired() {
    let gpu_scene_layout = read_runtime_src("graphics/scene/gpu_scene/layout.rs");
    let gpu_scene_binding = read_runtime_src("graphics/scene/gpu_scene/binding.rs");
    let gpu_scene_runtime = read_runtime_src("graphics/scene/gpu_scene/gpu_scene.rs");
    let gpu_scene_morph = read_runtime_src("graphics/scene/gpu_scene/morph.rs");
    let gpu_scene_wgsl =
        read_runtime_src("graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
    let morph_upload = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs",
    );
    let pending = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs",
    );
    let gpu_scene_sync = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs",
    );
    let morphed_wgsl = read_runtime_src("graphics/shader/wgsl/zr_geometry_morphed.wgsl");
    let skinned_morphed_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl");
    let surface_types = read_runtime_src("graphics/shader/wgsl/zr_surface_types.wgsl");
    let forward_template = read_runtime_src("graphics/shader/wgsl/zr_template_forward.wgsl");
    let velocity_template = read_runtime_src("graphics/shader/wgsl/zr_template_velocity.wgsl");
    let velocity_alpha_template =
        read_runtime_src("graphics/shader/wgsl/zr_template_velocity_alpha.wgsl");
    let virtual_geometry_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let gpu_scene_doc = read_repo("docs/zircon_runtime/graphics/scene/gpu_scene/mod.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "GPUScene layout owns morph payload metadata without reusing VG payload_slot",
        &format!("{gpu_scene_layout}{gpu_scene_binding}{gpu_scene_runtime}{gpu_scene_morph}"),
        &[
            "GpuMorphPayload",
            "GPU_MORPH_PAYLOAD_STRIDE",
            "GPU_INSTANCE_DATA_MORPH_PAYLOAD_SLOT_OFFSET",
            "morph_payload_slot: u32",
            "GPU_SCENE_MORPH_PAYLOADS_BINDING: u32 = 11",
            "[wgpu::BindGroupLayoutEntry; 12]",
            "morph_payloads_buffer",
            "morph_payloads_shadow",
            "prepare_morph_buffers",
            "payload_count",
        ],
    );
    assert_contains_all(
        "morph upload creates payload headers and writes draw slots",
        &format!("{morph_upload}{pending}{gpu_scene_sync}"),
        &[
            "PendingMorphPayload",
            "vertex_count: u32",
            "target_count: u32",
            "morph_payload_slot: Option<u32>",
            "GpuMorphPayload::new",
            "MORPH_DELTA_ROWS_PER_VERTEX_TARGET",
            "MESH_ATTRIBUTE_NORMAL",
            "MESH_ATTRIBUTE_TANGENT",
            "MESH_ATTRIBUTE_COLOR",
            "weights.push(GpuMorphWeight::new(weight))",
            "pending_draw.morph_payload_slot =",
            "morph_payload_slot: pending_draw",
            "morph_payload_projection_keeps_normal_tangent_and_color_delta_rows",
        ],
    );
    assert_contains_all(
        "WGSL indexes morph rows through instance slot plus vertex_index",
        &format!(
            "{gpu_scene_wgsl}{morphed_wgsl}{skinned_morphed_wgsl}{surface_types}{forward_template}{velocity_template}{velocity_alpha_template}"
        ),
        &[
            "@group(3) @binding(11) var<storage, read> zr_morph_payloads",
            "fn zr_gpu_scene_morph_payload(",
            "@builtin(vertex_index) vertex_index: u32",
            "input.vertex_index = v.vertex_index",
            "instance.morph_payload_slot",
            "v.vertex_index",
            "ZR_MORPH_DELTA_ROWS_PER_VERTEX_TARGET",
            "ZR_MORPH_POSITION_ROW",
            "ZR_MORPH_NORMAL_ROW",
            "ZR_MORPH_TANGENT_ROW",
            "ZR_MORPH_COLOR_ROW",
            "fetch_color(v, instance_index)",
            "fetch_color(current_input, instance_index)",
        ],
    );
    assert!(
        !format!("{morphed_wgsl}{skinned_morphed_wgsl}").contains("instance_index + v.joints.x"),
        "morph WGSL must not use the old instance-index plus joint-index placeholder"
    );
    assert_contains_all(
        "VirtualGeometry keeps primitive payload_slot ownership separate",
        &virtual_geometry_wgsl,
        &["zr_gpu_scene_primitive_for_instance(instance_index).payload_slot"],
    );

    for (path, source) in [
        (
            "graphics/scene/gpu_scene/layout.rs",
            gpu_scene_layout.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/binding.rs",
            gpu_scene_binding.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/gpu_scene.rs",
            gpu_scene_runtime.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/morph.rs",
            gpu_scene_morph.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs",
            morph_upload.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_slot_indexing.rs",
            include_str!("morph_payload_slot_indexing.rs"),
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
        ("GPUScene doc", gpu_scene_doc.as_str()),
        ("mesh cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Morph payload slot indexing",
                STATUS,
                "runtime_15_morph_payload_slot_indexing_is_wired",
            ],
        );
    }
}
