#[test]
fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2() {
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../docs/zircon_runtime/scene/ecs.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let schedule_runner = include_str!("../../scene/ecs/schedule_runner.rs");
    let query_tests = include_str!("../../scene/tests/ecs_performance_acceptance.rs");
    let change_tests = include_str!("../../scene/tests/ecs_change_detection.rs");
    let session_tests = include_str!("../../dynamic_api/session/tests.rs");
    for required_plan_anchor in [
        "M1 | 1.3 热点清单",
        "hotspot_inventory.md",
        "inventory_scaffold_static_passed_pending_authoritative_values",
        "无权威 runtime 数值不得进入 M2",
        "render 计划 02/04",
    ] {
        assert!(
            runtime_07_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "Runtime 07 plan/index should record hotspot inventory anchor `{required_plan_anchor}`"
        );
    }

    assert!(
        !runtime_07_plan.contains("热点清单 top3：__"),
        "Runtime 07 should not leave the M1.3 hotspot inventory placeholder untouched"
    );

    for required_doc_anchor in [
        "Evidence Gate",
        "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
        "Authoritative Top List",
        "Pending authoritative runtime sample",
        "Render-Plan Diversions",
        "vkCmdCopyBuffer",
        "Runtime 07 M2 is not allowed to fix render submission",
        "Candidate Evidence Matrix",
        "frame_extract_rebuild_skips_unchanged_entities",
        "query_state_reuses_archetype_matches_across_unchanged_frames",
        "change_detection_scan_skips_unmarked_archetypes",
        "asset.worker.budgeted_threads",
    ] {
        assert!(
            hotspot_doc.contains(required_doc_anchor),
            "hotspot inventory doc should keep evidence gate anchor `{required_doc_anchor}`"
        );
    }

    for required_query_anchor in [
        "const ENTITY_COUNT: usize = 128;",
        "const REPEATED_QUERY_RUNS: usize = 8;",
        "query_state_cache_stats_record_reuse_and_rebuild_counts",
        "query_state_reuses_archetype_matches_across_unchanged_frames",
        "assert_eq!(reused.cache_hits, REPEATED_QUERY_RUNS as u64)",
        "assert_eq!(reused.cache_misses, 1)",
        "assert_eq!(reused.cache_rebuilds, initial.cache_rebuilds)",
    ] {
        assert!(
            query_tests.contains(required_query_anchor),
            "QueryState performance evidence should retain `{required_query_anchor}`"
        );
    }

    for required_change_anchor in [
        "change_detection_scan_stats_record_mark_checks_and_diagnostics",
        "change_detection_scan_skips_unmarked_archetypes",
        "assert_eq!(stats.scanned_marks, unmarked.len() as u64 * 2)",
        "assert_eq!(stats.added_matches, 0)",
        "assert_eq!(stats.changed_matches, 0)",
    ] {
        assert!(
            change_tests.contains(required_change_anchor),
            "change-detection evidence should retain `{required_change_anchor}`"
        );
    }

    for required_extract_anchor in [
        "headless_session_capture_records_frame_extract_diagnostics",
        "frame_extract_rebuild_skips_unchanged_entities",
        "EXTRACT_REBUILD_CLONES_DIAGNOSTIC",
        "EXTRACT_OUTPUT_BYTES_DIAGNOSTIC",
        "rebuilds.history.iter().all(|sample| sample.value == 1.0)",
        "output_bytes.history[0].value, output_bytes.history[1].value",
    ] {
        assert!(
            session_tests.contains(required_extract_anchor),
            "extract evidence should retain `{required_extract_anchor}`"
        );
    }

    for required_schedule_span_anchor in [
        "profile_dynamic_scope!",
        "\"runtime\"",
        "\"frame\"",
        "runtime_frame_schedule_stage.{stage:?}",
    ] {
        assert!(
            schedule_runner.contains(required_schedule_span_anchor),
            "SceneScheduleRunner should keep Runtime 07 schedule-stage span anchor `{required_schedule_span_anchor}`"
        );
    }

    for required_schedule_doc_anchor in [
        "runtime_frame_schedule_stage",
        "SceneScheduleRunner",
        "stage-level span",
    ] {
        assert!(
            runtime_07_plan.contains(required_schedule_doc_anchor)
                || runtime_index.contains(required_schedule_doc_anchor)
                || hotspot_doc.contains(required_schedule_doc_anchor)
                || dynamic_session_doc.contains(required_schedule_doc_anchor)
                || ecs_doc.contains(required_schedule_doc_anchor)
                || architecture_review.contains(required_schedule_doc_anchor),
            "Runtime 07 schedule span docs should retain `{required_schedule_doc_anchor}`"
        );
    }

    for required_review_anchor in [
        "Runtime 07 Hotspot Inventory Guard",
        "zircon_runtime/src/scene/ecs/schedule_runner.rs",
        "runtime_frame_schedule_stage.<SystemStage>",
        "SceneScheduleRunner",
        "stage-level span",
    ] {
        assert!(
            architecture_review.contains(required_review_anchor),
            "runtime architecture review should retain Runtime 07 stage-span anchor `{required_review_anchor}`"
        );
    }

    for required_render_anchor in [
        "230 draws",
        "231 pre-draw",
        "31 render passes",
        "render 计划 02/04",
        "Runtime 07 M2 is not allowed to fix render submission",
    ] {
        assert!(
            runtime_07_plan.contains(required_render_anchor)
                || hotspot_doc.contains(required_render_anchor),
            "Runtime 07 plan/docs should retain render diversion anchor `{required_render_anchor}`"
        );
    }
}

#[test]
fn runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit() {
    let large_file_doc =
        include_str!("../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md");

    for required_large_file_doc_anchor in [
        "`hotspot_count = 41`",
        "`classification_count = 5`",
        "`decision_group_count = 5`",
        "`large_file_migration_debt_count = 5`",
        "`unclassified_hotspot_count = 0`",
        "`runtime-framework-render = 2`",
        "`runtime-other = 17`",
        "`support-hub = 3`",
        "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs",
        "zircon_runtime/src/core/framework/render/backend_types.rs",
        "zircon_hub/src/tauri_app/runtime_state/project_actions.rs",
        "zircon_hub/src/tauri_app/view_model.rs",
        "zircon_hub/src/tauri_app/runtime_state.rs",
    ] {
        assert!(
            large_file_doc.contains(required_large_file_doc_anchor),
            "large-file owner gate doc should retain current audit anchor `{required_large_file_doc_anchor}`"
        );
    }

    for stale_large_file_doc_anchor in [
        "zircon_hub/src/app/runtime.rs",
        "zircon_hub/src/app/view_model.rs",
        "`hotspot_count = 33`",
        "`runtime-framework-render = 1`",
        "`runtime-other = 10`",
    ] {
        assert!(
            !large_file_doc.contains(stale_large_file_doc_anchor),
            "large-file owner gate doc should not keep stale audit anchor `{stale_large_file_doc_anchor}`"
        );
    }

    for required_runtime_07_owner_gate_anchor in [
        "Runtime 07 owner-budgeted optimization gate",
        "large_file_ownership_gate",
        "migration-debt-present",
        "hotspots 41",
        "debt groups 5",
        "owner classes 5",
        "unclassified 0",
    ] {
        assert!(
            runtime_07_plan.contains(required_runtime_07_owner_gate_anchor)
                || runtime_index.contains(required_runtime_07_owner_gate_anchor)
                || hotspot_doc.contains(required_runtime_07_owner_gate_anchor)
                || architecture_review.contains(required_runtime_07_owner_gate_anchor)
                || interface_doc.contains(required_runtime_07_owner_gate_anchor),
            "Runtime 07 owner-budget gate mirrors should retain `{required_runtime_07_owner_gate_anchor}`"
        );
    }

    for required_mirror_anchor in [
        "hotspots 41, debt groups 5, owner classes 5, unclassified hotspots 0",
        "41 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots",
        "`editor-retained-host=11`, `editor-ui=8`, `runtime-framework-render=2`, `runtime-other=17`, and `support-hub=3`",
        "threshold 1000 lines, 41 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots",
    ] {
        assert!(
            runtime_07_plan.contains(required_mirror_anchor)
                || runtime_index.contains(required_mirror_anchor)
                || hotspot_doc.contains(required_mirror_anchor)
                || architecture_review.contains(required_mirror_anchor)
                || interface_doc.contains(required_mirror_anchor),
            "Runtime 07 mirror docs should retain exact large-file gate summary `{required_mirror_anchor}`"
        );
    }
}
