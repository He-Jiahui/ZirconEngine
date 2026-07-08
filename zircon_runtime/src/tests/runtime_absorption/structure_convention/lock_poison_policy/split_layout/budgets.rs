use super::super::support::*;
use super::sources::LockPoisonSources;

pub(super) fn assert_lock_poison_owner_budgets(sources: &LockPoisonSources) {
    assert_eq!(
        sources.parent.matches(TEST_ATTRIBUTE).count()
            + sources.core_runtime.matches(TEST_ATTRIBUTE).count()
            + sources
                .core_runtime_config_devtools
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources.core_runtime_global_gate.matches(TEST_ATTRIBUTE).count()
            + sources
                .core_runtime_handle_accessors
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources.core_runtime_scene_eventbus.matches(TEST_ATTRIBUTE).count()
            + sources
                .core_runtime_task_profiling
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources.runtime_services.matches(TEST_ATTRIBUTE).count()
            + sources
                .runtime_services_plugin_bridge
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources
                .runtime_services_dynamic_scene
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources
                .runtime_services_navigation_resource
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources.asset_render_input.matches(TEST_ATTRIBUTE).count()
            + sources
                .asset_render_input_asset_pipeline
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources
                .asset_render_input_render_animation
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources
                .asset_render_input_input_script
                .matches(TEST_ATTRIBUTE)
                .count()
            + sources
                .split_layout_folder_backing
                .matches(TEST_ATTRIBUTE)
                .count(),
        27,
        "lock poison policy parent plus split children should preserve 21 original guards plus the production global gate, the ZrVM runtime lock guard, and four layout guards"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy.rs",
            sources.parent.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime.rs",
            sources.core_runtime.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
            sources.core_runtime_config_devtools.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
            sources.core_runtime_global_gate.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
            sources.core_runtime_handle_accessors.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
            sources.core_runtime_scene_eventbus.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
            sources.core_runtime_task_profiling.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services.rs",
            sources.runtime_services.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
            sources.runtime_services_plugin_bridge.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
            sources.runtime_services_dynamic_scene.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
            sources.runtime_services_navigation_resource.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            sources.asset_render_input.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
            sources.asset_render_input_asset_pipeline.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
            sources.asset_render_input_render_animation.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
            sources.asset_render_input_input_script.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/folder_backing.rs",
            sources.split_layout_folder_backing.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_lock_poison_split_layout_guard_is_folder_backed() {
    let sources = super::sources::read_lock_poison_sources();

    assert_contains_all(
        "lock poison split-layout parent mounts focused child owners",
        &sources.split_layout,
        &[
            "mod budgets;",
            "mod folder_backing;",
            "mod mounts;",
            "mod sources;",
            "mod status_mirrors;",
        ],
    );
    assert!(
        !sources.split_layout.contains(TEST_ATTRIBUTE),
        "lock poison split-layout route owner should not define tests directly"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy/split_layout.rs",
            sources.split_layout.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/sources.rs",
            include_str!("sources.rs"),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/folder_backing.rs",
            include_str!("folder_backing.rs"),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/mounts.rs",
            include_str!("mounts.rs"),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/budgets.rs",
            include_str!("budgets.rs"),
        ),
        (
            "structure_convention/lock_poison_policy/split_layout/status_mirrors.rs",
            include_str!("status_mirrors.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused lock-poison split-layout budget; got {line_count}"
        );
    }
}
