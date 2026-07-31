pub(super) struct HotspotInventorySources {
    pub(super) runtime_07_plan: &'static str,
    pub(super) runtime_07_archive: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) render_index: &'static str,
    pub(super) hotspot_doc: &'static str,
    pub(super) dynamic_session_doc: &'static str,
    pub(super) ecs_doc: &'static str,
    pub(super) animation_doc: &'static str,
    pub(super) diagnostics_doc: &'static str,
    pub(super) architecture_review: &'static str,
    pub(super) schedule_runner: &'static str,
    pub(super) ecs_frame_diagnostics: &'static str,
    pub(super) query_filter: &'static str,
    pub(super) query_iter: &'static str,
    pub(super) query_many_iter: &'static str,
    pub(super) query_state_root: &'static str,
    pub(super) query_state_cache: &'static str,
    pub(super) query_state_read_only_cached: &'static str,
    pub(super) query_state_stats: &'static str,
    pub(super) query_state_system_param: &'static str,
    pub(super) system_param: &'static str,
    pub(super) system_state: &'static str,
    pub(super) param_set: &'static str,
    pub(super) world_performance_diagnostics: &'static str,
    pub(super) world_driver: &'static str,
    pub(super) query_tests: &'static str,
    pub(super) change_tests: &'static str,
    pub(super) session_tests: &'static str,
    pub(super) session_extract_cache: &'static str,
    pub(super) session_extract_stats: &'static str,
    pub(super) asset_worker_source: &'static str,
    pub(super) asset_worker_manager: &'static str,
    pub(super) asset_worker_tests: &'static str,
    pub(super) animation_plugin_runtime_system: &'static str,
    pub(super) animation_plugin_tick: &'static str,
    pub(super) animation_plugin_events: &'static str,
    pub(super) animation_clip_events: &'static str,
    pub(super) animation_level_system: &'static str,
    pub(super) root_manifest: &'static str,
    pub(super) runtime_manifest: &'static str,
    pub(super) zircon_build: &'static str,
    pub(super) dev_fast_build: &'static str,
    pub(super) build_tool_doc: &'static str,
    pub(super) profiling_doc: &'static str,
    pub(super) interface_profiling: &'static str,
    pub(super) profiling_counter_hotspot: &'static str,
    pub(super) profiling_export: &'static str,
    pub(super) profiling_mod: &'static str,
    pub(super) render_profiling: &'static str,
}

impl HotspotInventorySources {
    pub(super) fn load() -> Self {
        Self {
            runtime_07_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
            ),
            runtime_07_archive: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
            ),
            runtime_index: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/index.md"
            ),
            render_index: include_str!(
                "../../../../../../docs/plans/zircon_runtime/render/index.md"
            ),
            hotspot_doc: include_str!(
                "../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md"
            ),
            dynamic_session_doc: include_str!(
                "../../../../../../docs/zircon_runtime/dynamic_api/session.md"
            ),
            ecs_doc: include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md"),
            animation_doc: include_str!(
                "../../../../../../docs/zircon_runtime/animation/runtime.md"
            ),
            diagnostics_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/diagnostics.md"
            ),
            architecture_review: include_str!(
                "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
            schedule_runner: include_str!("../../../../scene/ecs/schedule_runner.rs"),
            ecs_frame_diagnostics: include_str!(
                "../../../../scene/ecs/frame_performance_diagnostics.rs"
            ),
            query_filter: include_str!("../../../../scene/ecs/query/query_filter.rs"),
            query_iter: include_str!("../../../../scene/ecs/query/query_iter.rs"),
            query_many_iter: include_str!("../../../../scene/ecs/query/query_many_iter.rs"),
            query_state_root: include_str!("../../../../scene/ecs/query/query_state/mod.rs"),
            query_state_cache: include_str!("../../../../scene/ecs/query/query_state/cache.rs"),
            query_state_read_only_cached: include_str!(
                "../../../../scene/ecs/query/query_state/read_only_cached.rs"
            ),
            query_state_stats: include_str!("../../../../scene/ecs/query/query_state/stats.rs"),
            query_state_system_param: include_str!(
                "../../../../scene/ecs/query/query_state/system_param.rs"
            ),
            system_param: include_str!("../../../../scene/ecs/system/system_param.rs"),
            system_state: include_str!("../../../../scene/ecs/system/system_state.rs"),
            param_set: include_str!("../../../../scene/ecs/system/param_set.rs"),
            world_performance_diagnostics: include_str!(
                "../../../../scene/world/performance_diagnostics.rs"
            ),
            world_driver: include_str!("../../../../scene/module/world_driver.rs"),
            query_tests: include_str!("../../../../scene/tests/ecs_performance_acceptance.rs"),
            change_tests: include_str!("../../../../scene/tests/ecs_change_detection.rs"),
            session_tests: include_str!(
                "../../../../dynamic_api/session/tests/frame_diagnostics.rs"
            ),
            session_extract_cache: include_str!("../../../../dynamic_api/session/extract_cache.rs"),
            session_extract_stats: include_str!("../../../../dynamic_api/session/extract_stats.rs"),
            asset_worker_source: include_str!("../../../../asset/pipeline/worker_pool.rs"),
            asset_worker_manager: include_str!(
                "../../../../asset/pipeline/manager/project_asset_manager/construction.rs"
            ),
            asset_worker_tests: include_str!("../../../../asset/tests/pipeline/worker_pool.rs"),
            animation_plugin_runtime_system: include_str!(
                "../../../../../../zircon_plugins/animation/runtime/src/runtime_system.rs"
            ),
            animation_plugin_tick: include_str!(
                "../../../../../../zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs"
            ),
            animation_plugin_events: include_str!(
                "../../../../../../zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs"
            ),
            animation_clip_events: include_str!("../../../../animation/clip_event.rs"),
            animation_level_system: include_str!("../../../../scene/level_system.rs"),
            root_manifest: include_str!("../../../../../../Cargo.toml"),
            runtime_manifest: include_str!("../../../../../../zircon_runtime/Cargo.toml"),
            zircon_build: include_str!("../../../../../../tools/zircon_build.py"),
            dev_fast_build: include_str!("../../../../../../tools/dev-fast-build.ps1"),
            build_tool_doc: include_str!(
                "../../../../../../docs/cli-and-tooling/zircon-build-tool.md"
            ),
            profiling_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/diagnostics/profiling.md"
            ),
            interface_profiling: include_str!(
                "../../../../../../zircon_runtime_interface/src/profiling.rs"
            ),
            profiling_counter_hotspot: include_str!(
                "../../../../core/runtime/diagnostics/profiling/counter_hotspot.rs"
            ),
            profiling_export: include_str!(
                "../../../../core/runtime/diagnostics/profiling/export.rs"
            ),
            profiling_mod: include_str!("../../../../core/runtime/diagnostics/profiling/mod.rs"),
            render_profiling: include_str!("../../../../graphics/tests/render_profiling.rs"),
        }
    }
}
