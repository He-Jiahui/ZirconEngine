use super::super::support::assert_contains_all;
use super::document_sources::RecentStaticGuardSources;

pub(super) fn assert_runtime_01_to_04_anchors(sources: &RecentStaticGuardSources) {
    let runtime_01_plan_anchors = [
        "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
        "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
        "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
        "plugin physics Cargo gates",
    ];
    let runtime_01_doc_anchors = [
        "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
        "runtime_text_doc_records_three_layer_stack_and_cross_reference",
        "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
        "export_archive_policy_allows_zip_only_for_archive_materializer",
        "editor_only_dependency_candidates_have_editor_backlog_owner",
    ];
    let runtime_02_plan_anchors = [
        "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        "core_spine_root_generated_boundary",
        "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "core/root/generated/export_build_plan/app/editor/plugin",
    ];
    let runtime_02_doc_anchors = [
        "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "generated_code_boundary.m1_gate_status",
        "classified-and-clear",
    ];
    let runtime_03_anchors = [
        "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
        "ecs_schedule/time/session/schedule_parallel",
        "RuntimeTimeAdvance",
        "fixed_step_plan_reports_overstep_fraction_in_unit_range",
        "ScheduleParallelExecutionReport",
    ];
    let runtime_04_anchors = [
        "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        "AssetWorkerPoolOptions",
        "asset_worker_pool_matches_runtime_04_and_11_decisions",
        "artifact_store_roundtrips_scene_assets_with",
        "watcher",
    ];

    assert_contains_all(
        "Runtime 01 subplan",
        sources.runtime_01_plan,
        &runtime_01_plan_anchors,
    );
    assert_contains_all(
        "Runtime 01 tech-stack doc",
        sources.runtime_01_tech_stack_doc,
        &[
            runtime_01_doc_anchors[0],
            runtime_01_doc_anchors[3],
            runtime_01_doc_anchors[4],
        ],
    );
    assert_contains_all(
        "Runtime 01 text doc",
        sources.runtime_01_text_doc,
        &[
            "Backend Responsibility Matrix",
            runtime_01_doc_anchors[1],
            "text_shaper_stack_uses_shared_text_service_for_font_backends",
        ],
    );
    assert_contains_all(
        "Runtime 01 physics doc",
        sources.runtime_01_physics_doc,
        &[
            runtime_01_doc_anchors[0],
            runtime_01_doc_anchors[2],
            "only executable V1 backend",
        ],
    );
    assert_contains_all(
        "Runtime 01 editor-only backlog doc",
        sources.runtime_01_editor_backlog_doc,
        &[runtime_01_doc_anchors[4], "rfd", "arboard"],
    );
    assert_contains_all(
        "Runtime 02 subplan",
        sources.runtime_02_plan,
        &runtime_02_plan_anchors,
    );
    assert_contains_all(
        "Runtime 02 root surface doc",
        sources.runtime_02_root_doc,
        &runtime_02_doc_anchors[..2],
    );
    assert_contains_all(
        "Runtime 02 generated boundary doc",
        sources.runtime_02_generated_doc,
        &runtime_02_doc_anchors[0..1],
    );
    assert_contains_all(
        "Runtime 02 generated boundary status doc",
        sources.runtime_02_generated_doc,
        &runtime_02_doc_anchors[2..],
    );
    assert_contains_all(
        "Runtime 03 subplan",
        sources.runtime_03_plan,
        &runtime_03_anchors,
    );
    assert_contains_all(
        "Runtime 03 frame schedule doc",
        sources.runtime_03_frame_doc,
        &runtime_03_anchors[2..],
    );
    assert_contains_all(
        "Runtime 03 schedule parallel doc",
        sources.runtime_03_parallel_doc,
        &[
            runtime_03_anchors[4],
            "schedule_parallel_batches_chain_through_job_handles",
        ],
    );
    assert_contains_all(
        "Runtime 04 subplan",
        sources.runtime_04_plan,
        &runtime_04_anchors,
    );
    assert_contains_all(
        "Runtime 04 asset facade doc",
        sources.runtime_04_facade_doc,
        &[
            "dangling_handle_queries_report_not_loaded_instead_of_panicking",
            "failed_asset_exposes_failure_reason_through_facade",
        ],
    );
    assert_contains_all(
        "Runtime 04 asset worker doc",
        sources.runtime_04_worker_doc,
        &runtime_04_anchors[1..3],
    );
    assert_contains_all(
        "Runtime 04 asset watcher doc",
        sources.runtime_04_watcher_doc,
        &["Asset Watcher", "Reloading", "watcher"],
    );
    assert_contains_all(
        "Runtime 04 asset artifact doc",
        sources.runtime_04_artifact_doc,
        &[runtime_04_anchors[3], "cache-wire boundary problem"],
    );
    assert_contains_all(
        "Runtime 04 resource doc",
        sources.runtime_04_resource_doc,
        &["ResourceRecord", "failure_reason", "Reloading"],
    );
}
