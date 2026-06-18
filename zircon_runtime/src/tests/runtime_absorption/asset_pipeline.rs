const EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "dangling_handle_queries_report_not_loaded_instead_of_panicking",
    "failed_asset_exposes_failure_reason_through_facade",
    "resource_state_rejects_error_to_ready_without_reloading",
    "resource_state_recovers_from_error_only_through_reloading",
    "resource_state_rejects_reload_failure_without_reload_boundary",
    "asset_load_state_projection_matches_resource_record_matrix",
    "worker_pool_unbounded_mode_is_explicit_opt_in",
    "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
    "concurrent_requests_for_same_asset_decode_once_and_notify_all",
    "worker_pool_diagnostics_track_in_flight_and_failure_counts",
    "worker_pool_frame_sampler_records_per_frame_completion_deltas",
    "project_asset_manager_spawns_worker_pool_with_frame_sampler",
    "rapid_successive_writes_within_debounce_window_emit_single_reload",
    "watcher_failure_on_removed_directory_surfaces_observable_error",
    "hot_reload_transitions_through_reloading_state_and_emits_modified_event",
    "reload_failure_emits_reload_failed_event_and_lands_failed_state",
    "artifact_store_roundtrips_scene_assets_with_mesh_references",
    "artifact_store_roundtrips_scene_assets_with_camera_targets",
    "artifact_store_roundtrips_scene_assets_with_physics_components",
    "artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
];

#[test]
fn runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");

    for source_file in [
        "src/asset/facade/handle.rs",
        "src/asset/facade/assets.rs",
        "src/asset/facade/load_state.rs",
        "src/asset/facade/manager.rs",
        "src/asset/facade/event.rs",
        "src/asset/pipeline/worker_pool.rs",
        "src/asset/pipeline/manager/project_asset_manager/construction.rs",
        "src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs",
        "src/asset/pipeline/manager/project_asset_manager/runtime.rs",
        "src/asset/pipeline/manager/resource_sync/register_project_resource.rs",
        "src/asset/watch/asset_watcher.rs",
        "src/asset/watch/watch_loop.rs",
        "src/asset/watch/asset_watch_error.rs",
        "src/asset/artifact/cache_payload.rs",
        "src/asset/artifact/cache_payload/json_value.rs",
        "src/asset/artifact/cache_payload/mesh.rs",
        "src/asset/artifact/cache_payload/scene.rs",
        "src/asset/artifact/cache_payload/toml_value.rs",
        "src/asset/artifact/store.rs",
        "src/asset/module.rs",
        "src/core/resource/manager/registry_ops.rs",
        "../zircon_runtime_interface/src/resource/resource_record.rs",
    ] {
        assert!(
            runtime_root.join(source_file).exists(),
            "Runtime 04 audited source `{source_file}` is missing; update asset_pipeline_boundary before changing asset ownership"
        );
    }

    for guard_file in [
        "src/asset/tests/facade/handle_lifecycle.rs",
        "src/asset/tests/facade/failure_reason.rs",
        "src/asset/tests/facade/hot_reload.rs",
        "src/asset/tests/pipeline/worker_pool.rs",
        "src/asset/tests/watcher.rs",
        "src/asset/tests/assets/artifact_store.rs",
        "src/core/resource/tests.rs",
        "src/tests/runtime_absorption/asset_surface.rs",
        "src/tests/runtime_absorption/asset_worker_policy.rs",
        "src/tests/runtime_absorption/asset_pipeline.rs",
        "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
    ] {
        assert!(
            runtime_root.join(guard_file).exists(),
            "Runtime 04 guard/test file `{guard_file}` is missing; update asset_pipeline_boundary before changing guard ownership"
        );
    }

    let guard_sources = [
        include_str!("../../asset/tests/facade/handle_lifecycle.rs"),
        include_str!("../../asset/tests/facade/failure_reason.rs"),
        include_str!("../../asset/tests/facade/hot_reload.rs"),
        include_str!("../../asset/tests/pipeline/worker_pool.rs"),
        include_str!("../../asset/tests/watcher.rs"),
        include_str!("../../asset/tests/assets/artifact_store.rs"),
        include_str!("../../core/resource/tests.rs"),
        include_str!("asset_surface.rs"),
        include_str!("asset_worker_policy.rs"),
        include_str!("asset_pipeline.rs"),
        include_str!("plan_status/cargo_gates/early.rs"),
    ]
    .join("\n");
    let behavior_sources = [
        include_str!("../../asset/tests/facade/handle_lifecycle.rs"),
        include_str!("../../asset/tests/facade/failure_reason.rs"),
        include_str!("../../asset/tests/facade/hot_reload.rs"),
        include_str!("../../asset/tests/pipeline/worker_pool.rs"),
        include_str!("../../asset/tests/watcher.rs"),
        include_str!("../../asset/tests/assets/artifact_store.rs"),
        include_str!("../../core/resource/tests.rs"),
        include_str!("../../asset/facade/load_state.rs"),
    ]
    .join("\n");

    for guard_anchor in [
        "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
        "dangling_handle_queries_report_not_loaded_instead_of_panicking",
        "failed_asset_exposes_failure_reason_through_facade",
        "resource_state_rejects_error_to_ready_without_reloading",
        "resource_state_recovers_from_error_only_through_reloading",
        "resource_state_rejects_reload_failure_without_reload_boundary",
        "asset_load_state_projection_matches_resource_record_matrix",
        "worker_pool_unbounded_mode_is_explicit_opt_in",
        "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
        "concurrent_requests_for_same_asset_decode_once_and_notify_all",
        "worker_pool_diagnostics_track_in_flight_and_failure_counts",
        "worker_pool_frame_sampler_records_per_frame_completion_deltas",
        "project_asset_manager_spawns_worker_pool_with_frame_sampler",
        "rapid_successive_writes_within_debounce_window_emit_single_reload",
        "watcher_failure_on_removed_directory_surfaces_observable_error",
        "hot_reload_transitions_through_reloading_state_and_emits_modified_event",
        "reload_failure_emits_reload_failed_event_and_lands_failed_state",
        "artifact_store_roundtrips_scene_assets_with_mesh_references",
        "artifact_store_roundtrips_scene_assets_with_camera_targets",
        "artifact_store_roundtrips_scene_assets_with_physics_components",
        "artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
        "asset_worker_pool_matches_runtime_04_and_11_decisions",
        "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            guard_sources.contains(guard_anchor),
            "Runtime 04 guard anchor `{guard_anchor}` should stay visible to asset_pipeline_boundary"
        );
    }

    assert_eq!(
        EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS.len(),
        20,
        "Runtime 04 behavior-test anchor count should mirror asset_pipeline_boundary"
    );
    for behavior_anchor in EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS {
        assert!(
            behavior_sources.contains(behavior_anchor),
            "Runtime 04 behavior-test anchor `{behavior_anchor}` should stay visible to asset_pipeline_boundary"
        );
    }

    let mirror_docs = [
        (
            "Runtime 04 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "asset facade doc",
            include_str!("../../../../docs/zircon_runtime/asset/facade.md"),
        ),
        (
            "asset worker pool doc",
            include_str!("../../../../docs/zircon_runtime/asset/worker_pool.md"),
        ),
        (
            "asset watcher doc",
            include_str!("../../../../docs/zircon_runtime/asset/watcher.md"),
        ),
        (
            "asset artifact doc",
            include_str!("../../../../docs/zircon_runtime/asset/artifact.md"),
        ),
        (
            "core resource doc",
            include_str!("../../../../docs/zircon_runtime/core/resource.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "runtime-interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "asset_pipeline_boundary",
            "expected_source_file_count = 22",
            "expected_guard_file_count = 11",
            "worker_diagnostic_count = 7",
            "expected_worker_diagnostic_count = 7",
            "artifact_store_roundtrip_count = 4",
            "expected_artifact_store_roundtrip_count = 4",
            "watcher_acceptance_reference_count = 1",
            "expected_watcher_acceptance_count = 7",
            "artifact_acceptance_reference_count = 3",
            "test_anchor_count = 24",
            "behavior_test_anchor_count = 20",
            "missing_behavior_test_anchors = []",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "retired_worker_new_references = []",
            "old_watch_debounce_references = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 04 asset-pipeline audit anchor `{expected_anchor}`"
            );
        }
    }

    assert!(
        !workspace_root.join("zircon_asset/src/lib.rs").exists(),
        "Runtime 04 mirror guard assumes the standalone zircon_asset crate stays absorbed"
    );
}
