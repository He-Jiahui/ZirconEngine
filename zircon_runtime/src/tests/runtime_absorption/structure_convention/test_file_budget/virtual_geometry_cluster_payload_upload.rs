use super::*;

const STATUS: &str =
    "render_plan08_virtual_geometry_cluster_payload_upload_direct_binary_wgpu_passed_renderdoc_deferred";

#[test]
fn runtime_15_virtual_geometry_cluster_payload_upload_is_wired() {
    let vg_root = read_runtime_src("core/framework/render/virtual_geometry_debug_snapshot.rs");
    let vg_payload =
        read_runtime_src("core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs");
    let vg_snapshot =
        read_runtime_src("core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs");
    let render_mod = read_runtime_src("core/framework/render/mod.rs");
    let snapshot_builder = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs",
    );
    let gpu_scene_layout = read_runtime_src("graphics/scene/gpu_scene/layout.rs");
    let gpu_scene_mod = read_runtime_src("graphics/scene/gpu_scene/mod.rs");
    let resident_upload = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs",
    );
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let vg_snapshot_doc =
        read_repo("docs/zircon_runtime/core/framework/render/virtual_geometry_debug_snapshot.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");
    let debug_snapshot_sources = format!("{vg_root}{vg_payload}{vg_snapshot}{render_mod}");
    let gpu_scene_sources = format!("{gpu_scene_layout}{gpu_scene_mod}");

    assert_contains_all(
        "VG debug snapshot exposes resident page payload contract",
        &debug_snapshot_sources,
        &[
            "mod page_payload;",
            "RenderVirtualGeometryPagePayload",
            "RenderVirtualGeometryPagePayloadVertex",
            "pub resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>",
            "pub position: Vec3",
            "pub normal: Vec3",
            "pub tangent: Vec4",
        ],
    );
    assert_contains_all(
        "production snapshot consumes context-carried resident page payloads",
        &snapshot_builder,
        &["context.virtual_geometry_resident_page_payloads().to_vec()"],
    );
    assert_contains_all(
        "GPUScene ABI declares cluster words per vertex for payload upload",
        &gpu_scene_sources,
        &[
            "GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX",
            "GpuVirtualGeometryClusterWord",
        ],
    );
    assert_contains_all(
        "resident upload projects snapshot payload vertices into cluster words",
        &resident_upload,
        &[
            "virtual_geometry_payload_rows_from_snapshot",
            "resident_page_payloads",
            "append_page_payload_cluster_words",
            "append_vertex_payload_cluster_words",
            "GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX",
            "vertex.position.x",
            "vertex.normal.x",
            "vertex.tangent.to_array()",
            "virtual_geometry_cluster_words_follow_resident_page_payloads",
        ],
    );

    for (path, source) in [
        (
            "core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs",
            vg_payload.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs",
            resident_upload.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_cluster_payload_upload.rs",
            include_str!("virtual_geometry_cluster_payload_upload.rs"),
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
        ("VG debug snapshot doc", vg_snapshot_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry cluster payload upload",
                STATUS,
                "runtime_15_virtual_geometry_cluster_payload_upload_is_wired",
            ],
        );
    }
}
