use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventError, UiComponentEventKind, UiComponentKeyboardAction,
    UiComponentState, UiValue,
};

#[test]
fn notification_center_selects_notification_and_marks_it_read() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let notification_center = registry
        .descriptor("NotificationCenter")
        .expect("NotificationCenter descriptor");
    assert!(notification_center.supports_event(UiComponentEventKind::SelectOption));

    let mut state = UiComponentState::new()
        .with_value("notifications", notification_entries())
        .with_value("unread_count", UiValue::Int(2))
        .with_value("focused_index", UiValue::Int(0));

    state
        .apply_event(
            notification_center,
            UiComponentEvent::SelectOption {
                property: "selected_notification_id".to_string(),
                option_id: "asset".to_string(),
                selected: true,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("selected_notification_id"),
        Some(&UiValue::String("asset".to_string()))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("unread_count"), Some(&UiValue::Int(1)));
    assert!(state.flags.selected);
    assert_notification_unread(&state, "asset", false);

    let error = state
        .apply_event(
            notification_center,
            UiComponentEvent::SelectOption {
                property: "selected_notification_id".to_string(),
                option_id: "source".to_string(),
                selected: true,
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        UiComponentEventError::DisabledOption { option_id, .. } if option_id == "source"
    ));
    assert_eq!(
        state.value("selected_notification_id"),
        Some(&UiValue::String("asset".to_string()))
    );
}

#[test]
fn notification_center_keyboard_navigation_focuses_and_activates_rows() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let notification_center = registry
        .descriptor("NotificationCenter")
        .expect("NotificationCenter descriptor");
    assert!(notification_center.supports_event(UiComponentEventKind::KeyboardAction));

    let mut state = UiComponentState::new()
        .with_value("notifications", keyboard_navigation_entries())
        .with_value("focused_index", UiValue::Int(-1))
        .with_value("keyboard_navigation", UiValue::Bool(true));

    state
        .apply_event(
            notification_center,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("unread_count"), Some(&UiValue::Int(2)));

    state
        .apply_event(
            notification_center,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("focused_index"),
        Some(&UiValue::Int(0)),
        "disabled notification rows should be skipped when they are the only later row"
    );

    state
        .apply_event(
            notification_center,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("selected_notification_id"),
        Some(&UiValue::String("build".to_string()))
    );
    assert_eq!(state.value("unread_count"), Some(&UiValue::Int(1)));
    assert_notification_unread(&state, "build", false);
}

#[test]
fn notification_center_value_changes_resync_unread_and_clear_stale_selection() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let notification_center = registry
        .descriptor("NotificationCenter")
        .expect("NotificationCenter descriptor");

    let mut state = UiComponentState::new()
        .with_value(
            "selected_notification_id",
            UiValue::String("missing".to_string()),
        )
        .with_value("focused_index", UiValue::Int(6));

    state
        .apply_event(
            notification_center,
            UiComponentEvent::ValueChanged {
                property: "notifications".to_string(),
                value: notification_entries(),
            },
        )
        .unwrap();

    assert_eq!(state.value("unread_count"), Some(&UiValue::Int(2)));
    assert_eq!(
        state.value("selected_notification_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(0)));
    assert!(!state.flags.selected);
}

fn notification_entries() -> UiValue {
    UiValue::Array(vec![
        notification("build", "Build failed", "Shader compile error", true, false),
        notification(
            "asset",
            "Asset import complete",
            "StoneWall.mesh ready",
            true,
            false,
        ),
        notification(
            "source",
            "Source control synced",
            "No local conflicts",
            false,
            true,
        ),
    ])
}

fn keyboard_navigation_entries() -> UiValue {
    UiValue::Array(vec![
        notification("build", "Build failed", "Shader compile error", true, false),
        notification(
            "source",
            "Source control synced",
            "No local conflicts",
            true,
            true,
        ),
    ])
}

fn notification(id: &str, title: &str, message: &str, unread: bool, disabled: bool) -> UiValue {
    UiValue::Map(
        [
            ("id".to_string(), UiValue::String(id.to_string())),
            ("title".to_string(), UiValue::String(title.to_string())),
            ("message".to_string(), UiValue::String(message.to_string())),
            ("unread".to_string(), UiValue::Bool(unread)),
            ("disabled".to_string(), UiValue::Bool(disabled)),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert_notification_unread(state: &UiComponentState, id: &str, expected: bool) {
    let Some(UiValue::Array(entries)) = state.value("notifications") else {
        panic!("notifications should be retained as an array");
    };
    let entry = entries
        .iter()
        .find_map(|value| match value {
            UiValue::Map(values) => {
                let entry_id = values.get("id").and_then(string_value)?;
                (entry_id == id).then_some(values)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing notification `{id}`"));
    assert_eq!(entry.get("unread"), Some(&UiValue::Bool(expected)));
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}
