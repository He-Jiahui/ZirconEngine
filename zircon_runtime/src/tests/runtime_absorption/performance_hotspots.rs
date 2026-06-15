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
    let session_tests = include_str!("../../dynamic_api/session/tests/frame_diagnostics.rs");
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
        "`hotspot_count = 39`",
        "`classification_count = 5`",
        "`decision_group_count = 5`",
        "`large_file_migration_debt_count = 5`",
        "`unclassified_hotspot_count = 0`",
        "`editor-retained-host = 12`",
        "`runtime-framework-render = 4`",
        "`runtime-other = 12`",
        "`support-hub = 3`",
        "zircon_runtime/src/asset/assets/scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs",
        "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs",
        "zircon_runtime/src/core/framework/render/backend_types.rs",
        "zircon_runtime/src/core/framework/render/post_process/stack.rs",
        "zircon_runtime/src/core/framework/render/post_process/volume_component.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs",
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
        "`hotspot_count = 42`",
        "`hotspot_count = 41`",
        "`hotspot_count = 38`",
        "`hotspot_count = 37`",
        "`editor-retained-host = 11`",
        "`runtime-framework-render = 1`",
        "`runtime-framework-render = 2`",
        "`runtime-framework-render = 3`",
        "`runtime-other = 10`",
        "`runtime-other = 18`",
        "`runtime-other = 17`",
        "`runtime-other = 14`",
        "`runtime-other = 13`",
        "zircon_runtime/src/asset/assets/scene.rs",
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
        "hotspots 39",
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
        "hotspots 39, debt groups 5, owner classes 5, unclassified hotspots 0",
        "39 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots",
        "`editor-retained-host=12`, `editor-ui=8`, `runtime-framework-render=4`, `runtime-other=12`, and `support-hub=3`",
        "threshold 1000 lines, 39 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots",
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

#[test]
fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts() {
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../docs/zircon_runtime/scene/ecs.md");
    let interface_doc =
        include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let audit_script = include_str!(
        "../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py"
    );
    let performance_guard = include_str!("performance_hotspots.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/early.rs");

    for guard_anchor in [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
        "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
        "runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
        "runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
    ] {
        assert!(
            performance_guard.contains(guard_anchor) || cargo_gate_guard.contains(guard_anchor),
            "Runtime 07 guard anchor `{guard_anchor}` should stay visible to performance_hotpath_boundary"
        );
    }

    for audit_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 10",
        "EXPECTED_TEST_FILE_COUNT = 5",
        "MIRROR_DOCS_GUARD",
        "\"runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts\"",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(
            audit_script.contains(audit_anchor),
            "performance_hotpath_boundary should expose audit anchor `{audit_anchor}`"
        );
    }

    let mirror_docs = [
        ("Runtime 07 plan", runtime_07_plan),
        ("runtime index", runtime_index),
        ("hotspot inventory doc", hotspot_doc),
        ("dynamic session doc", dynamic_session_doc),
        ("ECS doc", ecs_doc),
        ("runtime interface convergence doc", interface_doc),
        ("runtime architecture review", architecture_review),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "performance_hotpath_boundary",
            "expected_source_file_count = 10",
            "expected_test_file_count = 5",
            "frame_span_anchor_count = 9",
            "query_counter_anchor_count = 13",
            "change_counter_anchor_count = 9",
            "extract_counter_anchor_count = 10",
            "asset_worker_anchor_count = 5",
            "hotspot_guard_anchor_count = 20",
            "test_anchor_count = 12",
            "doc_anchor_count = 17",
            "cargo_gate_anchor_count = 5",
            "stale_hotspot_placeholder_present = false",
            "large_file_m1_gate_status = migration-debt-present",
            "large_file_hotspot_count = 39",
            "large_file_migration_debt_count = 5",
            "large_file_owner_class_count = 5",
            "large_file_unclassified_hotspot_count = 0",
            "missing_large_file_owner_classes = []",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 07 performance-hotpath audit anchor `{expected_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner() {
    fn occurrence_count(source: &str, needle: &str) -> usize {
        source.matches(needle).count()
    }

    let scene_mod = include_str!("../../asset/assets/scene/mod.rs");
    let scene_lighting = include_str!("../../asset/assets/scene/lighting.rs");
    let scene_physics = include_str!("../../asset/assets/scene/physics.rs");
    let asset_assets_mod = include_str!("../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../asset/mod.rs");
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let scene_doc = include_str!("../../../../docs/zircon_runtime/asset/assets/scene.md");

    for module_decl in [
        "mod animation;",
        "mod asset;",
        "mod camera;",
        "mod defaults;",
        "mod entity;",
        "mod extensions;",
        "mod lighting;",
        "mod management;",
        "mod mesh;",
        "mod physics;",
        "mod post_process;",
        "mod transform;",
    ] {
        assert!(
            scene_mod.contains(module_decl),
            "scene/mod.rs should keep folder-backed declaration `{module_decl}`"
        );
    }

    assert_eq!(
        occurrence_count(scene_mod, "pub enum SceneMobilityAsset"),
        1,
        "scene/mod.rs should be the only SceneMobilityAsset enum owner"
    );
    assert!(
        !scene_physics.contains("SceneMobilityAsset"),
        "scene/physics.rs should not reintroduce a duplicate SceneMobilityAsset owner"
    );

    for export_anchor in [
        "pub use lighting::{",
        "SceneSpotLightAsset",
        "pub use scene::{",
        "SceneMobilityAsset",
    ] {
        assert!(
            scene_mod.contains(export_anchor)
                || asset_assets_mod.contains(export_anchor)
                || asset_mod.contains(export_anchor),
            "scene asset export chain should retain `{export_anchor}`"
        );
    }

    for spot_light_anchor in [
        "pub struct SceneSpotLightAsset",
        "pub direction: [Real; 3]",
        "pub outer_angle_radians: Real",
    ] {
        assert!(
            scene_lighting.contains(spot_light_anchor),
            "SceneSpotLightAsset should retain public field anchor `{spot_light_anchor}`"
        );
    }

    for doc_anchor in [
        "scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs",
        "SceneMobilityAsset",
        "SceneSpotLightAsset",
        "split-drift repair",
        "split_drift_static_passed_cargo_deferred_active_lanes",
        "scene asset split-drift repair",
    ] {
        assert!(
            runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || scene_doc.contains(doc_anchor),
            "Runtime 07 scene split docs should retain `{doc_anchor}`"
        );
    }
}

#[test]
fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners() {
    let project_io_root = include_str!("../../scene/world/project_io.rs");
    let camera = include_str!("../../scene/world/project_io/camera.rs");
    let physics = include_str!("../../scene/world/project_io/physics.rs");
    let post_process = include_str!("../../scene/world/project_io/post_process.rs");
    let references = include_str!("../../scene/world/project_io/references.rs");
    let script = include_str!("../../scene/world/project_io/script.rs");
    let transform = include_str!("../../scene/world/project_io/transform.rs");
    let project_io_doc = include_str!("../../../../docs/zircon_runtime/scene/world/project_io.md");
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let large_file_doc =
        include_str!("../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");

    for root_anchor in [
        "mod camera;",
        "mod physics;",
        "mod post_process;",
        "mod references;",
        "mod script;",
        "mod transform;",
        "pub fn from_scene_asset",
        "pub fn to_scene_asset",
    ] {
        assert!(
            project_io_root.contains(root_anchor),
            "project_io.rs should keep entry orchestration anchor `{root_anchor}`"
        );
    }

    for moved_helper in [
        "fn camera_target_from_asset",
        "fn collider_shape_from_asset",
        "fn post_process_settings_from_asset",
        "fn model_handle_for_reference",
        "fn script_bindings_for_record",
        "fn transform_from_asset",
    ] {
        assert!(
            !project_io_root.contains(moved_helper),
            "project_io.rs should not reclaim converter helper `{moved_helper}`"
        );
    }

    for (module_name, module_source, expected_anchor) in [
        ("camera", camera, "pub(super) fn camera_to_asset"),
        ("physics", physics, "pub(super) fn collider_shape_to_asset"),
        (
            "post_process",
            post_process,
            "pub(super) fn post_process_volume_to_asset",
        ),
        (
            "references",
            references,
            "pub(super) fn reference_for_model_handle",
        ),
        ("script", script, "pub(super) fn script_bindings_for_record"),
        ("transform", transform, "pub(super) fn transform_to_asset"),
    ] {
        assert!(
            module_source.contains(expected_anchor),
            "project_io/{module_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "Project I/O Folder Split",
        "project_io/{camera,physics,post_process,references,script,transform}.rs",
        "large_file_hotspot_count = 39",
        "runtime-other = 12",
        "project_io.rs 772 行",
        "project_io folder split",
    ] {
        assert!(
            project_io_doc.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || large_file_doc.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor),
            "Project I/O split docs should retain `{doc_anchor}`"
        );
    }
}

#[test]
fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner() {
    let session_root = include_str!("../../dynamic_api/session.rs");
    let session_events = include_str!("../../dynamic_api/session/events.rs");
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");

    for root_anchor in [
        "mod events;",
        "pub(super) unsafe fn handle_event(",
        "with_session(handle, |session| session.handle_event(event))",
    ] {
        assert!(
            session_root.contains(root_anchor),
            "session.rs should keep dynamic ABI event entry anchor `{root_anchor}`"
        );
    }

    for moved_event_anchor in [
        "fn handle_mouse_button",
        "fn handle_mouse_wheel",
        "fn handle_keyboard",
        "fn handle_ime",
        "fn handle_gamepad_axis",
        "fn sync_orbit_target_from_selection",
    ] {
        assert!(
            !session_root.contains(moved_event_anchor),
            "session.rs should not reclaim dynamic event helper `{moved_event_anchor}`"
        );
        assert!(
            session_events.contains(moved_event_anchor),
            "session/events.rs should own dynamic event helper `{moved_event_anchor}`"
        );
    }

    for events_anchor in [
        "pub(super) fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus",
        "UiAccessibilityActionRequest",
        "runtime_session_menu_action_at",
        "write_runtime_menu_action",
        "ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1",
    ] {
        assert!(
            session_events.contains(events_anchor),
            "session/events.rs should retain dynamic event dispatch anchor `{events_anchor}`"
        );
    }

    for doc_anchor in [
        "Dynamic Session Event Split",
        "session/events.rs",
        "large_file_hotspot_count = 39",
        "runtime-other = 12",
        "dynamic session event split",
    ] {
        assert!(
            runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor),
            "Dynamic session event split docs should retain `{doc_anchor}`"
        );
    }
}
