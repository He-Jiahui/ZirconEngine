use super::super::*;
use super::super::{guard_names::GuardNames, sources::GuardSources};

pub(super) fn assert_runtime_scene_children(sources: &GuardSources, guards: &GuardNames) {
    assert_contains_all(
        "code review findings test-budget child owns findings guard",
        &sources.code_review_findings,
        &[
            "#[path = \"code_review_findings/folder_backed_summary.rs\"]",
            "mod folder_backed_summary;",
            guards.code_review_findings_guard.as_str(),
        ],
    );
    assert_contains_all(
        "core framework test-budget child owns historical core-framework guard",
        &sources.core_framework,
        &[
            "use super::*;",
            "fn runtime_15_core_framework_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "core runtime deactivation test-budget child owns deactivation guard",
        &sources.core_runtime_deactivation,
        &[
            "use super::*;",
            guards.core_runtime_deactivation_guard.as_str(),
        ],
    );
    assert_contains_all(
        "dynamic-scene absorption test-budget child owns absorption guard",
        &sources.dynamic_scene_absorption,
        &[
            "use super::*;",
            guards.dynamic_scene_absorption_guard.as_str(),
        ],
    );
    assert_contains_all(
        "runtime diagnostics test-budget child owns diagnostics guard",
        &sources.runtime_diagnostics,
        &["use super::*;", guards.runtime_diagnostics_guard.as_str()],
    );
    assert_contains_all(
        "scene component-structure test-budget child owns component-structure guard",
        &sources.scene_component_structure,
        &[
            "use super::*;",
            guards.scene_component_structure_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene derived-state test-budget child owns derived-state guard",
        &sources.scene_derived_state,
        &["use super::*;", guards.scene_derived_state_guard.as_str()],
    );
    assert_contains_all(
        "dynamic-scene root test-budget child owns dynamic-scene root guard",
        &sources.scene_dynamic_scene_root,
        &[
            "use super::*;",
            guards.scene_dynamic_scene_root_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene dynamic-session test-budget child owns path-management guard",
        &sources.scene_dynamic_session,
        &["use super::*;", guards.scene_dynamic_session_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS reflect foundation test-budget child owns foundation guard",
        &sources.scene_ecs_reflect_foundation,
        &[
            "use super::*;",
            guards.scene_ecs_reflect_foundation_guard.as_str(),
        ],
    );
    assert_contains_all(
        "test-file budget module-layout child owns parent guard split",
        &sources.test_file_budget_module_layout,
        &[
            "use super::*;",
            "fn runtime_15_test_file_budget_parent_guard_child_owner_split",
        ],
    );
}
