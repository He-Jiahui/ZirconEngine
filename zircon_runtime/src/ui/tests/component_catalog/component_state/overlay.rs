use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
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
