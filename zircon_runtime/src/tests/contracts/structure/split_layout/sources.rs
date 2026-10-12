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
    }
}
