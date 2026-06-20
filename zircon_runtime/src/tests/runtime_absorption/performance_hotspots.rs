#[path = "performance_hotspots/owner_budget.rs"]
mod owner_budget;

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
    let animation_doc = include_str!("../../../../docs/zircon_runtime/animation/runtime.md");
    let diagnostics_doc = include_str!("../../../../docs/zircon_runtime/core/diagnostics.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let schedule_runner = include_str!("../../scene/ecs/schedule_runner.rs");
    let ecs_frame_diagnostics = include_str!("../../scene/ecs/frame_performance_diagnostics.rs");
    let query_filter = include_str!("../../scene/ecs/query/query_filter.rs");
    let query_iter = include_str!("../../scene/ecs/query/query_iter.rs");
    let query_many_iter = include_str!("../../scene/ecs/query/query_many_iter.rs");
    let query_state_root = include_str!("../../scene/ecs/query/query_state/mod.rs");
    let query_state_cache = include_str!("../../scene/ecs/query/query_state/cache.rs");
    let query_state_read_only_cached =
        include_str!("../../scene/ecs/query/query_state/read_only_cached.rs");
    let query_state_stats = include_str!("../../scene/ecs/query/query_state/stats.rs");
    let query_state_system_param =
        include_str!("../../scene/ecs/query/query_state/system_param.rs");
    let system_param = include_str!("../../scene/ecs/system/system_param.rs");
    let system_state = include_str!("../../scene/ecs/system/system_state.rs");
    let param_set = include_str!("../../scene/ecs/system/param_set.rs");
    let world_performance_diagnostics =
        include_str!("../../scene/world/performance_diagnostics.rs");
    let world_driver = include_str!("../../scene/module/world_driver.rs");
    let query_tests = include_str!("../../scene/tests/ecs_performance_acceptance.rs");
    let change_tests = include_str!("../../scene/tests/ecs_change_detection.rs");
    let session_tests = include_str!("../../dynamic_api/session/tests/frame_diagnostics.rs");
    let session_extract_cache = include_str!("../../dynamic_api/session/extract_cache.rs");
    let session_extract_stats = include_str!("../../dynamic_api/session/extract_stats.rs");
    let asset_worker_source = include_str!("../../asset/pipeline/worker_pool.rs");
    let asset_worker_manager =
        include_str!("../../asset/pipeline/manager/project_asset_manager/construction.rs");
    let asset_worker_tests = include_str!("../../asset/tests/pipeline/worker_pool.rs");
    let animation_scene_diagnostics = include_str!("../../animation/scene_hook/diagnostics.rs");
    let animation_scene_events = include_str!("../../animation/scene_hook/events.rs");
    let animation_scene_node_pose = include_str!("../../animation/scene_hook/node_pose.rs");
    let animation_scene_pending = include_str!("../../animation/scene_hook/pending.rs");
    let animation_scene_scan = include_str!("../../animation/scene_hook/scan.rs");
    let animation_scene_tick = include_str!("../../animation/scene_hook/tick.rs");
    let root_manifest = include_str!("../../../../Cargo.toml");
    let runtime_manifest = include_str!("../../../../zircon_runtime/Cargo.toml");
    let zircon_build = include_str!("../../../../tools/zircon_build.py");
    let dev_fast_build = include_str!("../../../../tools/dev-fast-build.ps1");
    let build_tool_doc = include_str!("../../../../docs/cli-and-tooling/zircon-build-tool.md");
    let profiling_doc =
        include_str!("../../../../docs/zircon_runtime/core/diagnostics/profiling.md");
    let interface_profiling = include_str!("../../../../zircon_runtime_interface/src/profiling.rs");
    let profiling_counter_hotspot =
        include_str!("../../core/runtime/diagnostics/profiling/counter_hotspot.rs");
    let profiling_export = include_str!("../../core/runtime/diagnostics/profiling/export.rs");
    let profiling_mod = include_str!("../../core/runtime/diagnostics/profiling/mod.rs");
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
        "AnimationSceneFrameDiagnostics",
        "animation.scene.scanned_entities",
        "animation.scene.output_poses",
        "animation_scene_frame_diagnostics_static_passed_cargo_deferred",
        "CounterHotspotReport",
        "counter_hotspots.json",
    ] {
        assert!(
            hotspot_doc.contains(required_doc_anchor)
                || animation_doc.contains(required_doc_anchor)
                || diagnostics_doc.contains(required_doc_anchor),
            "Runtime 07 docs should keep evidence gate anchor `{required_doc_anchor}`"
        );
    }

    for required_counter_hotspot_anchor in [
        "PROFILE_COUNTER_HOTSPOTS_FILE",
        "pub struct CounterHotspotReport",
        "pub struct CounterHotspotEntry",
        "pub fn analyze_counter_hotspots",
        "counter_hotspots.json",
        "ProfileControlResponse.counter_hotspot_report",
        "summary.push_str(\"\\n## Counter Hotspots\\n\");",
        "response.counter_hotspot_report = Some(report.counter_hotspots);",
    ] {
        assert!(
            interface_profiling.contains(required_counter_hotspot_anchor)
                || profiling_counter_hotspot.contains(required_counter_hotspot_anchor)
                || profiling_export.contains(required_counter_hotspot_anchor)
                || profiling_mod.contains(required_counter_hotspot_anchor)
                || hotspot_doc.contains(required_counter_hotspot_anchor)
                || profiling_doc.contains(required_counter_hotspot_anchor),
            "Runtime 07 generic profiling counter hotspot export should retain `{required_counter_hotspot_anchor}`"
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
        "EXTRACT_CACHE_HITS_DIAGNOSTIC",
        "EXTRACT_CACHE_MISSES_DIAGNOSTIC",
        "rebuilds.history[1].value, 0.0",
        "cache_hits.history[1].value, 1.0",
        "cache_misses.history[0].value, 1.0",
        "unchanged headless capture should reuse the cached extract",
        "frame_extract_rebuilds_after_scene_change",
        "scene mutations should invalidate the dynamic-session extract cache",
        "output_bytes.history[0].value, output_bytes.history[1].value",
    ] {
        assert!(
            session_tests.contains(required_extract_anchor),
            "extract evidence should retain `{required_extract_anchor}`"
        );
    }

    for required_extract_cache_anchor in [
        "pub(super) struct RuntimeFrameExtractCache",
        "struct RuntimeFrameExtractCacheKey",
        "change_tick: world.read_change_tick()",
        "query_cache_revision: world.query_cache_revision()",
        "active_camera: world.active_camera()",
        "RuntimeFrameExtractCacheStatus::Rebuilt => 1",
        "RuntimeFrameExtractCacheStatus::Reused => 0",
    ] {
        assert!(
            session_extract_cache.contains(required_extract_cache_anchor)
                || session_extract_stats.contains(required_extract_cache_anchor),
            "extract cache path should retain `{required_extract_cache_anchor}`"
        );
    }

    for required_asset_worker_anchor in [
        "ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC",
        "ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC",
        "pub struct AssetWorkerPoolFrameSampler",
        "pub fn spawn_worker_pool_with_frame_sampler(",
        "AssetWorkerPoolFrameSampler::from_pool(&pool)",
        "pub completed_delta: u64",
        "pub fn sample(&mut self, pool: &AssetWorkerPool) -> AssetWorkerPoolFrameDiagnostics",
        "worker_pool_frame_sampler_records_per_frame_completion_deltas",
        "asset.worker.frame_completed",
    ] {
        assert!(
            asset_worker_source.contains(required_asset_worker_anchor)
                || asset_worker_manager.contains(required_asset_worker_anchor)
                || asset_worker_tests.contains(required_asset_worker_anchor),
            "asset worker frame diagnostics should retain `{required_asset_worker_anchor}`"
        );
    }

    for required_animation_scene_anchor in [
        "ANIMATION_SCENE_SCANNED_ENTITIES_DIAGNOSTIC",
        "animation.scene.scanned_entities",
        "animation.scene.sequence_samples",
        "animation.scene.output_poses",
        "animation.scene.applied_transforms",
        "animation.scene.published_events",
        "animation.scene.state_transitions",
        "pub(super) struct AnimationSceneFrameDiagnostics",
        "pub(super) fn from_scan(scan: &AnimationSceneScan) -> Self",
        "pub(super) fn record(self, core: &CoreHandle)",
        "scanned_entities: entity_ids.len()",
        "let event_count = events.len();",
        "let update_count = updates.len();",
        "AnimationSceneFrameDiagnostics::from_scan(&scan)",
        "frame_diagnostics.published_events += publish_events(level, graph_events);",
        "frame_diagnostics.applied_transforms =",
        "frame_diagnostics.state_transitions = transition_updates.len();",
        "AnimationSceneFrameDiagnostics::default().record(core);",
    ] {
        assert!(
            animation_scene_diagnostics.contains(required_animation_scene_anchor)
                || animation_scene_events.contains(required_animation_scene_anchor)
                || animation_scene_node_pose.contains(required_animation_scene_anchor)
                || animation_scene_pending.contains(required_animation_scene_anchor)
                || animation_scene_scan.contains(required_animation_scene_anchor)
                || animation_scene_tick.contains(required_animation_scene_anchor),
            "animation scene diagnostics should retain `{required_animation_scene_anchor}`"
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

    for required_ecs_frame_diagnostic_anchor in [
        "pub struct EcsFramePerformanceDiagnostics",
        "pub fn add_query_stats(&mut self, stats: QueryStateCacheStats)",
        "pub fn add_change_detection_stats(&mut self, stats: ChangeDetectionScanStats)",
        "pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64)",
        "ecs_frame_performance_diagnostics_record_query_and_change_counts",
        "last_reported_cache_stats",
        "change_detection_stats: Cell<ChangeDetectionScanStats>",
        "last_reported_change_detection_stats",
        "pub fn saturating_delta_since(self, baseline: Self) -> Self",
        "pub(crate) fn take_unreported_cache_stats(&mut self) -> QueryStateCacheStats",
        "pub(crate) fn take_unreported_change_detection_stats(&mut self) -> ChangeDetectionScanStats",
        "fn record_performance_diagnostics(world: &mut World, state: &mut Self::State)",
        "world.record_ecs_query_cache_stats(query_stats);",
        "world.record_ecs_change_detection_stats(change_detection_stats);",
        "state.record_change_detection_stats(self.change_detection_stats);",
        "state: Option<NonNull<QueryState<D, F>>>",
        "P::record_performance_diagnostics(world, &mut self.state);",
        "P::record_performance_diagnostics(world, state);",
        "A::record_performance_diagnostics(world, &mut state.0);",
        "pub fn reset_ecs_frame_performance_diagnostics(&mut self)",
        "pub fn ecs_frame_performance_diagnostics(&self) -> EcsFramePerformanceDiagnostics",
        "pub(crate) fn record_ecs_query_cache_stats(&mut self, stats: QueryStateCacheStats)",
        "pub(crate) fn record_ecs_change_detection_stats(&mut self, stats: ChangeDetectionScanStats)",
        "level.with_world_mut(|world| world.reset_ecs_frame_performance_diagnostics());",
        "system_state_records_query_cache_stats_into_world_frame_diagnostics",
        "system_state_records_change_detection_stats_into_world_frame_diagnostics",
        "matches_component_locations_with_stats",
        "stats.scan_changed(component_ticks, ticks)",
        "stats.scan_added(component_ticks, ticks)",
    ] {
        assert!(
            ecs_frame_diagnostics.contains(required_ecs_frame_diagnostic_anchor)
                || query_filter.contains(required_ecs_frame_diagnostic_anchor)
                || query_iter.contains(required_ecs_frame_diagnostic_anchor)
                || query_many_iter.contains(required_ecs_frame_diagnostic_anchor)
                || query_state_root.contains(required_ecs_frame_diagnostic_anchor)
                || query_state_cache.contains(required_ecs_frame_diagnostic_anchor)
                || query_state_read_only_cached.contains(required_ecs_frame_diagnostic_anchor)
                || query_state_stats.contains(required_ecs_frame_diagnostic_anchor)
                || query_state_system_param.contains(required_ecs_frame_diagnostic_anchor)
                || system_param.contains(required_ecs_frame_diagnostic_anchor)
                || system_state.contains(required_ecs_frame_diagnostic_anchor)
                || param_set.contains(required_ecs_frame_diagnostic_anchor)
                || world_performance_diagnostics.contains(required_ecs_frame_diagnostic_anchor)
                || world_driver.contains(required_ecs_frame_diagnostic_anchor)
                || query_tests.contains(required_ecs_frame_diagnostic_anchor),
            "ECS frame performance diagnostic aggregation should retain `{required_ecs_frame_diagnostic_anchor}`"
        );
    }

    for required_profiling_build_anchor in [
        "#### 切片 0.2 profiling 构建超时破解",
        "profile.profiling",
        "profiling-tracy",
        "profiling-chrome",
        "python tools/zircon_build.py --targets runtime",
        "./tools/dev-fast-build.ps1 -Profile client -Action check",
        "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
    ] {
        assert!(
            runtime_07_plan.contains(required_profiling_build_anchor)
                || runtime_index.contains(required_profiling_build_anchor)
                || hotspot_doc.contains(required_profiling_build_anchor)
                || build_tool_doc.contains(required_profiling_build_anchor)
                || profiling_doc.contains(required_profiling_build_anchor),
            "Runtime 07 profiling build tooling should retain `{required_profiling_build_anchor}`"
        );
    }

    for required_cargo_profile_anchor in [
        "[profile.profiling]",
        "inherits = \"release\"",
        "debug = true",
        "strip = false",
    ] {
        assert!(
            root_manifest.contains(required_cargo_profile_anchor),
            "root Cargo.toml should retain profiling profile anchor `{required_cargo_profile_anchor}`"
        );
    }

    for required_runtime_feature_anchor in [
        "profiling = []",
        "profiling-chrome = [\"profiling\"]",
        "profiling-tracy = [\"profiling\", \"dep:tracing-subscriber\", \"dep:tracing-tracy\"]",
        "profiling-memory = [\"profiling\"]",
    ] {
        assert!(
            runtime_manifest.contains(required_runtime_feature_anchor),
            "zircon_runtime Cargo.toml should retain profiling feature anchor `{required_runtime_feature_anchor}`"
        );
    }

    for required_zircon_build_anchor in [
        "MODES = (\"debug\", \"release\", \"profiling\")",
        "TARGET_FEATURES = (\"target-client\", \"target-server\", \"target-editor-host\")",
        "def feature_arg_for_target(self, target_feature: str) -> str:",
        "parser.add_argument(",
        "--runtime-features",
        "--mode profiling is not supported for the hub/Tauri target.",
        "command.extend([\"--profile\", \"profiling\"])",
        "python tools/zircon_build.py --targets runtime --out E:\\builds\\zircon-smoke --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run",
    ] {
        assert!(
            zircon_build.contains(required_zircon_build_anchor)
                || build_tool_doc.contains(required_zircon_build_anchor),
            "tools/zircon_build.py profiling path should retain `{required_zircon_build_anchor}`"
        );
    }

    for required_dev_fast_build_anchor in [
        "[ValidateSet(\"debug\", \"release\", \"profiling\")]",
        "[string]$CargoProfile = \"debug\"",
        "$CargoProfile -eq \"profiling\"",
        "$args.Add(\"--profile\")",
        "$args.Add(\"profiling\")",
        "./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride \"target-client profiling profiling-tracy\"",
    ] {
        assert!(
            dev_fast_build.contains(required_dev_fast_build_anchor)
                || build_tool_doc.contains(required_dev_fast_build_anchor)
                || profiling_doc.contains(required_dev_fast_build_anchor),
            "tools/dev-fast-build.ps1 profiling path should retain `{required_dev_fast_build_anchor}`"
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
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
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
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
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

#[test]
fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed() {
    let cache_root = include_str!("../../asset/artifact/cache_payload.rs");
    let cache_json = include_str!("../../asset/artifact/cache_payload/json_value.rs");
    let cache_mesh = include_str!("../../asset/artifact/cache_payload/mesh.rs");
    let cache_scene = include_str!("../../asset/artifact/cache_payload/scene.rs");
    let cache_toml = include_str!("../../asset/artifact/cache_payload/toml_value.rs");
    let runtime_04_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let artifact_doc = include_str!("../../../../docs/zircon_runtime/asset/artifact.md");

    for root_anchor in [
        "mod json_value;",
        "mod mesh;",
        "mod scene;",
        "mod toml_value;",
        "pub(super) enum ArtifactCacheAsset",
        "Data(ArtifactCacheDataAsset)",
        "Mesh(ArtifactCacheMeshAsset)",
    ] {
        assert!(
            cache_root.contains(root_anchor),
            "cache_payload.rs should retain artifact wire dispatcher anchor `{root_anchor}`"
        );
    }

    for moved_owner_anchor in [
        "enum ArtifactCacheJsonValue",
        "enum ArtifactCacheTomlValue",
        "struct ArtifactCacheMeshAsset",
        "enum ArtifactCacheMeshAttributeValues",
        "enum ArtifactCacheMeshIndices",
    ] {
        assert!(
            !cache_root.contains(moved_owner_anchor),
            "cache_payload.rs should not reclaim moved cache wire owner `{moved_owner_anchor}`"
        );
    }

    for json_anchor in [
        "pub(super) enum ArtifactCacheJsonValue",
        "pub(super) fn from_json",
        "pub(super) fn into_json",
        "pub(super) fn json_table_to_cache",
        "pub(super) fn cache_table_to_json",
    ] {
        assert!(
            cache_json.contains(json_anchor),
            "cache_payload/json_value.rs should own JSON wire anchor `{json_anchor}`"
        );
    }

    for mesh_anchor in [
        "pub(super) struct ArtifactCacheMeshAsset",
        "pub(super) fn into_asset",
        "enum ArtifactCacheMeshAttributeValues",
        "enum ArtifactCacheMeshIndices",
        "fn mesh_attribute_table_to_cache",
    ] {
        assert!(
            cache_mesh.contains(mesh_anchor),
            "cache_payload/mesh.rs should own mesh wire anchor `{mesh_anchor}`"
        );
    }

    for toml_anchor in [
        "pub(super) type ArtifactCacheTomlTable",
        "pub(super) enum ArtifactCacheTomlValue",
        "pub(super) fn from_toml",
        "pub(super) fn into_toml",
        "pub(super) fn toml_table_like_to_cache",
        "pub(super) fn cache_table_like_to_toml",
    ] {
        assert!(
            cache_toml.contains(toml_anchor),
            "cache_payload/toml_value.rs should own TOML wire anchor `{toml_anchor}`"
        );
    }

    assert!(
        cache_scene.contains(
            "use super::json_value::{cache_table_to_json, json_table_to_cache, ArtifactCacheJsonValue};"
        ),
        "scene cache wire owner should consume JSON conversion through cache_payload/json_value.rs"
    );

    for doc_anchor in [
        "artifact_cache_payload_owner_split_static_passed_cargo_deferred",
        "cache_payload/{json_value,mesh,toml_value}.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "expected_source_file_count = 22",
    ] {
        assert!(
            runtime_04_plan.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || artifact_doc.contains(doc_anchor),
            "artifact cache payload owner split docs should retain `{doc_anchor}`"
        );
    }
}

#[test]
fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed() {
    let product_root = include_str!("../../core/runtime/diagnostics/render_stats_store/product.rs");
    let camera =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/camera.rs");
    let visibility =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/visibility.rs");
    let hzb = include_str!("../../core/runtime/diagnostics/render_stats_store/product/hzb.rs");
    let light_grid =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/light_grid.rs");
    let effect_stack =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/effect_stack.rs");
    let material =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/material.rs");
    let light = include_str!("../../core/runtime/diagnostics/render_stats_store/product/light.rs");
    let mesh_queue =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs");
    let gpu_scene =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs");
    let sprite =
        include_str!("../../core/runtime/diagnostics/render_stats_store/product/sprite.rs");
    let ui = include_str!("../../core/runtime/diagnostics/render_stats_store/product/ui.rs");
    let diagnostics_doc = include_str!("../../../../docs/zircon_runtime/core/diagnostics.md");
    let runtime_07_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let large_file_doc =
        include_str!("../../../../docs/engine-architecture/large-file-ownership-m1.md");

    for module_decl in [
        "mod camera;",
        "mod visibility;",
        "mod hzb;",
        "mod light_grid;",
        "mod effect_stack;",
        "mod material;",
        "mod light;",
        "mod mesh_queue;",
        "mod gpu_scene;",
        "mod sprite;",
        "mod ui;",
    ] {
        assert!(
            product_root.contains(module_decl),
            "product.rs should keep render product diagnostic child declaration `{module_decl}`"
        );
    }

    for root_dispatch_anchor in [
        "camera::record(store, stats);",
        "visibility::record(store, stats);",
        "hzb::record(store, stats);",
        "light_grid::record(store, stats);",
        "material::record(store, stats);",
        "light::record(store, stats);",
        "mesh_queue::record(store, stats);",
        "gpu_scene::record(store, stats);",
        "sprite::record(store, stats);",
        "effect_stack::record(store, stats);",
        "ui::record(store, stats);",
    ] {
        assert!(
            product_root.contains(root_dispatch_anchor),
            "product.rs should keep only render product dispatch anchor `{root_dispatch_anchor}`"
        );
    }

    for moved_owner_anchor in [
        "fn record_camera",
        "fn record_visibility",
        "fn record_hzb",
        "fn record_light_grid",
        "fn record_light_family",
        "fn record_mesh_queue",
        "fn record_gpu_scene",
        "fn record_sprite",
        "fn record_ui",
    ] {
        assert!(
            !product_root.contains(moved_owner_anchor),
            "product.rs should not reclaim moved render product owner `{moved_owner_anchor}`"
        );
    }

    for (module_name, module_source, expected_anchor) in [
        (
            "camera",
            camera,
            "render.camera.target.graph_import.ready_for_direct_import",
        ),
        (
            "visibility",
            visibility,
            "render.visibility.static_index.main_view_static_candidate_count",
        ),
        ("hzb", hzb, "render.hzb.occlusion.remaining_instance_count"),
        ("light_grid", light_grid, "render.light_grid.tile_count"),
        (
            "effect_stack",
            effect_stack,
            "render.post_process.motion_vector.camera.ready",
        ),
        ("material", material, "render.material.ready_count"),
        ("light", light, "render.light.directional.ready_count"),
        (
            "mesh_queue",
            mesh_queue,
            "render.mesh.queue.cached_command_hit_count",
        ),
        ("gpu_scene", gpu_scene, "render.gpu_scene.uploaded_bytes"),
        (
            "sprite",
            sprite,
            "render.sprite.queue.transparent_draw_batch_count",
        ),
        ("ui", ui, "render.ui.graph_executed_pass_count"),
    ] {
        assert!(
            module_source.contains("pub(super) fn record")
                && module_source.contains(expected_anchor),
            "render_stats_store/product/{module_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "render_product_diagnostics_owner_split_static_passed_cargo_deferred",
        "render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs",
        "render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs",
        "large_file_hotspot_count = 39",
        "runtime-other = 15",
        "hotspot_guard_anchor_count = 25",
        "expected_source_file_count = 38",
        "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
    ] {
        assert!(
            diagnostics_doc.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || large_file_doc.contains(doc_anchor),
            "render product diagnostics owner split docs should retain `{doc_anchor}`"
        );
    }
}
