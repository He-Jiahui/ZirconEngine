use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

#[test]
fn material_selection_popups_update_retained_popup_flags_through_public_reducer() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    for component_id in ["Select", "Autocomplete"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing {component_id} descriptor"));
        assert!(descriptor.supports_event(UiComponentEventKind::OpenPopup));
        assert!(descriptor.supports_event(UiComponentEventKind::ClosePopup));

        let mut state = UiComponentState::new();
        state
            .apply_event(descriptor, UiComponentEvent::OpenPopup)
            .unwrap();
        assert!(state.flags.popup_open, "{component_id} should open popup");

        state
            .apply_event(descriptor, UiComponentEvent::ClosePopup)
            .unwrap();
        assert!(!state.flags.popup_open, "{component_id} should close popup");
    }
}

#[test]
fn popup_anchor_events_record_pointer_anchor_through_public_reducer() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let menu = registry
        .descriptor("ContextActionMenu")
        .expect("ContextActionMenu descriptor");
    assert!(menu.supports_event(UiComponentEventKind::OpenPopupAt));

    let mut state = UiComponentState::new();
    state
        .apply_event(menu, UiComponentEvent::OpenPopupAt { x: 212.0, y: 96.0 })
        .unwrap();

    assert!(state.flags.popup_open);
    assert_eq!(state.value("popup_anchor_x"), Some(&UiValue::Float(212.0)));
    assert_eq!(state.value("popup_anchor_y"), Some(&UiValue::Float(96.0)));
}

#[test]
fn dialog_escape_dismisses_confirm_requires_action() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let dialog = registry.descriptor("Dialog").expect("Dialog descriptor");
    let confirm_dialog = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");
    assert!(dialog.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(confirm_dialog.supports_event(UiComponentEventKind::KeyboardAction));

    let mut dialog_state = UiComponentState::new()
        .with_value("open", UiValue::Bool(true))
        .with_value("popup_open", UiValue::Bool(true));
    dialog_state.flags.popup_open = true;
    dialog_state
        .apply_event(
            dialog,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();
    assert!(!dialog_state.flags.popup_open);
    assert_eq!(dialog_state.value("open"), Some(&UiValue::Bool(false)));
    assert_eq!(
        dialog_state.value("popup_open"),
        Some(&UiValue::Bool(false))
    );

    let mut confirm_state = UiComponentState::new()
        .with_value("open", UiValue::Bool(true))
        .with_value("popup_open", UiValue::Bool(true))
        .with_value("requires_explicit_action", UiValue::Bool(true))
        .with_value(
            "confirm_action_id",
            UiValue::String("delete-node".to_string()),
        )
        .with_value(
            "cancel_action_id",
            UiValue::String("cancel-delete-node".to_string()),
        );
    confirm_state.flags.popup_open = true;
    confirm_state
        .apply_event(
            confirm_dialog,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();
    assert!(
        confirm_state.flags.popup_open,
        "ConfirmDialog must ignore implicit Escape dismissal when explicit action is required"
    );
    assert_eq!(confirm_state.value("open"), Some(&UiValue::Bool(true)));
    assert_eq!(
        confirm_state.value("popup_open"),
        Some(&UiValue::Bool(true))
    );

    confirm_state
        .apply_event(
            confirm_dialog,
            UiComponentEvent::Commit {
                property: "dialog_action_id".to_string(),
                value: UiValue::String("cancel-delete-node".to_string()),
            },
        )
        .unwrap();
    assert!(!confirm_state.flags.popup_open);
    assert_eq!(confirm_state.value("open"), Some(&UiValue::Bool(false)));
    assert_eq!(
        confirm_state.value("popup_open"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(
        confirm_state.value("dialog_action_id"),
        Some(&UiValue::String("cancel-delete-node".to_string()))
    );
    assert_eq!(
        confirm_state.value("confirmed"),
        Some(&UiValue::Bool(false))
    );
}

#[test]
fn confirm_dialog_commit_records_confirm_action_when_enabled() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let confirm_dialog = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");

    let mut state = UiComponentState::new()
        .with_value("open", UiValue::Bool(true))
        .with_value("popup_open", UiValue::Bool(true))
        .with_value(
            "confirm_action_id",
            UiValue::String("delete-node".to_string()),
        )
        .with_value("confirm_enabled", UiValue::Bool(true));
    state.flags.popup_open = true;
    state
        .apply_event(
            confirm_dialog,
            UiComponentEvent::Commit {
                property: "dialog_action_id".to_string(),
                value: UiValue::String("delete-node".to_string()),
            },
        )
        .unwrap();

    assert!(!state.flags.popup_open);
    assert_eq!(
        state.value("dialog_action_id"),
        Some(&UiValue::String("delete-node".to_string()))
    );
    assert_eq!(state.value("confirmed"), Some(&UiValue::Bool(true)));
    assert_eq!(state.value("open"), Some(&UiValue::Bool(false)));
    assert_eq!(state.value("popup_open"), Some(&UiValue::Bool(false)));
}
