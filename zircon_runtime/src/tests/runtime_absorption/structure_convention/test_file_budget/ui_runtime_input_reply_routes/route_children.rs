use super::*;

#[test]
fn runtime_15_ui_runtime_input_reply_route_children_are_folder_backed() {
    let keyboard_parent =
        read_runtime_src("ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs");
    let keyboard_directional = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs",
    );
    let keyboard_focus_path = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs",
    );
    let keyboard_semantic_actions = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs",
    );
    let keyboard_timers_disabled = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs",
    );
    let tree_parent =
        read_runtime_src("ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs");
    let tree_drag_reorder = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs",
    );
    let tree_selection = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs",
    );
    let tree_virtualization = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs",
    );

    assert_contains_all(
        "keyboard navigation reply-route parent mounts folder-backed children",
        &keyboard_parent,
        &[
            "mod directional;",
            "mod focus_path;",
            "mod semantic_actions;",
            "mod timers_disabled;",
            "fn horizontal_route_surface()",
            "fn keyboard_navigation_event(",
        ],
    );
    assert_contains_all(
        "tree-view pointer reply-route parent mounts folder-backed children",
        &tree_parent,
        &[
            "mod drag_reorder;",
            "mod selection;",
            "mod virtualization;",
            "fn dispatch_tree_pointer(",
            "fn tree_view_pointer_route_surface(",
        ],
    );
    assert_eq!(
        keyboard_parent.matches("#[test]").count(),
        0,
        "keyboard_navigation_routes.rs should only mount child test owners and shared helpers"
    );
    assert_eq!(
        tree_parent.matches("#[test]").count(),
        0,
        "tree_view_pointer_routes.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "unified_keyboard_tab_routes_to_navigation_next_from_focused_path",
        "unified_keyboard_arrow_right_prefers_tree_view_expand_keyboard_action_binding",
        "unified_keyboard_text_arms_typeahead_expiry_timer_and_tick_dispatches_event",
        "tree_view_primary_click_selects_clicked_item_on_owner",
        "tree_view_drag_release_reorders_nodes_and_emits_move_element",
        "tree_view_virtualized_reparent_drag_updates_window",
    ] {
        assert!(
            !keyboard_parent.contains(moved_test) && !tree_parent.contains(moved_test),
            "moved runtime input reply child test `{moved_test}` should not return to either parent"
        );
    }

    assert_contains_all(
        "keyboard focus-path child owns tab and vertical navigation tests",
        &keyboard_focus_path,
        &[
            "fn unified_keyboard_tab_routes_to_navigation_next_from_focused_path",
            "fn unified_keyboard_shift_tab_routes_to_navigation_previous_from_focused_path",
            "fn unified_keyboard_arrow_down_routes_to_directional_navigation_from_focused_path",
        ],
    );
    assert_contains_all(
        "keyboard semantic-actions child owns component-specific keyboard bindings",
        &keyboard_semantic_actions,
        &[
            "fn unified_keyboard_arrow_right_prefers_semantic_tabs_keyboard_action_binding",
            "fn unified_keyboard_arrow_right_prefers_tree_view_expand_keyboard_action_binding",
            "fn unified_keyboard_f2_prefers_tree_view_begin_edit_keyboard_action_binding",
        ],
    );
    assert_contains_all(
        "keyboard timers-disabled child owns typeahead, submenu, toast, and disabled gates",
        &keyboard_timers_disabled,
        &[
            "fn unified_keyboard_text_arms_typeahead_expiry_timer_and_tick_dispatches_event",
            "fn submenu_hover_timer_dispatches_ready_value_changed_event",
            "fn unified_keyboard_printable_text_respects_disabled_component_gate",
        ],
    );
    assert_contains_all(
        "keyboard directional child owns remaining directional navigation tests",
        &keyboard_directional,
        &[
            "fn unified_keyboard_arrow_up_routes_to_directional_navigation_from_focused_path",
            "fn unified_keyboard_arrow_right_routes_to_directional_navigation_from_horizontal_focused_path",
            "fn unified_keyboard_arrow_left_routes_to_directional_navigation_from_horizontal_focused_path",
        ],
    );
    assert_contains_all(
        "tree-view selection child owns click selection tests",
        &tree_selection,
        &[
            "fn tree_view_primary_click_selects_clicked_item_on_owner",
            "fn tree_view_control_click_toggles_item_in_multi_selection",
            "fn material_tree_view_secondary_release_begins_context_rename_for_clicked_item",
        ],
    );
    assert_contains_all(
        "tree-view drag child owns drag and reorder tests",
        &tree_drag_reorder,
        &[
            "fn tree_view_drag_release_reorders_nodes_and_emits_move_element",
            "fn material_tree_view_items_reordering_reorders_items_array",
        ],
    );
    assert_contains_all(
        "tree-view virtualization child owns scroll and virtualized drag tests",
        &tree_virtualization,
        &[
            "fn tree_view_scroll_updates_virtual_window_and_emits_visible_range",
            "fn tree_view_virtualized_reparent_drag_updates_window",
        ],
    );

    let keyboard_test_total = [
        keyboard_directional.as_str(),
        keyboard_focus_path.as_str(),
        keyboard_semantic_actions.as_str(),
        keyboard_timers_disabled.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        keyboard_test_total, 15,
        "keyboard navigation route children should preserve all 15 parent tests"
    );
    let tree_test_total = [
        tree_drag_reorder.as_str(),
        tree_selection.as_str(),
        tree_virtualization.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        tree_test_total, 9,
        "tree-view pointer route children should preserve all 9 parent tests"
    );

    for (path, source) in [
        (
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs",
            keyboard_parent.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs",
            keyboard_directional.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs",
            keyboard_focus_path.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs",
            keyboard_semantic_actions.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs",
            keyboard_timers_disabled.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs",
            tree_parent.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs",
            tree_drag_reorder.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs",
            tree_selection.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs",
            tree_virtualization.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI runtime input reply route child folder split",
                "runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs",
                "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs",
                "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs",
                "runtime_15_ui_runtime_input_reply_route_children_are_folder_backed",
            ],
        );
    }
}
