use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

#[test]
fn menu_search_query_filters_options_and_moves_focus_to_first_visible_match() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu_list = registry
        .descriptor("MenuList")
        .expect("MenuList descriptor");
    assert!(menu_list.supports_event(UiComponentEventKind::ValueChanged));
    assert!(menu_list.prop("search_query").is_some());
    assert!(menu_list.prop("filtered_option_ids").is_some());
    assert!(menu_list.prop("filter_no_results").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
                menu_option("save", "Save All"),
                menu_option("delete", "Delete Selection"),
                menu_option("close", "Close View"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(0));

    state
        .apply_event(
            menu_list,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String("cl".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("search_query"),
        Some(&UiValue::String("cl".to_string()))
    );
    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(vec![UiValue::String("close".to_string())]))
    );
    assert_eq!(
        state.value("filter_no_results"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(3)),
        "MenuList search filtering should focus the visible result by full option index"
    );

    state
        .apply_event(
            menu_list,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String("zz".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(Vec::new()))
    );
    assert_eq!(state.value("filter_no_results"), Some(&UiValue::Bool(true)));
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(-1)));

    state
        .apply_event(
            menu_list,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String(" ".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(vec![
            UiValue::String("open".to_string()),
            UiValue::String("save".to_string()),
            UiValue::String("delete".to_string()),
            UiValue::String("close".to_string()),
        ]))
    );
    assert_eq!(
        state.value("filter_no_results"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "clearing search should restore focus to the first visible option"
    );
}

#[test]
fn menu_search_query_keeps_disabled_matches_hidden_from_focus_navigation() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert!(menu.supports_event(UiComponentEventKind::ValueChanged));
    assert!(menu.prop("allow_search").is_some());
    assert!(menu.prop("search_bar_enabled_on_item_count").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
                menu_option("delete", "Delete Selection"),
                menu_option("close", "Close View"),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("delete".to_string())]),
        )
        .with_value("focused_index", UiValue::Int(0));

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String("delete".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(vec![UiValue::String("delete".to_string())]))
    );
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(-1)),
        "disabledItemsFocusable=false should keep the sole disabled match unfocused"
    );
}

#[test]
fn menu_search_query_retains_submenu_path_for_child_match() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu = registry.descriptor("Menu").expect("Menu descriptor");

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option_with_children(
                    "file",
                    "File",
                    vec![
                        menu_option("new_scene", "New Scene"),
                        menu_option("open_recent", "Open Recent Project"),
                    ],
                ),
                menu_option_with_children("edit", "Edit", vec![menu_option("undo", "Undo")]),
                menu_option("view", "View"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(2));

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String("recent".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(vec![
            UiValue::String("file".to_string()),
            UiValue::String("open_recent".to_string()),
        ])),
        "child submenu matches should retain the visible parent path"
    );
    assert_eq!(
        state.value("filter_no_results"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "root menu focus should move to the parent row that can open the matching submenu"
    );

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "search_query".to_string(),
                value: UiValue::String("undo".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_option_ids"),
        Some(&UiValue::Array(vec![
            UiValue::String("edit".to_string()),
            UiValue::String("undo".to_string()),
        ]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
}

#[test]
fn menu_hovered_submenu_option_waits_for_hover_ready_before_opening() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert!(menu.prop("hovered_option_id").is_some());
    assert!(menu.prop("submenu_pending_option_id").is_some());
    assert!(menu.prop("submenu_open_option_id").is_some());
    assert!(menu.prop("submenu_hover_delay_ms").is_some());

    let mut state = UiComponentState::new().with_value(
        "options",
        UiValue::Array(vec![
            menu_option_with_children(
                "file",
                "File",
                vec![
                    menu_option("new_scene", "New Scene"),
                    menu_option("open_recent", "Open Recent Project"),
                ],
            ),
            menu_option("edit", "Edit"),
        ]),
    );

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "hovered_option_id".to_string(),
                value: UiValue::String("file".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("submenu_pending_option_id"),
        Some(&UiValue::String("file".to_string()))
    );
    assert_eq!(
        state.value("submenu_open_option_id"),
        Some(&UiValue::String(String::new())),
        "hovering a submenu row should arm it before opening"
    );
    assert_eq!(
        state.value("submenu_hover_ready"),
        Some(&UiValue::Bool(false))
    );

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "submenu_hover_ready".to_string(),
                value: UiValue::Bool(true),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("submenu_pending_option_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(
        state.value("submenu_open_option_id"),
        Some(&UiValue::String("file".to_string()))
    );
    assert_eq!(
        state.value("submenu_focus_scope"),
        Some(&UiValue::String("submenu".to_string()))
    );
    assert_eq!(
        state.value("submenu_active_parent_index"),
        Some(&UiValue::Int(0))
    );

    state
        .apply_event(
            menu,
            UiComponentEvent::ValueChanged {
                property: "hovered_option_id".to_string(),
                value: UiValue::String("edit".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("submenu_open_option_id"),
        Some(&UiValue::String(String::new())),
        "hovering a leaf row should close the active submenu loop"
    );
    assert_eq!(
        state.value("submenu_focus_scope"),
        Some(&UiValue::String("root".to_string()))
    );
}

#[test]
fn menu_keyboard_activate_and_cancel_cycle_submenu_focus_scope() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert!(menu.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(menu.prop("submenu_focus_scope").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option_with_children(
                    "file",
                    "File",
                    vec![menu_option("new_scene", "New Scene")],
                ),
                menu_option("edit", "Edit"),
            ]),
        )
        .with_value("focused_index", UiValue::Int(0))
        .with_value("open", UiValue::Bool(true))
        .with_value("popup_open", UiValue::Bool(true));

    state
        .apply_event(
            menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("submenu_open_option_id"),
        Some(&UiValue::String("file".to_string()))
    );
    assert_eq!(
        state.value("submenu_focus_scope"),
        Some(&UiValue::String("submenu".to_string()))
    );
    assert_eq!(
        state.value("submenu_active_parent_index"),
        Some(&UiValue::Int(0))
    );
    assert_eq!(state.value("open"), Some(&UiValue::Bool(true)));
    assert_eq!(state.value("popup_open"), Some(&UiValue::Bool(true)));

    state
        .apply_event(
            menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("submenu_open_option_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(
        state.value("submenu_focus_scope"),
        Some(&UiValue::String("root".to_string()))
    );
    assert_eq!(
        state.value("open"),
        Some(&UiValue::Bool(true)),
        "first cancel should return focus to the root menu before closing the popup"
    );
    assert_eq!(state.value("popup_open"), Some(&UiValue::Bool(true)));

    state
        .apply_event(
            menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();

    assert_eq!(state.value("open"), Some(&UiValue::Bool(false)));
    assert_eq!(state.value("popup_open"), Some(&UiValue::Bool(false)));
}

#[test]
fn context_menu_keyboard_navigation_reuses_menu_option_focus_rules() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let context_menu = registry
        .descriptor("ContextMenu")
        .expect("ContextMenu descriptor");
    assert!(context_menu.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(context_menu.prop("focused_index").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("open", "Open Scene"),
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

    state
        .apply_event(
            context_menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "ContextMenu should skip disabled popup options while moving focus"
    );
    assert_eq!(
        state.value("selected_index"),
        Some(&UiValue::Int(0)),
        "popup option focus movement must not commit selection"
    );
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("open".to_string()))
    );

    state
        .apply_event(
            context_menu,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "ContextMenu should wrap through focusable popup options by default"
    );
}

#[test]
fn dropdown_popup_keyboard_text_reuses_menu_typeahead_focus_rules() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let dropdown_popup = registry
        .descriptor("DropdownPopup")
        .expect("DropdownPopup descriptor");
    assert!(dropdown_popup.supports_event(UiComponentEventKind::KeyboardText));
    assert!(dropdown_popup.supports_event(UiComponentEventKind::TypeaheadExpired));
    assert!(dropdown_popup.prop("typeahead_buffer").is_some());

    let selected = UiValue::Array(vec![UiValue::String("atlas".to_string())]);
    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                menu_option("atlas", "Atlas"),
                menu_option("asset", "Asset"),
                menu_option("archive", "Archive"),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("archive".to_string())]),
        )
        .with_value("selected_options", selected.clone())
        .with_value("focused_index", UiValue::Int(0));

    state
        .apply_event(
            dropdown_popup,
            UiComponentEvent::KeyboardText {
                text: "as".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "DropdownPopup typeahead should move roving focus by option label"
    );
    assert_eq!(
        state.value("selected_options"),
        Some(&selected),
        "typeahead focus movement must not commit dropdown selection"
    );
    assert_eq!(
        state.value("typeahead_buffer"),
        Some(&UiValue::String("as".to_string()))
    );

    state
        .apply_event(dropdown_popup, UiComponentEvent::TypeaheadExpired)
        .unwrap();
    assert_eq!(
        state.value("typeahead_buffer_expired"),
        Some(&UiValue::Bool(true))
    );

    state
        .apply_event(
            dropdown_popup,
            UiComponentEvent::KeyboardText {
                text: "a".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "expired popup typeahead should reset before applying the next key"
    );
    assert_eq!(
        state.value("typeahead_buffer"),
        Some(&UiValue::String("a".to_string()))
    );
    assert_eq!(
        state.value("typeahead_buffer_expired"),
        Some(&UiValue::Bool(false))
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

fn menu_option_with_children(id: &str, label: &str, children: Vec<UiValue>) -> UiValue {
    UiValue::Map(
        [
            ("id".to_string(), UiValue::String(id.to_string())),
            ("label".to_string(), UiValue::String(label.to_string())),
            ("children".to_string(), UiValue::Array(children)),
        ]
        .into_iter()
        .collect(),
    )
}
