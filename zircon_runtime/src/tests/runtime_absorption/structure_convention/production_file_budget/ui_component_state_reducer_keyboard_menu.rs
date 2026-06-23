use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner() {
    let parent = read_runtime_src("ui/component/state_reducer/keyboard/menu.rs");
    let submenu = read_runtime_src("ui/component/state_reducer/keyboard/menu/submenu.rs");
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
        "keyboard menu parent keeps typeahead/search-filter entry points and child mount",
        &parent,
        &[
            "mod submenu;",
            "pub(super) use submenu::{close_active_submenu, open_focused_submenu};",
            "pub(super) fn apply_keyboard_text(",
            "pub(super) fn apply_typeahead_expired(",
            "pub(super) fn sync_search_filter(",
            "pub(super) fn option_is_hidden_by_search_filter(",
            "fn sync_search_filter_state(",
            "fn menu_typeahead_searches(",
            "fn menu_search_options(",
            "fn recursive_search_filter(",
            "submenu::sync_submenu_state(",
        ],
    );
    for moved_owner in [
        "const MENU_SUBMENU_ACTIVE_PARENT_INDEX",
        "const MENU_SUBMENU_OPEN_OPTION_ID",
        "fn sync_submenu_state(",
        "fn open_submenu_for_option_id(",
        "fn clear_submenu_state(",
        "fn submenu_target_for_option_id(",
        "fn write_submenu_string(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "keyboard/menu.rs should delegate submenu state owner `{moved_owner}` to submenu.rs"
        );
    }

    assert_contains_all(
        "submenu child owns submenu state transitions and target lookup",
        &submenu,
        &[
            "const MENU_SUBMENU_ACTIVE_PARENT_INDEX",
            "const MENU_SUBMENU_FOCUS_SCOPE_SUBMENU",
            "const MENU_SUBMENU_OPEN_OPTION_ID",
            "pub(in crate::ui::component::state_reducer::keyboard) fn open_focused_submenu(",
            "pub(in crate::ui::component::state_reducer::keyboard) fn close_active_submenu(",
            "pub(super) fn sync_submenu_state(",
            "pub(super) fn is_submenu_state_property(",
            "fn sync_hovered_submenu_option(",
            "fn promote_pending_submenu_if_ready(",
            "fn prune_invalid_submenu_state(",
            "fn open_submenu_for_option_id(",
            "fn clear_submenu_state(",
            "fn submenu_target_for_option_id(",
            "fn write_submenu_string(",
            "super::menu_search_options(",
        ],
    );

    for (path, source) in [
        (
            "ui/component/state_reducer/keyboard/menu.rs",
            parent.as_str(),
        ),
        (
            "ui/component/state_reducer/keyboard/menu/submenu.rs",
            submenu.as_str(),
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
                "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split",
                "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred",
                "ui/component/state_reducer/keyboard/menu.rs",
                "ui/component/state_reducer/keyboard/menu/submenu.rs",
                "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split",
            "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred",
            "ui/component/state_reducer/keyboard/menu.rs",
            "ui/component/state_reducer/keyboard/menu/submenu.rs",
            "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner",
        ],
    );
}
