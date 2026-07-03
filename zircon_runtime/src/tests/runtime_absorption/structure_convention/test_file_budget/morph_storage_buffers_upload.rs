use super::*;

const STATUS: &str = "render_plan08_morph_storage_buffers_upload_check_passed_wgpu_deferred";

#[test]
fn runtime_15_morph_storage_buffers_upload_is_wired() {
    let gpu_scene_layout = read_runtime_src("graphics/scene/gpu_scene/layout.rs");
    let gpu_scene_binding = read_runtime_src("graphics/scene/gpu_scene/binding.rs");
    let gpu_scene_runtime = read_runtime_src("graphics/scene/gpu_scene/gpu_scene.rs");
    let gpu_scene_mod = read_runtime_src("graphics/scene/gpu_scene/mod.rs");
    let gpu_scene_morph = read_runtime_src("graphics/scene/gpu_scene/morph.rs");
    let gpu_scene_wgsl =
        read_runtime_src("graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
    let morphed_wgsl = read_runtime_src("graphics/shader/wgsl/zr_geometry_morphed.wgsl");
    let skinned_morphed_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let gpu_scene_doc = read_repo("docs/zircon_runtime/graphics/scene/gpu_scene/mod.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");
    let gpu_scene_sources =
        format!("{gpu_scene_layout}{gpu_scene_binding}{gpu_scene_runtime}{gpu_scene_mod}");

    assert_contains_all(
        "GPUScene declares typed morph storage ABI and reserves bindings 7/8",
        &gpu_scene_sources,
        &[
            "GpuMorphDelta",
            "GpuMorphWeight",
            "GPU_MORPH_DELTA_STRIDE",
            "GPU_MORPH_WEIGHT_STRIDE",
            "GPU_SCENE_MORPH_DELTAS_BINDING: u32 = 7",
            "GPU_SCENE_MORPH_WEIGHTS_BINDING: u32 = 8",
            "GPU_SCENE_MORPH_PAYLOADS_BINDING: u32 = 11",
            "[wgpu::BindGroupLayoutEntry; 12]",
            "storage_binding(GPU_SCENE_MORPH_DELTAS_BINDING",
            "storage_binding(GPU_SCENE_MORPH_WEIGHTS_BINDING",
            "storage_binding(GPU_SCENE_MORPH_PAYLOADS_BINDING",
            "morph_deltas_buffer",
            "morph_weights_buffer",
            "morph_payloads_buffer",
            "morph_deltas_shadow",
            "morph_weights_shadow",
            "morph_payloads_shadow",
            "mod morph;",
            "GpuSceneMorphUploadReport",
        ],
    );
    assert_contains_all(
        "GPUScene morph upload owner writes buffers and rebuilds bind groups",
        &gpu_scene_morph,
        &[
            "upload_morph_buffers",
            "GpuSceneMorphUploadReport",
            "write_full_pod_buffer",
            "zircon-gpu-scene-morph-deltas",
            "zircon-gpu-scene-morph-weights",
            "rebuild_scene_bind_group(device)",
            "render_gpu_scene_uploads_morph_storage_buffers",
        ],
    );
    assert_contains_all(
        "WGSL declares morph storage in GPUScene and geometry includes use helpers",
        &format!("{gpu_scene_wgsl}{morphed_wgsl}{skinned_morphed_wgsl}"),
        &[
            "@group(3) @binding(7) var<storage, read> zr_morph_deltas",
            "@group(3) @binding(8) var<storage, read> zr_morph_weights",
            "@group(3) @binding(11) var<storage, read> zr_morph_payloads",
            "fn zr_gpu_scene_morph_delta(",
            "fn zr_gpu_scene_morph_payload(",
            "fn zr_gpu_scene_morph_weight(",
            "zr_gpu_scene_morph_delta_row(row_index)",
            "zr_gpu_scene_morph_weight(payload.y + target_index)",
        ],
    );
    assert!(
        !morphed_wgsl.contains("@group(3) @binding(7)")
            && !skinned_morphed_wgsl.contains("@group(3) @binding(7)"),
        "morphed geometry includes must not redeclare GPUScene morph storage bindings"
    );

    for (path, source) in [
        ("graphics/scene/gpu_scene/layout.rs", gpu_scene_layout.as_str()),
        (
            "graphics/scene/gpu_scene/binding.rs",
            gpu_scene_binding.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/gpu_scene.rs",
            gpu_scene_runtime.as_str(),
        ),
        ("graphics/scene/gpu_scene/morph.rs", gpu_scene_morph.as_str()),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/morph_storage_buffers_upload.rs",
            include_str!("morph_storage_buffers_upload.rs"),
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
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Morph storage buffers upload",
                STATUS,
                "runtime_15_morph_storage_buffers_upload_is_wired",
            ],
        );
    }
}
