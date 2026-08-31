use super::*;

const STATUS: &str = "render_plan08_virtual_geometry_resident_buffers_upload_direct_binary_wgpu_passed_renderdoc_deferred";

#[test]
fn runtime_15_virtual_geometry_resident_buffers_upload_is_wired() {
    let gpu_scene_layout = read_runtime_src("graphics/scene/gpu_scene/layout.rs");
    let gpu_scene_runtime = read_runtime_src("graphics/scene/gpu_scene/gpu_scene.rs");
    let gpu_scene_mod = read_runtime_src("graphics/scene/gpu_scene/mod.rs");
    let gpu_scene_virtual_geometry =
        read_runtime_src("graphics/scene/gpu_scene/virtual_geometry.rs");
    let mesh_build_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs");
    let mesh_build =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs");
    let mesh_gpu_scene_sync = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs",
    );
    let mesh_virtual_geometry_upload = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs",
    );
    let submission_detail = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_submission_detail.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let gpu_scene_sources = format!("{gpu_scene_layout}{gpu_scene_runtime}{gpu_scene_mod}");
    let upload_sources = format!("{gpu_scene_virtual_geometry}{mesh_virtual_geometry_upload}");
    let mesh_sources =
        format!("{mesh_build_mod}{mesh_build}{mesh_gpu_scene_sync}{submission_detail}");

    assert_contains_all(
        "GPUScene declares typed VG resident buffer ABI and CPU mirrors",
        &gpu_scene_sources,
        &[
            "GpuVirtualGeometryPage",
            "GpuVirtualGeometryClusterWord",
            "GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT",
            "with_additional_uploaded_bytes",
            "virtual_geometry_pages_shadow",
            "virtual_geometry_clusters_shadow",
            "GpuSceneVirtualGeometryUploadReport",
        ],
    );
    assert_contains_all(
        "GPUScene upload owner writes resident VG page and cluster buffers",
        &upload_sources,
        &[
            "prepare_virtual_geometry_resident_buffers",
            "GpuScenePreparedVirtualGeometryUpload",
            "push_changed_pod_slice",
            "zircon-gpu-scene-virtual-geometry-pages",
            "zircon-gpu-scene-virtual-geometry-clusters",
            "rebuild_scene_bind_group(device)",
            "render_gpu_scene_uploads_virtual_geometry_resident_buffers",
            "render_gpu_scene_dropped_virtual_geometry_preparation_keeps_shadow_for_retry",
            "virtual_geometry_payload_rows_from_snapshot",
            "RenderVirtualGeometryExecutionState::Resident",
        ],
    );
    assert_contains_all(
        "mesh build uploads VG resident rows before GPUScene draw sync",
        &mesh_sources,
        &[
            "mod virtual_geometry_resident_upload;",
            "virtual_geometry_upload",
            "append_virtual_geometry_upload(virtual_geometry_upload)",
            "virtual_geometry_scene_counts",
            "upload_virtual_geometry_resident_payloads(",
            "frame.virtual_geometry_debug_snapshot.as_ref()",
            "virtual_geometry_payload_slot_for_pending_draw",
            ".and_then(|draw_ref| draw_ref.segment_key.submission_slot)",
            "payload_slot,",
            "pub(crate) fn payload_slot(self) -> Option<u32>",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/gpu_scene/gpu_scene.rs",
            gpu_scene_runtime.as_str(),
        ),
        (
            "graphics/scene/gpu_scene/virtual_geometry.rs",
            gpu_scene_virtual_geometry.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs",
            mesh_gpu_scene_sync.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs",
            mesh_virtual_geometry_upload.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_resident_buffers_upload.rs",
            include_str!("virtual_geometry_resident_buffers_upload.rs"),
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
                "VirtualGeometry resident buffers upload",
                STATUS,
                "runtime_15_virtual_geometry_resident_buffers_upload_is_wired",
            ],
        );
    }
}
