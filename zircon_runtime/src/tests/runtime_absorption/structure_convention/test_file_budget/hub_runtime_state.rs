use super::{assert_contains_all, read_repo};

const SLICE: &str = "Runtime 15 M3 support Hub runtime-state tests child-owner split";
const STATUS: &str =
    "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred";
const DATE: &str = "2026-06-27";
const GUARD: &str = "runtime_15_support_hub_runtime_state_tests_are_child_owner";
const RUNTIME_STATE: &str = "zircon_hub/src/tauri_app/runtime_state.rs";
const RUNTIME_STATE_TESTS: &str = "zircon_hub/src/tauri_app/runtime_state/tests.rs";
const PARENT_FILE_BUDGET: usize = 1000;
const TEST_FILE_BUDGET: usize = 800;

#[test]
fn runtime_15_support_hub_runtime_state_tests_are_child_owner() {
    let parent = read_repo(RUNTIME_STATE);
    let tests = read_repo(RUNTIME_STATE_TESTS);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let hub_actionable_pages = read_repo("docs/zircon_hub/pages/actionable-pages.md");

    assert_contains_all(
        "Hub runtime-state parent mounts test child owner",
        &parent,
        &["#[path = \"runtime_state/tests.rs\"]", "mod tests;"],
    );
    for moved_test in [
        "fn load_from_paths_merges_repairs_registers_source_and_persists_runtime_state",
        "fn save_settings_action_applies_typed_payload_and_refreshes_source_engine",
        "fn persist_failure_sets_recoverable_status_and_recovers_after_retry",
        "fn project_view_action_status_localizes_in_chinese_view_model",
    ] {
        assert!(
            !parent.contains(moved_test),
            "{RUNTIME_STATE} should delegate Hub runtime-state test fixture `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "{RUNTIME_STATE_TESTS} should own Hub runtime-state test fixture `{moved_test}`"
        );
    }

    assert!(
        parent.lines().count() < PARENT_FILE_BUDGET,
        "{RUNTIME_STATE} should stay below the {PARENT_FILE_BUDGET}-line large-file hotspot threshold after the split"
    );
    assert!(
        tests.lines().count() < TEST_FILE_BUDGET,
        "{RUNTIME_STATE_TESTS} should stay below {TEST_FILE_BUDGET} lines after the split"
    );
}
