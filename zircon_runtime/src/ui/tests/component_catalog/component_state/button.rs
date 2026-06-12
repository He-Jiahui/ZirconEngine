use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
};

#[test]
fn material_button_family_applies_interaction_flags_through_public_reducer() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    for component_id in ["Button", "IconButton", "FloatingActionButton"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing {component_id} descriptor"));
        for event in [
            UiComponentEventKind::Focus,
            UiComponentEventKind::Hover,
            UiComponentEventKind::Press,
            UiComponentEventKind::Commit,
        ] {
            assert!(
                descriptor.supports_event(event),
                "{component_id} should expose button-family {event:?} events"
            );
        }

        let mut state = UiComponentState::new();
        state
            .apply_event(descriptor, UiComponentEvent::Focus { focused: true })
            .unwrap();
        state
            .apply_event(descriptor, UiComponentEvent::Hover { hovered: true })
            .unwrap();
        state
            .apply_event(descriptor, UiComponentEvent::Press { pressed: true })
            .unwrap();

        assert!(state.flags.focused, "{component_id} should retain focus");
        assert!(state.flags.hovered, "{component_id} should retain hover");
        assert!(state.flags.pressed, "{component_id} should retain press");

        state
            .apply_event(descriptor, UiComponentEvent::Hover { hovered: false })
            .unwrap();
        state
            .apply_event(descriptor, UiComponentEvent::Press { pressed: false })
            .unwrap();

        assert!(!state.flags.hovered, "{component_id} should clear hover");
        assert!(!state.flags.pressed, "{component_id} should clear press");
    }
}

#[test]
fn material_button_family_preserves_commit_value_delivery() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let button = registry.descriptor("Button").expect("Button descriptor");
    let mut state = UiComponentState::new();

    state
        .apply_event(
            button,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            },
        )
        .unwrap();

    assert_eq!(state.value("activated"), Some(&UiValue::Bool(true)));
}
