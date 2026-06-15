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
