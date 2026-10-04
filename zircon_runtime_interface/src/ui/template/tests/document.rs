use super::*;

#[test]
fn binding_mode_contract_serializes_trigger_timing_and_write_permissions() {
    let cases = [
        (
            "OneTime",
            UiBindingMode::OneTime,
            UiBindingTriggerTiming::Instantiation,
            UiBindingWritePermissions::TARGET_ONLY,
        ),
        (
            "OneWay",
            UiBindingMode::OneWay,
            UiBindingTriggerTiming::SourceChange,
            UiBindingWritePermissions::TARGET_ONLY,
        ),
        (
            "TwoWay",
            UiBindingMode::TwoWay,
            UiBindingTriggerTiming::SourceOrTargetChange,
            UiBindingWritePermissions::SOURCE_AND_TARGET,
        ),
        (
            "Event",
            UiBindingMode::Event,
            UiBindingTriggerTiming::EventDispatch,
            UiBindingWritePermissions::TARGET_ONLY,
        ),
        (
            "Command",
            UiBindingMode::Command,
            UiBindingTriggerTiming::CommandDispatch,
            UiBindingWritePermissions::COMMAND_ONLY,
        ),
    ];

    for (serialized_mode, expected_mode, expected_trigger, expected_permissions) in cases {
        let binding: UiBindingRef = toml::from_str(&format!(
            "id = \"mode.contract\"\nevent = \"Click\"\nmode = \"{serialized_mode}\"\n"
        ))
        .unwrap();

        assert_eq!(binding.mode, expected_mode);
        assert_eq!(binding.mode.trigger_timing(), expected_trigger);
        assert_eq!(binding.mode.write_permissions(), expected_permissions);
        assert!(toml::to_string(&binding)
            .unwrap()
            .contains(&format!("mode = \"{serialized_mode}\"")));
    }

    let legacy: UiBindingRef = toml::from_str("id = \"legacy\"\nevent = \"Click\"\n").unwrap();
    assert_eq!(legacy.mode, UiBindingMode::Event);
}

#[test]
fn typed_component_event_serde_round_trips_declared_identity() {
    let binding: UiBindingRef = toml::from_str(
        r#"
id = "product.lower_snake"
event = "Click"
component_event = "OpenPopup"
route = "component_lab.open_popup.product"
"#,
    )
    .expect("typed component event should deserialize");

    assert_eq!(
        binding.component_event,
        Some(UiComponentEventKind::OpenPopup)
    );
    let serialized = toml::to_string(&binding).expect("typed component event should serialize");
    assert!(serialized.contains("component_event = \"OpenPopup\""));

    let legacy: UiBindingRef = toml::from_str("id = \"legacy\"\nevent = \"Click\"\n").unwrap();
    assert_eq!(legacy.component_event, None);
}
