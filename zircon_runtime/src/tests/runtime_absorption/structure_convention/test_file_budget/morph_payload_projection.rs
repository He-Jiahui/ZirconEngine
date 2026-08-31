use super::*;

const STATUS: &str = "render_plan08_morph_payload_projection_check_passed_wgpu_deferred";

#[test]
fn runtime_15_morph_payload_projection_is_wired() {
    let morph_payload_upload = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs",
    );
    let build_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs");
    let build =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs");
    let extend = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
    );
    let pending = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs",
    );
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
        "morph payload upload projects direct mesh assets into GPUScene rows",
        &morph_payload_upload,
        &[
            "morph_payload_from_mesh_asset",
            "MESH_ATTRIBUTE_POSITION",
            "as_float32x3",
            "GpuMorphDelta::position_xyz",
            "GpuMorphWeight::new",
            "upload_morph_payloads",
            "collect_morph_payload_rows",
            "Arc::as_ptr",
            "prepare_morph_buffers",
            "morph_payload_projection_keeps_active_position_deltas_and_weights",
            "morph_payload_collection_deduplicates_shared_draw_payloads",
        ],
    );
    assert_contains_all(
        "mesh build mounts and calls the morph payload upload child owner",
        &format!("{build_mod}{build}"),
        &[
            "mod morph_payload_upload;",
            "upload_morph_payloads",
            "let morph_upload = upload_morph_payloads(device, gpu_scene, &mut pending_draws);",
            "append_morph_upload(morph_upload)",
        ],
    );
    assert_contains_all(
        "pending direct mesh draws carry optional shared morph payloads",
        &format!("{extend}{pending}"),
        &[
            "PendingMorphPayload",
            "morph_payload: Option<Arc<PendingMorphPayload>>",
            "direct_mesh_morph_payload",
            "direct_mesh_morph_payload(streamer, mesh_id, mesh_instance, previous_morph_weights)",
            "morph_payload: Some(payload)",
            "morph_payload: morph_payload.clone()",
            "morph_payload: None",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs",
            build.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            extend.as_str(),
        ),
    ] {
        assert!(
            !source.contains("GEOMETRY_SOURCE_ID_MORPHED_MESH")
                && !source.contains("GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH"),
            "{path} must not select morphed or skinned-morphed shader ids while direct draws are still CPU-baked"
        );
    }

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs",
            build.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            extend.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs",
            morph_payload_upload.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_projection.rs",
            include_str!("morph_payload_projection.rs"),
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
                "Morph payload projection",
                STATUS,
                "runtime_15_morph_payload_projection_is_wired",
            ],
        );
    }
}
