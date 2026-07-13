use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_build_mesh_draws_gpu_scene_sync_is_child_owner() {
    let root =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs");
    let module =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs");
    let gpu_scene_sync = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs",
    );
    let geometry_source_selection = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs",
    );

    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");

    assert_contains_all(
        "build_mesh_draws build module mounts GPUScene sync owner",
        &module,
        &[
            "mod build;",
            "mod geometry_source_selection;",
            "mod gpu_scene_sync;",
            "mod previous_skinned_palette;",
        ],
    );

    assert_contains_all(
        "build.rs keeps orchestration and imports the GPUScene sync owner",
        &root,
        &[
            "pub(crate) fn build_mesh_draws(",
            "let (gpu_scene_upload_report, gpu_scene_entries) = sync_gpu_scene_pending_draws(",
            "frame.environment().baked_lighting(),",
            "prepared_mesh_queue_stats_for_pending_draws(",
            "mesh_visibility_states(",
            "submission_detail_from_draw_ref(",
        ],
    );
    for moved_owner in [
        "fn primitive_data_for_pending_draw(",
        "fn instance_data_for_pending_draw(",
        "fn previous_model_matrix_for_gpu_scene_entry(",
        "fn shadow_params_from_pending_draw(",
        "fn motion_params_from_pending_draw(",
        "fn resolved_skinned_gpu_source_for_pending_draw(",
        "fn resolve_skinned_gpu_source_mesh(",
        "GPU_PRIMITIVE_FLAG_VISIBLE",
    ] {
        assert!(
            !root.contains(moved_owner),
            "build.rs should delegate GPUScene sync owner `{moved_owner}` to gpu_scene_sync.rs"
        );
    }

    assert_contains_all(
        "gpu_scene_sync child owns GPUScene payload and skinned source sync helpers",
        &gpu_scene_sync,
        &[
            "pub(super) struct SyncedGpuSceneEntry",
            "pub(super) fn sync_gpu_scene_pending_draws(",
            "lightmaps: Option<&LightmapConsumeContract>",
            "fn primitive_data_for_pending_draw(",
            "fn instance_data_for_pending_draw(",
            "fn previous_model_matrix_for_gpu_scene_entry(",
            "fn shadow_params_from_pending_draw(",
            "fn motion_params_from_pending_draw(",
            "fn resolved_skinned_gpu_source_for_pending_draw(",
        ],
    );
    assert_contains_all(
        "geometry source selection child owns pending draw source classification",
        &geometry_source_selection,
        &[
            "pub(super) fn pending_mesh_source_selection(",
            "pub(super) fn pending_mesh_draw_queue_profile(",
            "pub(super) fn skinned_gpu_source_geometry_source(",
        ],
    );

    for (path, source) in [
        ("build_mesh_draws/build/build.rs", root.as_str()),
        (
            "build_mesh_draws/build/gpu_scene_sync.rs",
            gpu_scene_sync.as_str(),
        ),
        (
            "build_mesh_draws/build/geometry_source_selection.rs",
            geometry_source_selection.as_str(),
        ),
        ("build_mesh_draws/build/mod.rs", module.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the GPUScene sync split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("mesh pass docs", mesh_pass_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "build_mesh_draws GPUScene sync owner split",
                "render_plan02_build_mesh_draws_gpu_scene_sync_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs",
                "runtime_15_build_mesh_draws_gpu_scene_sync_is_child_owner",
            ],
        );
    }
}
