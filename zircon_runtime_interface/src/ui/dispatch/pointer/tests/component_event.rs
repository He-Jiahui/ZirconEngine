use super::*;

#[test]
fn template_action_round_trips_with_typed_object_payload() {
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("test.template.action"),
        UiNodeId::new(7),
        "BakeSelected",
        "BakeSelected/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_template_action(UiTemplateActionInvocation::route(
        "navigation.bake.surface",
        BTreeMap::from([
            ("surface_entity".to_string(), UiValue::Int(73)),
            ("force_full_rebuild".to_string(), UiValue::Bool(true)),
        ]),
    ));

    let encoded = serde_json::to_value(&event).expect("pointer event should serialize");
    assert_eq!(encoded["template_action"]["target"]["kind"], "route");
    assert_eq!(
        encoded["template_action"]["target"]["id"],
        "navigation.bake.surface"
    );
    assert_eq!(
        encoded["template_action"]["payload"]["surface_entity"],
        serde_json::json!({ "Int": 73 })
    );
    assert_eq!(
        encoded["template_action"]["payload"]["force_full_rebuild"],
        serde_json::json!({ "Bool": true })
    );

    let restored: UiPointerComponentEvent =
        serde_json::from_value(encoded).expect("pointer event should deserialize");
    assert_eq!(restored.template_action, event.template_action);
}

#[test]
fn template_action_is_backward_compatible_when_absent() {
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("test.template.action.legacy"),
        UiNodeId::new(8),
        "LegacyButton",
        "LegacyButton/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    );

    let encoded = serde_json::to_value(&event).expect("legacy pointer event should serialize");
    assert!(encoded.get("template_action").is_none());
    let restored: UiPointerComponentEvent =
        serde_json::from_value(encoded).expect("legacy pointer event should deserialize");
    assert_eq!(restored.template_action, None);
}

#[test]
fn template_action_round_trip_preserves_authored_action_identity() {
    let action = UiTemplateActionInvocation::action("view.console.clear");

    let encoded = serde_json::to_value(&action).expect("template action should serialize");
    assert_eq!(encoded["target"]["kind"], "action");
    assert_eq!(encoded["target"]["id"], "view.console.clear");
    assert!(encoded.get("payload").is_none());

    let restored: UiTemplateActionInvocation =
        serde_json::from_value(encoded).expect("template action should deserialize");
    assert_eq!(restored, action);
}
