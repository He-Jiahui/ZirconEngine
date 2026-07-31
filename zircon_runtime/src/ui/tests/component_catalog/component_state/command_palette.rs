use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventError, UiComponentEventKind, UiComponentKeyboardAction,
    UiComponentState, UiValue,
};

#[test]
fn command_palette_query_filters_commands_by_text_and_command_source() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let palette = registry
        .descriptor("CommandPalette")
        .expect("CommandPalette descriptor");
    assert!(palette.supports_event(UiComponentEventKind::ValueChanged));

    let mut state = UiComponentState::new()
        .with_value(
            "commands",
            UiValue::Array(vec![
                command("open_scene", "Open Scene", "workbench", "Ctrl+O"),
                command("build_project", "Build Project", "workbench", "Ctrl+B"),
                command("reload_runtime", "Reload Runtime", "runtime", "Ctrl+R"),
                UiValue::String("toggle_console|label=Toggle Console|shortcut=Ctrl+`".to_string()),
            ]),
        )
        .with_value("command_source", UiValue::String("workbench".to_string()))
        .with_value(
            "selected_command_id",
            UiValue::String("open_scene".to_string()),
        )
        .with_value("focused_index", UiValue::Int(0));

    state
        .apply_event(
            palette,
            UiComponentEvent::ValueChanged {
                property: "query".to_string(),
                value: UiValue::String("build".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_commands"),
        Some(&UiValue::Array(vec![UiValue::String(
            "build_project".to_string()
        )]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(0)));
    assert_eq!(
        state.value("selected_command_id"),
        Some(&UiValue::String("build_project".to_string()))
    );

    state
        .apply_event(
            palette,
            UiComponentEvent::ValueChanged {
                property: "query".to_string(),
                value: UiValue::String("reload".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("filtered_commands"),
        Some(&UiValue::Array(Vec::new()))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(-1)));
    assert_eq!(
        state.value("selected_command_id"),
        Some(&UiValue::String(String::new()))
    );
}

#[test]
fn command_palette_keyboard_text_updates_query_and_skips_disabled_focus() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let palette = registry
        .descriptor("CommandPalette")
        .expect("CommandPalette descriptor");
    assert!(palette.supports_event(UiComponentEventKind::KeyboardText));
    assert!(palette.supports_event(UiComponentEventKind::KeyboardAction));

    let mut state = UiComponentState::new()
        .with_value(
            "commands",
            UiValue::Array(vec![
                command("build_project", "Build Project", "workbench", "Ctrl+B"),
                command("build_assets", "Build Assets", "workbench", "Ctrl+Shift+B"),
                command("open_scene", "Open Scene", "workbench", "Ctrl+O"),
            ]),
        )
        .with_value(
            "disabled_commands",
            UiValue::Array(vec![UiValue::String("build_project".to_string())]),
        );

    state
        .apply_event(
            palette,
            UiComponentEvent::KeyboardText {
                text: "build".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("query"),
        Some(&UiValue::String("build".to_string()))
    );
    assert_eq!(
        state.value("filtered_commands"),
        Some(&UiValue::Array(vec![
            UiValue::String("build_project".to_string()),
            UiValue::String("build_assets".to_string()),
        ]))
    );
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "disabled commands remain visible but are skipped by roving focus"
    );
    assert_eq!(
        state.value("selected_command_id"),
        Some(&UiValue::String("build_assets".to_string()))
    );

    state
        .apply_event(
            palette,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(1)),
        "the only enabled filtered command should keep focus when navigation wraps"
    );
}

#[test]
fn command_palette_selects_enabled_command_and_rejects_disabled_command() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let palette = registry
        .descriptor("CommandPalette")
        .expect("CommandPalette descriptor");
    assert!(palette.supports_event(UiComponentEventKind::SelectOption));

    let mut state = UiComponentState::new()
        .with_value(
            "commands",
            UiValue::Array(vec![
                command("open_scene", "Open Scene", "workbench", "Ctrl+O"),
                command("delete_node", "Delete Node", "workbench", "Delete"),
            ]),
        )
        .with_value(
            "filtered_commands",
            UiValue::Array(vec![
                UiValue::String("open_scene".to_string()),
                UiValue::String("delete_node".to_string()),
            ]),
        )
        .with_value(
            "disabled_commands",
            UiValue::Array(vec![UiValue::String("delete_node".to_string())]),
        );

    state
        .apply_event(
            palette,
            UiComponentEvent::SelectOption {
                property: "selected_command_id".to_string(),
                option_id: "open_scene".to_string(),
                selected: true,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("selected_command_id"),
        Some(&UiValue::String("open_scene".to_string()))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(0)));
    assert!(state.flags.selected);

    let error = state
        .apply_event(
            palette,
            UiComponentEvent::SelectOption {
                property: "selected_command_id".to_string(),
                option_id: "delete_node".to_string(),
                selected: true,
            },
        )
        .unwrap_err();

    assert!(matches!(
        error,
        UiComponentEventError::DisabledOption {
            option_id,
            ..
        } if option_id == "delete_node"
    ));
    assert_eq!(
        state.value("selected_command_id"),
        Some(&UiValue::String("open_scene".to_string()))
    );
}

#[test]
fn command_palette_keyboard_requests_deep_windows_without_local_wrap() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let palette = registry
        .descriptor("CommandPalette")
        .expect("CommandPalette descriptor");

    for count in [1_usize, 12] {
        let mut state = command_palette_window_state(count, 0, count - 1, 7);
        state
            .apply_event(
                palette,
                UiComponentEvent::KeyboardAction {
                    action: UiComponentKeyboardAction::Next,
                },
            )
            .unwrap();

        assert_eq!(state.value("window_request_offset"), None);
        assert_eq!(
            state.value("focused_index"),
            Some(&UiValue::Int((count - 1) as i64))
        );
    }

    let mut thirteen = command_palette_window_state(13, 0, 11, 11);
    thirteen
        .apply_event(
            palette,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_window_request(&thirteen, 12, "first", 11);

    let mut thousand = command_palette_window_state(1_000, 0, 0, 19);
    thousand
        .apply_event(
            palette,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_window_request(&thousand, 996, "last", 19);
}

fn command_palette_window_state(
    total_count: usize,
    window_offset: usize,
    focused_index: usize,
    catalog_generation: i64,
) -> UiComponentState {
    let window_count = total_count.saturating_sub(window_offset).min(12);
    let commands = (0..window_count)
        .map(|index| {
            UiValue::String(format!(
                "command_{:04}|label=Command {:04}",
                window_offset + index,
                window_offset + index
            ))
        })
        .collect::<Vec<_>>();
    let selected = format!("command_{:04}", window_offset + focused_index);
    UiComponentState::new()
        .with_value("commands", UiValue::Array(commands))
        .with_value("selected_command_id", UiValue::String(selected))
        .with_value("focused_index", UiValue::Int(focused_index as i64))
        .with_value("catalog_generation", UiValue::Int(catalog_generation))
        .with_value("match_count", UiValue::Int(total_count as i64))
        .with_value("window_count", UiValue::Int(12))
        .with_value("window_offset", UiValue::Int(window_offset as i64))
}

fn assert_window_request(state: &UiComponentState, offset: i64, focus: &str, generation: i64) {
    assert_eq!(
        state.value("window_request_current_offset"),
        state.value("window_offset")
    );
    assert_eq!(
        state.value("window_request_offset"),
        Some(&UiValue::Int(offset))
    );
    assert_eq!(
        state.value("window_request_focus"),
        Some(&UiValue::String(focus.to_string()))
    );
    assert_eq!(
        state.value("window_request_generation"),
        Some(&UiValue::Int(generation))
    );
}

fn command(id: &str, label: &str, source: &str, shortcut: &str) -> UiValue {
    UiValue::Map(
        [
            ("id".to_string(), UiValue::String(id.to_string())),
            ("label".to_string(), UiValue::String(label.to_string())),
            ("source".to_string(), UiValue::String(source.to_string())),
            (
                "shortcut".to_string(),
                UiValue::String(shortcut.to_string()),
            ),
        ]
        .into_iter()
        .collect(),
    )
}
