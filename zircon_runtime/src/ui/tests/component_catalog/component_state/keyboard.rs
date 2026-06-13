use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

#[test]
fn material_keyboard_action_activates_buttons_and_toggles_checked_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let button = registry.descriptor("Button").expect("Button descriptor");
    assert!(button.supports_event(UiComponentEventKind::KeyboardAction));
    let mut button_state = UiComponentState::new();
    button_state
        .apply_event(
            button,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        button_state.value("activated"),
        Some(&UiValue::Bool(true)),
        "semantic activate should deliver a retained activation value"
    );

    let checkbox = registry
        .descriptor("Checkbox")
        .expect("Checkbox descriptor");
    assert!(checkbox.supports_event(UiComponentEventKind::KeyboardAction));
    let mut checkbox_state = UiComponentState::new()
        .with_value("checked", UiValue::Bool(false))
        .with_value("indeterminate", UiValue::Bool(true));
    checkbox_state
        .apply_event(
            checkbox,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(checkbox_state.value("checked"), Some(&UiValue::Bool(true)));
    assert_eq!(
        checkbox_state.value("indeterminate"),
        Some(&UiValue::Bool(false))
    );
    assert!(checkbox_state.flags.checked);

    checkbox_state
        .apply_event(
            checkbox,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(checkbox_state.value("checked"), Some(&UiValue::Bool(false)));
    assert!(!checkbox_state.flags.checked);
}

#[test]
fn material_keyboard_action_moves_tabs_with_selection_following_focus() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tabs = registry.descriptor("Tabs").expect("Tabs descriptor");
    assert!(tabs.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(tabs.prop("selected_index").is_some());
    assert!(tabs.prop("focused_index").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("overview".to_string()),
                UiValue::String("details".to_string()),
                UiValue::String("stats".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("overview".to_string()))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("selection_follows_focus", UiValue::Bool(true));

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("details".to_string()))
    );
    assert!(state.flags.focused);

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("stats".to_string()))
    );

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("details".to_string()))
    );
}

#[test]
fn material_keyboard_action_moves_grouped_selection_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let radio_group = registry
        .descriptor("RadioGroup")
        .expect("RadioGroup descriptor");
    assert!(radio_group.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(radio_group.prop("selected_index").is_some());
    assert!(radio_group.prop("focused_index").is_some());

    let mut radio_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("editor".to_string()),
                UiValue::String("runtime".to_string()),
                UiValue::String("qa".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("editor".to_string()))
        .with_value("value_text", UiValue::String("editor".to_string()))
        .with_value("group_value", UiValue::String("editor".to_string()))
        .with_value("selected_index", UiValue::Int(0));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        radio_state.value("value"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert_eq!(
        radio_state.value("value_text"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert_eq!(
        radio_state.value("group_value"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert!(radio_state.flags.focused);
    assert!(radio_state.flags.selected);

    let toggle_group = registry
        .descriptor("ToggleButtonGroup")
        .expect("ToggleButtonGroup descriptor");
    assert!(toggle_group.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(toggle_group.prop("selected_index").is_some());
    assert!(toggle_group.prop("focused_index").is_some());

    let mut toggle_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("translate".to_string()),
                UiValue::String("rotate".to_string()),
                UiValue::String("scale".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("translate".to_string()))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("selection_state", UiValue::Enum("exclusive".to_string()))
        .with_value("selection_follows_focus", UiValue::Bool(true));

    toggle_state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(toggle_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(toggle_state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        toggle_state.value("value"),
        Some(&UiValue::String("scale".to_string()))
    );
    assert_eq!(
        toggle_state.value("value_text"),
        Some(&UiValue::String("scale".to_string()))
    );
    assert!(toggle_state.flags.focused);
    assert!(toggle_state.flags.selected);
}

#[test]
fn material_keyboard_action_skips_disabled_grouped_selection_options() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let radio_group = registry
        .descriptor("RadioGroup")
        .expect("RadioGroup descriptor");

    let mut radio_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("editor".to_string()),
                UiValue::String("runtime".to_string()),
                UiValue::String("qa".to_string()),
                UiValue::String("shipping".to_string()),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![
                UiValue::String("runtime".to_string()),
                UiValue::String("shipping".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("editor".to_string()))
        .with_value("value_text", UiValue::String("editor".to_string()))
        .with_value("group_value", UiValue::String("editor".to_string()))
        .with_value("selected_index", UiValue::Int(0));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        radio_state.value("value"),
        Some(&UiValue::String("qa".to_string()))
    );

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(
        radio_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "Last should stop on the last enabled option"
    );
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(2)));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(0)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(0)));
    assert_eq!(
        radio_state.value("group_value"),
        Some(&UiValue::String("editor".to_string()))
    );
}

#[test]
fn material_keyboard_action_toggles_multiple_toggle_button_group_focused_option() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let toggle_group = registry
        .descriptor("ToggleButtonGroup")
        .expect("ToggleButtonGroup descriptor");
    assert!(toggle_group.supports_event(UiComponentEventKind::KeyboardAction));

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("translate".to_string()),
                UiValue::String("rotate".to_string()),
                UiValue::String("scale".to_string()),
            ]),
        )
        .with_value(
            "value",
            UiValue::Array(vec![UiValue::Enum("translate".to_string())]),
        )
        .with_value("selected_index", UiValue::Int(0))
        .with_value("focused_index", UiValue::Int(1))
        .with_value("selection_state", UiValue::Enum("multiple".to_string()));

    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![
            UiValue::Enum("translate".to_string()),
            UiValue::Enum("rotate".to_string())
        ]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
    assert!(state.flags.focused);
    assert!(state.flags.selected);

    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum(
            "translate".to_string()
        )]))
    );
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
    assert!(!state.flags.selected);

    state = state
        .with_value("focused_index", UiValue::Int(2))
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("scale".to_string())]),
        );
    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum(
            "translate".to_string()
        )])),
        "disabled focused options must not mutate the multiple selection"
    );
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
}

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

#[test]
fn material_keyboard_text_appends_text_input_values_without_full_editing_policy() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let search = registry
        .descriptor("SearchField")
        .expect("SearchField descriptor");
    assert!(search.supports_event(UiComponentEventKind::KeyboardText));
    let mut search_state =
        UiComponentState::new().with_value("query", UiValue::String("sc".to_string()));
    search_state
        .apply_event(
            search,
            UiComponentEvent::KeyboardText {
                text: "ene".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        search_state.value("query"),
        Some(&UiValue::String("scene".to_string()))
    );
    assert!(search_state.flags.focused);

    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert!(text_field.supports_event(UiComponentEventKind::KeyboardText));
    let mut text_state =
        UiComponentState::new().with_value("value_text", UiValue::String("Mat".to_string()));
    text_state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "erial".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        text_state.value("value_text"),
        Some(&UiValue::String("Material".to_string()))
    );

    let input = registry.descriptor("Input").expect("Input descriptor");
    assert!(input.supports_event(UiComponentEventKind::KeyboardText));
    let mut input_state =
        UiComponentState::new().with_value("value_text", UiValue::String("UI".to_string()));
    input_state
        .apply_event(
            input,
            UiComponentEvent::KeyboardText {
                text: " Kit".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        input_state.value("value_text"),
        Some(&UiValue::String("UI Kit".to_string()))
    );
    assert_eq!(
        input_state.value("value"),
        Some(&UiValue::String("UI Kit".to_string())),
        "MUI text inputs keep value_text and value mirrored for render and schema consumers"
    );

    let textarea = registry
        .descriptor("TextareaAutosize")
        .expect("TextareaAutosize descriptor");
    let mut textarea_state =
        UiComponentState::new().with_value("value_text", UiValue::String("line".to_string()));
    textarea_state
        .apply_event(
            textarea,
            UiComponentEvent::KeyboardText {
                text: "\n 2".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        textarea_state.value("value_text"),
        Some(&UiValue::String("line 2".to_string())),
        "control characters are dropped, but printable spacing is preserved"
    );

    let source_editor = registry
        .descriptor("SourceEditor")
        .expect("SourceEditor descriptor");
    let mut source_state =
        UiComponentState::new().with_value("text", UiValue::String("let ".to_string()));
    source_state
        .apply_event(
            source_editor,
            UiComponentEvent::KeyboardText {
                text: "x".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        source_state.value("text"),
        Some(&UiValue::String("let x".to_string()))
    );

    source_state
        .apply_event(
            source_editor,
            UiComponentEvent::KeyboardText {
                text: "\t".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        source_state.value("text"),
        Some(&UiValue::String("let x".to_string())),
        "whitespace-only text payloads are not treated as editor text before the full plan-03 editing chain"
    );

    let mut readonly_state = UiComponentState::new()
        .with_value("value_text", UiValue::String("locked".to_string()))
        .with_value("readOnly", UiValue::Bool(true));
    readonly_state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "!".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        readonly_state.value("value_text"),
        Some(&UiValue::String("locked".to_string()))
    );
}

#[test]
fn material_keyboard_text_replaces_text_input_selection_and_updates_caret_state() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert!(text_field.supports_event(UiComponentEventKind::KeyboardText));
    assert!(text_field.prop("caret_offset").is_some());
    assert!(text_field.prop("selection_anchor").is_some());
    assert!(text_field.prop("selection_focus").is_some());

    let mut state = UiComponentState::new()
        .with_value("value_text", UiValue::String("abcd".to_string()))
        .with_value("caret_offset", UiValue::Int(3))
        .with_value("selection_anchor", UiValue::Int(1))
        .with_value("selection_focus", UiValue::Int(3));
    state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "X".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value_text"),
        Some(&UiValue::String("aXd".to_string()))
    );
    assert_eq!(state.value("caret_offset"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selection_anchor"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selection_focus"), Some(&UiValue::Int(2)));
    assert!(state.flags.focused);
}

#[test]
fn material_keyboard_action_steps_numeric_controls_and_closes_popup_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let number = registry
        .descriptor("NumberField")
        .expect("NumberField descriptor");
    assert!(number.supports_event(UiComponentEventKind::KeyboardAction));
    let mut number_state = UiComponentState::new()
        .with_value("value", UiValue::Float(10.0))
        .with_value("step", UiValue::Float(2.0))
        .with_value("large_step", UiValue::Float(5.0));
    number_state
        .apply_event(
            number,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(number_state.value("value"), Some(&UiValue::Float(12.0)));
    number_state
        .apply_event(
            number,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::LargeDecrement,
            },
        )
        .unwrap();
    assert_eq!(number_state.value("value"), Some(&UiValue::Float(7.0)));

    let range_slider = registry
        .descriptor("RangeSlider")
        .expect("RangeSlider descriptor");
    assert!(range_slider.supports_event(UiComponentEventKind::KeyboardAction));
    let mut range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(25.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("focused_thumb", UiValue::Enum("upper".to_string()));
    range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Decrement,
            },
        )
        .unwrap();
    assert_eq!(range_state.value("value"), Some(&UiValue::Float(20.0)));

    let select = registry.descriptor("Select").expect("Select descriptor");
    assert!(select.supports_event(UiComponentEventKind::KeyboardAction));
    let mut select_state = UiComponentState::new()
        .with_value("popup_open", UiValue::Bool(true))
        .with_value("open", UiValue::Bool(true));
    select_state.flags.popup_open = true;
    select_state
        .apply_event(
            select,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();
    assert!(!select_state.flags.popup_open);
    assert_eq!(
        select_state.value("popup_open"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(select_state.value("open"), Some(&UiValue::Bool(false)));
}

#[test]
fn material_keyboard_action_targets_range_slider_focused_thumb() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let range_slider = registry
        .descriptor("RangeSlider")
        .expect("RangeSlider descriptor");

    let mut lower_range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(80.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("large_step", UiValue::Float(100.0))
        .with_value("focused_thumb", UiValue::Enum("lower".to_string()));
    lower_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Decrement,
            },
        )
        .unwrap();
    assert_eq!(
        lower_range_state.value("range_min"),
        Some(&UiValue::Float(10.0))
    );
    assert_eq!(
        lower_range_state.value("value"),
        Some(&UiValue::Float(80.0))
    );
    lower_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::LargeIncrement,
            },
        )
        .unwrap();
    assert_eq!(
        lower_range_state.value("range_min"),
        Some(&UiValue::Float(80.0))
    );

    let mut upper_range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(80.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("focused_thumb", UiValue::Enum("upper".to_string()));
    upper_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(
        upper_range_state.value("range_min"),
        Some(&UiValue::Float(20.0))
    );
    assert_eq!(
        upper_range_state.value("value"),
        Some(&UiValue::Float(90.0))
    );
}

fn menu_option(id: &str, label: &str) -> UiValue {
    UiValue::Map(
        [
            ("id".to_string(), UiValue::String(id.to_string())),
            ("label".to_string(), UiValue::String(label.to_string())),
        ]
        .into_iter()
        .collect(),
    )
}
