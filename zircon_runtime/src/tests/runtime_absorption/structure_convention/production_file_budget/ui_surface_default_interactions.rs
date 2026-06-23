use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners() {
    let parent = read_runtime_src("ui/surface/surface/default_interactions.rs");
    let keyboard = read_runtime_src("ui/surface/surface/default_interactions/keyboard.rs");
    let timers = read_runtime_src("ui/surface/surface/default_interactions/timers.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "default interactions parent keeps pointer/toggle action entry points and shared binding helpers",
        &parent,
        &[
            "mod keyboard;",
            "mod timers;",
            "pub(super) fn apply_default_pointer_component_actions(",
            "fn apply_default_button_component_action(",
            "fn default_toggle_next_checked(",
            "fn default_open_boolean_value(",
            "fn component_event_reports_for_bindings(",
            "fn binding_targets_component_event(",
        ],
    );
    for moved_owner in [
        "pub(crate) fn apply_default_keyboard_component_action(",
        "pub(crate) fn apply_default_semantic_keyboard_component_action(",
        "pub(crate) fn typeahead_timeout_ms_for_component_node(",
        "pub(crate) fn apply_default_submenu_hover_ready_component_event(",
        "fn semantic_keyboard_action_for_behavior(",
        "fn tooltip_id_for_metadata(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/surface/surface/default_interactions.rs should delegate `{moved_owner}` to keyboard.rs or timers.rs"
        );
    }

    assert_contains_all(
        "keyboard child owns keyboard-driven default component actions",
        &keyboard,
        &[
            "pub(crate) fn apply_default_keyboard_component_action(",
            "pub(crate) fn apply_default_semantic_keyboard_component_action(",
            "pub(crate) fn apply_default_semantic_keyboard_component_text(",
            "fn default_keyboard_behavior(",
            "fn semantic_keyboard_action_for_behavior(",
            "fn semantic_keyboard_event_kinds(",
            "UiComponentKeyboardAction",
            "UiDefaultKeyboardActionReport",
        ],
    );
    assert_contains_all(
        "timers child owns menu/tooltip default interaction timers",
        &timers,
        &[
            "const DEFAULT_TYPEAHEAD_TIMEOUT_MS",
            "const DEFAULT_SUBMENU_HOVER_DELAY_MS",
            "const DEFAULT_TOOLTIP_DELAY_MS",
            "pub(crate) fn typeahead_timeout_ms_for_component_node(",
            "pub(crate) fn submenu_hover_delay_ms_for_component_node(",
            "pub(crate) fn tooltip_timer_for_component_node(",
            "pub(crate) fn apply_default_typeahead_expired_component_event(",
            "pub(crate) fn apply_default_submenu_hover_ready_component_event(",
            "fn is_menu_component(",
            "fn tooltip_id_for_metadata(",
        ],
    );

    for (path, source) in [
        (
            "ui/surface/surface/default_interactions.rs",
            parent.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/keyboard.rs",
            keyboard.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/timers.rs",
            timers.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split",
                "runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred",
                "ui/surface/surface/default_interactions.rs",
                "ui/surface/surface/default_interactions/keyboard.rs",
                "ui/surface/surface/default_interactions/timers.rs",
                "runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split",
            "runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions.rs",
            "ui/surface/surface/default_interactions/keyboard.rs",
            "ui/surface/surface/default_interactions/timers.rs",
            "runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners",
        ],
    );
}
