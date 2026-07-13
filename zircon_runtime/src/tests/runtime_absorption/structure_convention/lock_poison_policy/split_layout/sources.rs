use super::super::support::*;

pub(super) struct LockPoisonSources {
    pub(super) parent: String,
    pub(super) split_layout: String,
    pub(super) split_layout_folder_backing: String,
    pub(super) support: String,
    pub(super) core_runtime: String,
    pub(super) core_runtime_config_devtools: String,
    pub(super) core_runtime_global_gate: String,
    pub(super) core_runtime_handle_accessors: String,
    pub(super) core_runtime_scene_eventbus: String,
    pub(super) core_runtime_task_profiling: String,
    pub(super) runtime_services: String,
    pub(super) runtime_services_plugin_bridge: String,
    pub(super) runtime_services_dynamic_scene: String,
    pub(super) runtime_services_navigation_resource: String,
    pub(super) asset_render_input: String,
    pub(super) asset_render_input_asset_pipeline: String,
    pub(super) asset_render_input_render_animation: String,
    pub(super) asset_render_input_input_script: String,
    pub(super) runtime_15_plan: String,
    pub(super) runtime_index: String,
    pub(super) review_findings: String,
    pub(super) structure_convention: String,
    pub(super) module_doc: String,
    pub(super) frameworks_plan: String,
    pub(super) lock_poison_status_rows: String,
    pub(super) lock_poison_policy_guard_rows: String,
}

pub(super) fn read_lock_poison_sources() -> LockPoisonSources {
    LockPoisonSources {
        parent: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy.rs",
        ),
        split_layout: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout.rs",
        ),
        split_layout_folder_backing: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/folder_backing.rs",
        ),
        support: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/support.rs",
        ),
        core_runtime: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs",
        ),
        core_runtime_config_devtools: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
        ),
        core_runtime_global_gate: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
        ),
        core_runtime_handle_accessors: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
        ),
        core_runtime_scene_eventbus: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
        ),
        core_runtime_task_profiling: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
        ),
        runtime_services: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs",
        ),
        runtime_services_plugin_bridge: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
        ),
        runtime_services_dynamic_scene: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
        ),
        runtime_services_navigation_resource: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
        ),
        asset_render_input: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs",
        ),
        asset_render_input_asset_pipeline: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
        ),
        asset_render_input_render_animation: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
        ),
        asset_render_input_input_script: read_runtime_src(
            "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
        ),
        runtime_15_plan: read_repo(
            "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        ),
        runtime_index: read_repo(
            "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
        ),
        review_findings: read_repo(
            "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
        ),
        structure_convention: read_repo(
            "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
        ),
        module_doc: read_repo("docs/zircon_runtime/structure/module-convention.md"),
        frameworks_plan: read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        lock_poison_status_rows: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
        ),
        lock_poison_policy_guard_rows: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs",
        ),
    }
}
