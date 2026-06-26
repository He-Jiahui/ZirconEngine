use super::*;

#[test]
fn runtime_15_script_vm_tests_are_folder_backed() {
    let parent = read_runtime_src("script/vm/tests.rs");
    let bridge_host = read_runtime_src("script/vm/tests/bridge_host.rs");
    let host_exports = read_runtime_src("script/vm/tests/host_exports.rs");
    let lifecycle_failures = read_runtime_src("script/vm/tests/lifecycle_failures.rs");
    let module_surface = read_runtime_src("script/vm/tests/module_surface.rs");
    let plugin_runtime = read_runtime_src("script/vm/tests/plugin_runtime.rs");
    let reflection_docs = read_runtime_src("script/vm/tests/reflection_docs.rs");
    let support = read_runtime_src("script/vm/tests/support.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_doc = read_repo("docs/zircon_runtime/script/vm/tests.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "script VM parent test module mounts",
        &parent,
        &[
            "#[cfg(test)]\nmod lifecycle_failures;",
            "mod bridge_host;",
            "mod host_exports;",
            "mod module_surface;",
            "mod plugin_runtime;",
            "mod reflection_docs;",
            "mod support;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "script/vm/tests.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn host_export_registry_validates_descriptors_and_dispatches_callbacks",
        "fn bridge_host_module_registers_methods_from_package_manifest",
        "fn host_reflection_docs_render_synthetic_descriptor_deterministically",
        "fn vm_plugin_manager_discovers_packages_selects_backends_and_loads_slots",
        "fn core_resolve_plugin_exposes_vm_plugin_runtime_and_manager_facade_shares_it",
    ] {
        assert!(
            !parent.contains(moved_test),
            "script/vm/tests.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "host exports child owns host registry contracts",
        &host_exports,
        &[
            "use super::*;",
            "fn host_handles_are_stable_and_valid",
            "fn script_call_table_pre_resolves_host_export_callbacks",
            "fn zr_vm_real_backend_uses_script_call_table_for_host_callbacks",
        ],
    );
    assert_contains_all(
        "bridge host child owns bridge dispatch contracts",
        &bridge_host,
        &[
            "use super::*;",
            "trait VmWeatherBridge",
            "fn bridge_host_module_dispatches_vm_calls_through_resolved_bridge_slots",
            "fn bridge_host_module_reports_disabled_bridge_to_vm_callers",
        ],
    );
    assert_contains_all(
        "reflection docs child owns markdown and macro contracts",
        &reflection_docs,
        &[
            "fn host_reflection_docs_render_synthetic_descriptor_deterministically",
            "fn host_reflection_docs_writer_creates_parent_directory_and_file",
            "fn rust_reflection_macros_generate_type_function_and_module_descriptors",
        ],
    );
    assert_contains_all(
        "plugin runtime child owns VM lifecycle contracts",
        &plugin_runtime,
        &[
            "use super::support::*;",
            "fn hot_reload_coordinator_tracks_slot_lifecycle_records",
            "fn vm_plugin_manager_propagates_host_context_roots_and_backend_selector",
            "fn vm_plugin_discovery_supports_zr_vm_project_packages_without_bytecode",
        ],
    );
    assert_contains_all(
        "module surface child owns structure and runtime wiring contracts",
        &module_surface,
        &[
            "use super::support::*;",
            "fn builtin_host_modules_register_gameplay_capabilities",
            "fn vm_plugin_protocol_types_live_in_script_subsystem",
            "fn vm_subsystem_is_grouped_by_module_backend_host_plugin_and_runtime",
        ],
    );
    assert_contains_all(
        "support child owns shared script VM fixtures",
        &support,
        &[
            "pub(super) struct PluginFixture",
            "pub(super) struct ZrVmProjectFixture",
            "pub(super) fn test_package",
            "pub(super) fn test_host_context",
        ],
    );

    let migrated_test_count = [
        bridge_host.as_str(),
        host_exports.as_str(),
        lifecycle_failures.as_str(),
        module_surface.as_str(),
        plugin_runtime.as_str(),
        reflection_docs.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 32,
        "script VM child modules should preserve the original 32 tests including lifecycle failures"
    );

    for (path, source) in [
        ("script/vm/tests.rs", parent.as_str()),
        ("script/vm/tests/bridge_host.rs", bridge_host.as_str()),
        ("script/vm/tests/host_exports.rs", host_exports.as_str()),
        (
            "script/vm/tests/lifecycle_failures.rs",
            lifecycle_failures.as_str(),
        ),
        ("script/vm/tests/module_surface.rs", module_surface.as_str()),
        ("script/vm/tests/plugin_runtime.rs", plugin_runtime.as_str()),
        (
            "script/vm/tests/reflection_docs.rs",
            reflection_docs.as_str(),
        ),
        ("script/vm/tests/support.rs", support.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM test doc", script_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 script VM test folder split",
                "runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result",
                "script/vm/tests.rs",
                "script/vm/tests/host_exports.rs",
                "script/vm/tests/reflection_docs.rs",
                "runtime_15_script_vm_tests_are_folder_backed",
            ],
        );
    }
}

#[test]
fn runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed() {
    let parent = read_runtime_src("script/vm/runtime/hot_reload_coordinator.rs");
    let child = read_runtime_src("script/vm/runtime/hot_reload_coordinator/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "hot reload coordinator parent mounts test child owner",
        &parent,
        &["#[cfg(test)]\nmod tests;"],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "hot_reload_coordinator.rs should mount a child test owner instead of keeping executable tests"
    );

    for moved_test in [
        "fn hot_reload_policy_preserves_state_and_increments_generation_by_default",
        "fn stateless_hot_reload_policy_skips_state_transfer",
        "fn disabled_hot_reload_policy_rejects_reload_without_deactivating_slot",
        "fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking",
        "fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
    ] {
        assert!(
            !parent.contains(moved_test),
            "hot_reload_coordinator.rs should mount child test owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "hot reload coordinator child owns lifecycle, policy, and poison tests",
        &child,
        &[
            "use super::*;",
            "struct PolicyRecordingBackend",
            "struct LifecycleQueryBackend",
            "struct CoordinatorSlotLifecycle",
            "fn hot_reload_policy_preserves_state_and_increments_generation_by_default",
            "fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking",
            "fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
        ],
    );
    assert_eq!(
        child.matches("#[test]").count(),
        5,
        "hot reload coordinator child should preserve the original 5 module-local tests"
    );

    for (path, source) in [
        (
            "script/vm/runtime/hot_reload_coordinator.rs",
            parent.as_str(),
        ),
        (
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM doc", script_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 script VM hot-reload coordinator test folder split",
                "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
                "script/vm/runtime/hot_reload_coordinator.rs",
                "script/vm/runtime/hot_reload_coordinator/tests.rs",
                "runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
            ],
        );
    }
}

#[test]
fn runtime_15_gameplay_host_tests_are_folder_backed() {
    let parent = read_runtime_src("script/vm/gameplay_host/tests.rs");
    let spawn_transform = read_runtime_src("script/vm/gameplay_host/tests/spawn_transform.rs");
    let component_state = read_runtime_src("script/vm/gameplay_host/tests/component_state.rs");
    let combat_lifecycle = read_runtime_src("script/vm/gameplay_host/tests/combat_lifecycle.rs");
    let property_animation =
        read_runtime_src("script/vm/gameplay_host/tests/property_animation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let gameplay_doc = read_repo("docs/zircon_runtime/script/vm/gameplay_host.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "gameplay host parent test module mounts child owners",
        &parent,
        &[
            "mod combat_lifecycle;",
            "mod component_state;",
            "mod property_animation;",
            "mod spawn_transform;",
            "fn assert_vec3_close",
            "fn assert_quat_close",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "script/vm/gameplay_host/tests.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn gameplay_pose_exports_update_entity_transform",
        "fn gameplay_host_spawn_model_sets_bindings_and_hud_text",
        "fn gameplay_host_current_hp_and_particle_sprites_use_dynamic_components",
        "fn gameplay_host_component_string_reads_string_dynamic_state",
        "fn gameplay_host_damage_report_preserves_death_position",
        "fn script_held_entity_handle_reports_invalid_after_despawn",
        "fn gameplay_host_damage_entity_reports_hit_before_death",
        "fn gameplay_host_script_property_match_and_heal_update_bindings",
        "fn gameplay_host_sets_animation_bool_and_world_hud_bar_for_scripted_gameplay",
    ] {
        assert!(
            !parent.contains(moved_test),
            "script/vm/gameplay_host/tests.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "spawn/transform gameplay child owns transform and spawn contracts",
        &spawn_transform,
        &[
            "use super::*;",
            "fn gameplay_pose_exports_update_entity_transform",
            "fn gameplay_host_spawn_model_sets_bindings_and_hud_text",
        ],
    );
    assert_contains_all(
        "component-state gameplay child owns dynamic component contracts",
        &component_state,
        &[
            "use super::*;",
            "fn gameplay_host_current_hp_and_particle_sprites_use_dynamic_components",
            "fn gameplay_host_component_string_reads_string_dynamic_state",
        ],
    );
    assert_contains_all(
        "combat/lifecycle gameplay child owns damage and stale-handle contracts",
        &combat_lifecycle,
        &[
            "use super::*;",
            "fn gameplay_host_damage_report_preserves_death_position",
            "fn script_held_entity_handle_reports_invalid_after_despawn",
            "fn gameplay_host_damage_entity_reports_hit_before_death",
        ],
    );
    assert_contains_all(
        "property/animation gameplay child owns bindings and HUD contracts",
        &property_animation,
        &[
            "use super::*;",
            "fn gameplay_host_script_property_match_and_heal_update_bindings",
            "fn gameplay_host_sets_animation_bool_and_world_hud_bar_for_scripted_gameplay",
        ],
    );

    let migrated_test_count = [
        spawn_transform.as_str(),
        component_state.as_str(),
        combat_lifecycle.as_str(),
        property_animation.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 9,
        "gameplay host child modules should preserve the original 9 tests"
    );

    for (path, source) in [
        ("script/vm/gameplay_host/tests.rs", parent.as_str()),
        (
            "script/vm/gameplay_host/tests/spawn_transform.rs",
            spawn_transform.as_str(),
        ),
        (
            "script/vm/gameplay_host/tests/component_state.rs",
            component_state.as_str(),
        ),
        (
            "script/vm/gameplay_host/tests/combat_lifecycle.rs",
            combat_lifecycle.as_str(),
        ),
        (
            "script/vm/gameplay_host/tests/property_animation.rs",
            property_animation.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("gameplay host doc", gameplay_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 gameplay host test folder split",
                "runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred",
                "script/vm/gameplay_host/tests.rs",
                "script/vm/gameplay_host/tests/spawn_transform.rs",
                "script/vm/gameplay_host/tests/property_animation.rs",
                "runtime_15_gameplay_host_tests_are_folder_backed",
            ],
        );
    }
}
