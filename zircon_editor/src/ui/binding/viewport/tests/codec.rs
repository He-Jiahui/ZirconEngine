use super::*;

#[test]
fn overlay_provider_toggle_round_trips_through_the_viewport_codec() {
    let command = ViewportCommand::ToggleOverlayProvider {
        provider_id: "weather.viewport.overlay.provider".to_string(),
    };

    assert_eq!(
        ViewportCommand::from_call(command.to_call()).unwrap(),
        Some(command)
    );
}

#[test]
fn cancel_interaction_round_trips_through_the_viewport_codec() {
    let command = ViewportCommand::CancelInteraction;

    assert_eq!(
        ViewportCommand::from_call(command.to_call()).unwrap(),
        Some(command)
    );
}

#[test]
fn pivot_mode_round_trips_through_the_viewport_codec() {
    let command = ViewportCommand::SetPivotMode(PivotMode::Primary);

    assert_eq!(
        ViewportCommand::from_call(command.to_call()).unwrap(),
        Some(command)
    );
}

#[test]
fn custom_activation_cannot_encode_a_reserved_builtin_mode_id() {
    let call = UiBindingCall::new("ViewportCommand.ActivateSceneMode")
        .with_argument(UiBindingValue::string("Custom:scene.select"));

    assert!(ViewportCommand::from_call(call).is_err());
}
