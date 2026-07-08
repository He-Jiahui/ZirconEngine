pub(super) struct RecentStaticGuardSources {
    pub(super) runtime_index: &'static str,
    pub(super) review: &'static str,
    pub(super) runtime_01_plan: &'static str,
    pub(super) runtime_01_tech_stack_doc: &'static str,
    pub(super) runtime_01_text_doc: &'static str,
    pub(super) runtime_01_physics_doc: &'static str,
    pub(super) runtime_01_editor_backlog_doc: &'static str,
    pub(super) runtime_02_plan: &'static str,
    pub(super) runtime_02_root_doc: &'static str,
    pub(super) runtime_02_generated_doc: &'static str,
    pub(super) runtime_03_plan: &'static str,
    pub(super) runtime_03_frame_doc: &'static str,
    pub(super) runtime_03_parallel_doc: &'static str,
    pub(super) runtime_04_plan: &'static str,
    pub(super) runtime_04_facade_doc: &'static str,
    pub(super) runtime_04_worker_doc: &'static str,
    pub(super) runtime_04_watcher_doc: &'static str,
    pub(super) runtime_04_artifact_doc: &'static str,
    pub(super) runtime_04_resource_doc: &'static str,
    pub(super) runtime_05_plan: &'static str,
    pub(super) runtime_06_plan: &'static str,
    pub(super) runtime_06_native_doc: &'static str,
    pub(super) runtime_06_interface_doc: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) runtime_07_doc: &'static str,
    pub(super) runtime_08_plan: &'static str,
    pub(super) runtime_08_doc: &'static str,
    pub(super) runtime_09_plan: &'static str,
    pub(super) runtime_09_doc: &'static str,
    pub(super) runtime_10_plan: &'static str,
    pub(super) runtime_10_doc: &'static str,
    pub(super) runtime_10_interface_doc: &'static str,
    pub(super) runtime_11_plan: &'static str,
    pub(super) runtime_11_doc: &'static str,
    pub(super) runtime_12_plan: &'static str,
    pub(super) runtime_12_doc: &'static str,
    pub(super) runtime_13_plan: &'static str,
    pub(super) runtime_13_doc: &'static str,
    pub(super) runtime_14_plan: &'static str,
    pub(super) runtime_14_animation_doc: &'static str,
    pub(super) runtime_14_navigation_doc: &'static str,
    pub(super) runtime_14_diagnostic_doc: &'static str,
    pub(super) runtime_14_engine_module_doc: &'static str,
}

impl RecentStaticGuardSources {
    pub(super) fn load() -> Self {
        Self {
            runtime_index: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/index.md"
            ),
            review: include_str!(
                "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
            runtime_01_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
            ),
            runtime_01_tech_stack_doc: include_str!(
                "../../../../../../docs/engine-architecture/runtime-tech-stack.md"
            ),
            runtime_01_text_doc: include_str!("../../../../../../docs/zircon_runtime/ui/text.md"),
            runtime_01_physics_doc: include_str!(
                "../../../../../../docs/zircon_plugins/physics-plugin-options.md"
            ),
            runtime_01_editor_backlog_doc: include_str!(
                "../../../../../../docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md"
            ),
            runtime_02_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
            ),
            runtime_02_root_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/root_surface.md"
            ),
            runtime_02_generated_doc: include_str!(
                "../../../../../../docs/engine-architecture/generated-code-boundary.md"
            ),
            runtime_03_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
            ),
            runtime_03_frame_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/frame_schedule.md"
            ),
            runtime_03_parallel_doc: include_str!(
                "../../../../../../docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md"
            ),
            runtime_04_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
            ),
            runtime_04_facade_doc: include_str!(
                "../../../../../../docs/zircon_runtime/asset/facade.md"
            ),
            runtime_04_worker_doc: include_str!(
                "../../../../../../docs/zircon_runtime/asset/worker_pool.md"
            ),
            runtime_04_watcher_doc: include_str!(
                "../../../../../../docs/zircon_runtime/asset/watcher.md"
            ),
            runtime_04_artifact_doc: include_str!(
                "../../../../../../docs/zircon_runtime/asset/artifact.md"
            ),
            runtime_04_resource_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/resource.md"
            ),
            runtime_05_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
            ),
            runtime_06_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
            ),
            runtime_06_native_doc: include_str!(
                "../../../../../../docs/engine-architecture/native-plugin-boundary.md"
            ),
            runtime_06_interface_doc: include_str!(
                "../../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
            runtime_07_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
            ),
            runtime_07_doc: include_str!(
                "../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md"
            ),
            runtime_08_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
            ),
            runtime_08_doc: include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md"),
            runtime_09_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
            ),
            runtime_09_doc: include_str!(
                "../../../../../../docs/zircon_runtime/ui/architecture.md"
            ),
            runtime_10_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
            ),
            runtime_10_doc: include_str!(
                "../../../../../../docs/zircon_runtime/dynamic_api/session.md"
            ),
            runtime_10_interface_doc: include_str!(
                "../../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
            runtime_11_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
            ),
            runtime_11_doc: include_str!(
                "../../../../../../docs/zircon_runtime/core/job_system.md"
            ),
            runtime_12_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
            ),
            runtime_12_doc: include_str!(
                "../../../../../../docs/zircon_runtime/input/input_state.md"
            ),
            runtime_13_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
            ),
            runtime_13_doc: include_str!(
                "../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md"
            ),
            runtime_14_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
            ),
            runtime_14_animation_doc: include_str!(
                "../../../../../../docs/zircon_runtime/animation/runtime.md"
            ),
            runtime_14_navigation_doc: include_str!(
                "../../../../../../docs/zircon_runtime/navigation/runtime.md"
            ),
            runtime_14_diagnostic_doc: include_str!(
                "../../../../../../docs/zircon_runtime/diagnostic_log/mod.md"
            ),
            runtime_14_engine_module_doc: include_str!(
                "../../../../../../docs/zircon_runtime/engine_module/relationship.md"
            ),
        }
    }
}
