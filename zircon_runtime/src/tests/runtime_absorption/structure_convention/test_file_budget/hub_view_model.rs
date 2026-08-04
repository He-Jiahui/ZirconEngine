use super::{assert_contains_all, read_repo};

const SLICE: &str = "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split";
const STATUS: &str = "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred";
const DATE: &str = "2026-06-27";
const GUARD: &str = "runtime_15_support_hub_view_model_quick_actions_tests_are_child_owners";
const VIEW_MODEL: &str = "zircon_hub/src/tauri_app/view_model.rs";
const VIEW_MODEL_QUICK_ACTIONS: &str = "zircon_hub/src/tauri_app/view_model/quick_actions.rs";
const VIEW_MODEL_TESTS: &str = "zircon_hub/src/tauri_app/view_model/tests.rs";
const FILE_BUDGET: usize = 800;

#[test]
fn runtime_15_support_hub_view_model_quick_actions_tests_are_child_owners() {
    let parent = read_repo(VIEW_MODEL);
    let quick_actions = read_repo(VIEW_MODEL_QUICK_ACTIONS);
    let tests = read_repo(VIEW_MODEL_TESTS);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let hub_actionable_pages = read_repo("docs/zircon_hub/pages/actionable-pages.md");

    assert_contains_all(
        "Hub view-model parent mounts quick-actions and tests child owners",
        &parent,
        &[
            "mod quick_actions;",
            "use quick_actions::quick_actions;",
            "#[path = \"view_model/tests.rs\"]",
            "mod tests;",
        ],
    );
    assert!(
        !parent.contains("fn quick_actions("),
        "{VIEW_MODEL} should delegate quick action projection logic to {VIEW_MODEL_QUICK_ACTIONS}"
    );
    assert_contains_all(
        "Hub view-model quick-actions child owns projection logic",
        &quick_actions,
        &[
            "pub(super) fn quick_actions(",
            "enum QuickActionKind",
            "enum QuickActionProjectTarget",
            "enum QuickActionSourceEngineState",
            "fn quick_action_detail",
            "fn project_action_detail",
        ],
    );
    for moved_owner in [
        "enum QuickActionKind",
        "enum QuickActionProjectTarget",
        "enum QuickActionSourceEngineState",
        "fn quick_action_detail",
        "fn project_action_detail",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "{VIEW_MODEL} should not regain quick-action owner `{moved_owner}`"
        );
    }

    for moved_test in [
        "fn view_model_projects_come_from_snapshot_filtering_and_state_ids",
        "fn quick_actions_use_selected_project_scope_and_engine_binding",
        "fn quick_actions_disable_unbound_or_stale_project_targets",
        "fn task_summary_localizes_backend_operation_targets",
    ] {
        assert!(
            !parent.contains(moved_test),
            "{VIEW_MODEL} should delegate Hub view-model test fixture `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "{VIEW_MODEL_TESTS} should own Hub view-model test fixture `{moved_test}`"
        );
    }

    for (path, source) in [
        (VIEW_MODEL, parent.as_str()),
        (VIEW_MODEL_QUICK_ACTIONS, quick_actions.as_str()),
        (VIEW_MODEL_TESTS, tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < FILE_BUDGET,
            "{path} should stay below {FILE_BUDGET} lines after the Hub view-model owner split; got {line_count}"
        );
    }
}
