use super::super::*;

pub(super) fn assert_gameplay_host_tests_are_folder_backed() {
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
    let status_rows = format!(
        "{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
        )
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
                "Runtime 15 M3 script VM gameplay host guard child-owner split",
                "runtime_15_script_vm_gameplay_host_guard_child_owner_split_static_passed_cargo_deferred",
                "script/vm/gameplay_host/tests.rs",
                "script/vm/gameplay_host/tests/spawn_transform.rs",
                "script/vm/gameplay_host/tests/property_animation.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/gameplay_host.rs",
                "runtime_15_gameplay_host_tests_are_folder_backed",
                "runtime_15_script_vm_gameplay_host_guard_is_child_owner",
            ],
        );
    }
}
