use super::super::support::*;
use super::sources::LockPoisonSources;

pub(super) fn assert_parent_mounts_child_owners(sources: &LockPoisonSources) {
    assert_contains_all(
        "lock poison policy parent mounts child owners",
        &sources.parent,
        &[
            "mod asset_render_input;",
            "mod core_runtime;",
            "mod runtime_services;",
            "mod split_layout;",
            "mod support;",
        ],
    );

    assert_contains_all(
        "core runtime lock poison child owns core guards",
        &sources.core_runtime,
        &[
            "mod config_devtools;",
            "mod global_gate;",
            "mod handle_accessors;",
            "mod scene_eventbus;",
            "mod task_profiling;",
        ],
    );
    assert_contains_all(
        "runtime services lock poison child mounts plugin scene resource owners",
        &sources.runtime_services,
        &[
            "mod dynamic_scene;",
            "mod navigation_resource;",
            "mod plugin_bridge;",
            concat!(
                "fn ",
                "runtime_15_runtime_services_lock_poison_guard_child_owner_split"
            ),
        ],
    );
    assert_contains_all(
        "asset render input lock poison child mounts asset graphics input owners",
        &sources.asset_render_input,
        &[
            "mod asset_pipeline;",
            "mod input_script;",
            "mod render_animation;",
            concat!(
                "fn ",
                "runtime_15_asset_render_input_lock_poison_guard_child_owner_split"
            ),
        ],
    );
}

pub(super) fn assert_lock_poison_guards_stay_in_children(sources: &LockPoisonSources) {
    for moved_guard in [
        concat!(
            "fn ",
            "runtime_15_lock_poison_policy_guard_is_folder_backed"
        ),
        concat!(
            "fn ",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
        ),
        concat!(
            "fn ",
            "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors"
        ),
        concat!(
            "fn ",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
        ),
        concat!(
            "fn ",
            "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
        ),
    ] {
        assert!(
            !sources.parent.contains(moved_guard),
            "lock poison policy parent should mount child owners instead of defining {moved_guard}"
        );
    }

    let core_runtime_children = format!(
        "{}\n{}\n{}\n{}\n{}",
        sources.core_runtime_config_devtools,
        sources.core_runtime_global_gate,
        sources.core_runtime_handle_accessors,
        sources.core_runtime_scene_eventbus,
        sources.core_runtime_task_profiling
    );
    assert_contains_all(
        "core runtime lock poison children preserve core guards",
        &core_runtime_children,
        &[
            concat!(
                "fn ",
                "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
            ),
            concat!(
                "fn ",
                "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors"
            ),
        ],
    );

    let runtime_services_children = format!(
        "{}\n{}\n{}",
        sources.runtime_services_plugin_bridge,
        sources.runtime_services_dynamic_scene,
        sources.runtime_services_navigation_resource
    );
    assert_contains_all(
        "runtime services lock poison children preserve plugin scene resource guards",
        &runtime_services_children,
        &[
            concat!(
                "fn ",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
            ),
            concat!(
                "fn ",
                "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task"
            ),
            concat!(
                "fn ",
                "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
            ),
        ],
    );

    let asset_render_input_children = format!(
        "{}\n{}\n{}",
        sources.asset_render_input_asset_pipeline,
        sources.asset_render_input_render_animation,
        sources.asset_render_input_input_script
    );
    assert_contains_all(
        "asset render input lock poison children preserve asset graphics input guards",
        &asset_render_input_children,
        &[
            concat!(
                "fn ",
                "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
            ),
            concat!(
                "fn ",
                "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
            ),
            concat!(
                "fn ",
                "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock"
            ),
        ],
    );
}
