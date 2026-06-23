use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_prepared_mesh_queue_is_folder_backed() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue.rs");
    let stats_bridge =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs");
    let stats_bridge_tests =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs");
    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "prepared queue parent keeps production stats owner surface",
        &parent,
        &[
            "pub(crate) struct PreparedMeshQueue",
            "pub(crate) struct PreparedMeshQueueStats",
            "pub(crate) fn prepare_mesh_queue",
            "pub(crate) fn summarize_prepared_mesh_queue_items",
            "fn repeated_group_stats",
            "mod stats_bridge;",
            "mod stats_bridge_tests;",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "mod tests {",
        "fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask",
        "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
        "fn prepared_queue_stats_carry_gpu_scene_counts",
        "fn render_gpu_scene_upload_path",
        "fn with_mesh_pass_command_buffer_stats",
        "fn item<K>",
        "fn profile(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "prepared_queue.rs should delegate {moved_owner} to prepared_queue/tests.rs"
        );
    }

    assert_contains_all(
        "prepared queue stats bridge child owns cross-subsystem stat forwarding",
        &stats_bridge,
        &[
            "impl PreparedMeshQueueStats",
            "with_pending_command_cache_extraction_stats",
            "with_pending_command_cache_plan_stats",
            "with_mesh_pass_command_buffer_stats",
            "with_mesh_draw_replay_stats",
            "with_gpu_scene_stats",
            "fn render_gpu_scene_upload_path",
            "residual_material_phase_draw_count",
            "pre_mesh_draw_static_command_cache_residual_material_phase_draw_count",
        ],
    );

    assert_contains_all(
        "prepared queue tests child owns queue stats behavior coverage",
        &tests,
        &[
            "fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask",
            "fn prepared_queue_stats_require_repeated_direct_prepared_keys_for_batching",
            "fn prepared_queue_stats_count_dynamic_velocity_history_readiness",
            "fn prepared_queue_stats_count_cpu_morphed_gpu_skinning_source_as_dynamic_geometry",
            "fn prepared_queue_stats_exclude_gpu_skinned_draws_from_direct_batch_candidates",
        ],
    );
    for moved_stat_bridge_test in [
        "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
        "fn prepared_queue_stats_carry_pending_command_cache_plan_counts",
        "fn prepared_queue_stats_carry_pending_command_cache_extraction_counts",
        "fn prepared_queue_stats_carry_mesh_draw_replay_counts",
        "fn prepared_queue_stats_carry_gpu_scene_counts",
    ] {
        assert!(
            !tests.contains(moved_stat_bridge_test),
            "prepared_queue/tests.rs should delegate {moved_stat_bridge_test} to stats_bridge_tests.rs"
        );
    }

    assert_contains_all(
        "prepared queue stats bridge tests child owns forwarding coverage",
        &stats_bridge_tests,
        &[
            "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
            "fn prepared_queue_stats_carry_pending_command_cache_plan_counts",
            "fn prepared_queue_stats_carry_pending_command_cache_extraction_counts",
            "fn prepared_queue_stats_carry_mesh_draw_replay_counts",
            "fn prepared_queue_stats_carry_gpu_scene_counts",
            "PendingMeshCommandCacheExtractionStats",
            "MeshPassCommandBufferStats",
            "GpuSceneUploadReport",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue.rs",
            parent.as_str(),
            280,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
            stats_bridge.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs",
            tests.as_str(),
            620,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs",
            stats_bridge_tests.as_str(),
            220,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("mesh pass doc", mesh_pass_doc.as_str()),
        ("module convention doc", module_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 02 prepared queue tests owner split",
                "Plan 02 prepared queue stats bridge tests owner split",
                "Plan 02 prepared queue stats bridge owner split",
                "render_plan02_prepared_queue_tests_owner_split_static_passed",
                "render_plan02_prepared_queue_stats_bridge_tests_owner_split_static_passed_cargo_lock_blocked",
                "render_plan02_prepared_queue_stats_bridge_owner_split_static_passed",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs",
                "runtime_15_prepared_mesh_queue_is_folder_backed",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
