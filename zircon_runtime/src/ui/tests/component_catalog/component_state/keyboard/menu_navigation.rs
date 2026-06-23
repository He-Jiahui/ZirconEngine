use super::*;

#[test]
fn material_keyboard_action_moves_menu_focus_without_committing_selection() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu_list = registry
        .descriptor("MenuList")
        .expect("MenuList descriptor");
    assert!(menu_list.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(menu_list.prop("focused_index").is_some());

    let mut menu_list_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("open".to_string()),
                UiValue::String("save".to_string()),
                UiValue::String("disabled".to_string()),
                UiValue::String("close".to_string()),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("disabled".to_string())]),
        )
        .with_value("focused_index", UiValue::Int(0))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("value", UiValue::String("open".to_string()));

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "MenuList should skip disabled items while moving focus"
    );
    assert_eq!(
        menu_list_state.value("selected_index"),
        Some(&UiValue::Int(0)),
        "MenuList focus movement must not commit selection"
    );
    assert_eq!(
        menu_list_state.value("value"),
        Some(&UiValue::String("open".to_string()))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "MenuList should wrap by default"
    );

    menu_list_state = menu_list_state
        .with_value("focused_index", UiValue::Int(3))
        .with_value("disableListWrap", UiValue::Bool(true));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "disableListWrap should stop at the current focusable item"
    );

    menu_list_state = menu_list_state
        .with_value("focused_index", UiValue::Int(1))
        .with_value("disabledItemsFocusable", UiValue::Bool(true));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "disabledItemsFocusable should allow focusing disabled menu items"
    );

    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert!(menu.supports_event(UiComponentEventKind::KeyboardAction));
    let mut menu_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("file".to_string()),
                UiValue::String("edit".to_string()),
                UiValue::String("view".to_string()),
            ]),
        )
        .with_value("focused_index", UiValue::Int(0));
    menu_state
        .apply_event(
            menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(menu_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert!(menu_state.value("selected_index").is_none());
    assert!(menu_state.flags.focused);
}

#[test]
fn material_keyboard_action_moves_tree_and_table_focus_by_index() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let tree = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert!(tree.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(tree.prop("focused_index").is_some());
    assert!(tree.prop("selected_index").is_some());

    let mut tree_state = UiComponentState::new()
        .with_value(
            "nodes",
            UiValue::Array(vec![
                menu_option("scene", "Scene"),
                menu_option("camera", "Camera"),
                menu_option("light", "Light"),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("camera".to_string())]),
        )
        .with_value("focused_index", UiValue::Int(0))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("selection_follows_focus", UiValue::Bool(true));

    tree_state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        tree_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "TreeView keyboard navigation should skip disabled nodes"
    );
    assert_eq!(tree_state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        tree_state.value("value"),
        Some(&UiValue::String("light".to_string()))
    );

    tree_state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();
    assert_eq!(
        tree_state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "TreeView keyboard navigation should wrap by default"
    );

    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert!(data_grid.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(data_grid.prop("focused_index").is_some());
    assert!(data_grid.prop("selected_index").is_some());

    let mut grid_state = UiComponentState::new()
        .with_value(
            "rows",
            UiValue::Array(vec![
                menu_option("imported", "Imported"),
                menu_option("processed", "Processed"),
                menu_option("failed", "Failed"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(1))
        .with_value("selected_index", UiValue::Int(1));

    grid_state
        .apply_event(
            data_grid,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(grid_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        grid_state.value("selected_index"),
        Some(&UiValue::Int(1)),
        "DataGrid focus movement should not commit selection unless opted in"
    );

    grid_state = grid_state.with_value("selection_follows_focus", UiValue::Bool(true));
    grid_state
        .apply_event(
            data_grid,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::First,
            },
        )
        .unwrap();
    assert_eq!(grid_state.value("focused_index"), Some(&UiValue::Int(0)));
    assert_eq!(grid_state.value("selected_index"), Some(&UiValue::Int(0)));
    assert_eq!(
        grid_state.value("value"),
        Some(&UiValue::String("imported".to_string()))
    );
}

#[test]
fn material_keyboard_text_moves_menu_focus_by_first_character_without_committing_selection() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu_list = registry
        .descriptor("MenuList")
        .expect("MenuList descriptor");
    assert!(menu_list.supports_event(UiComponentEventKind::KeyboardText));

    let mut menu_list_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
                menu_option("save", "Save All"),
                menu_option("delete", "Delete Selection"),
                menu_option("close", "Close View"),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("delete".to_string())]),
        )
        .with_value("focused_index", UiValue::Int(0))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("value", UiValue::String("open".to_string()));

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "c".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "first-letter search should use menu item labels"
    );
    assert_eq!(
        menu_list_state.value("selected_index"),
        Some(&UiValue::Int(0)),
        "menu typeahead focus movement must not commit selection"
    );
    assert_eq!(
        menu_list_state.value("value"),
        Some(&UiValue::String("open".to_string()))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "s".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "MenuList text search should wrap by default"
    );

    menu_list_state = menu_list_state
        .with_value("focused_index", UiValue::Int(3))
        .with_value("disableListWrap", UiValue::Bool(true));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "s".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "disableListWrap should also constrain text search"
    );

    menu_list_state = menu_list_state
        .with_value("focused_index", UiValue::Int(0))
        .with_value("disableListWrap", UiValue::Bool(false));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "d".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "disabled items should be skipped by text search"
    );

    menu_list_state = menu_list_state.with_value("disabledItemsFocusable", UiValue::Bool(true));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "D".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "disabledItemsFocusable should allow typeahead to focus disabled items"
    );

    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert!(menu.supports_event(UiComponentEventKind::KeyboardText));
    let mut menu_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("file", "File"),
                menu_option("edit", "Edit"),
                menu_option("view", "View"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(0));
    menu_state
        .apply_event(
            menu,
            UiComponentEvent::KeyboardText {
                text: "v".to_string(),
            },
        )
        .unwrap();
    assert_eq!(menu_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert!(menu_state.value("selected_index").is_none());
    assert!(menu_state.flags.focused);
}

#[test]
fn material_keyboard_text_matches_menu_prefix_without_committing_selection() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu_list = registry
        .descriptor("MenuList")
        .expect("MenuList descriptor");
    assert!(menu_list.supports_event(UiComponentEventKind::KeyboardText));

    let mut menu_list_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
                menu_option("save", "Save All"),
                menu_option("delete", "Delete Selection"),
                menu_option("close", "Close View"),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("delete".to_string())]),
        )
        .with_value("focused_index", UiValue::Int(0))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("value", UiValue::String("open".to_string()));

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "cl".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "multi-character menu text should match by label prefix"
    );
    assert_eq!(
        menu_list_state.value("selected_index"),
        Some(&UiValue::Int(0)),
        "prefix search should only move focus"
    );
    assert_eq!(
        menu_list_state.value("value"),
        Some(&UiValue::String("open".to_string()))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "SAVE".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "prefix search should be case-insensitive and wrap by default"
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "de".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "disabled menu options should be skipped by prefix search"
    );

    menu_list_state = menu_list_state.with_value("disabledItemsFocusable", UiValue::Bool(true));
    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "De".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "disabledItemsFocusable should allow prefix search to focus disabled menu items"
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: " \t ".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "blank keyboard text payloads should not move focus"
    );
}

#[test]
fn material_keyboard_text_buffers_menu_prefix_across_key_events_until_expired() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu_list = registry
        .descriptor("MenuList")
        .expect("MenuList descriptor");
    assert!(menu_list.prop("typeahead_buffer").is_some());
    assert!(menu_list.prop("typeahead_buffer_expired").is_some());
    assert!(menu_list.prop("typeahead_timeout_ms").is_some());
    assert!(menu_list.supports_event(UiComponentEventKind::TypeaheadExpired));

    let mut menu_list_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
                menu_option("close", "Close View"),
                menu_option("layer", "Layer Stack"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(0));

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "c".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1))
    );
    assert_eq!(
        menu_list_state.value("typeahead_buffer"),
        Some(&UiValue::String("c".to_string()))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "l".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "second key should extend the retained prefix to `cl`, not run a fresh `l` search"
    );
    assert_eq!(
        menu_list_state.value("typeahead_buffer"),
        Some(&UiValue::String("cl".to_string()))
    );

    menu_list_state
        .apply_event(menu_list, UiComponentEvent::TypeaheadExpired)
        .unwrap();
    assert_eq!(
        menu_list_state.value("typeahead_buffer"),
        Some(&UiValue::String("cl".to_string())),
        "expiration marks the buffer stale without destroying the last searchable prefix"
    );
    assert_eq!(
        menu_list_state.value("typeahead_buffer_expired"),
        Some(&UiValue::Bool(true))
    );

    menu_list_state
        .apply_event(
            menu_list,
            UiComponentEvent::KeyboardText {
                text: "l".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        menu_list_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "an expired buffer should reset before applying the next key"
    );
    assert_eq!(
        menu_list_state.value("typeahead_buffer"),
        Some(&UiValue::String("l".to_string()))
    );
    assert_eq!(
        menu_list_state.value("typeahead_buffer_expired"),
        Some(&UiValue::Bool(false))
    );
}
