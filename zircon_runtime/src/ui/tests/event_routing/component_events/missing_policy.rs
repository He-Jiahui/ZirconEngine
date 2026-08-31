use super::*;

#[test]
fn required_and_error_missing_value_policy_reject_atomically() {
    for (policy, expected_message) in [
        (
            UiBindingMissingValuePolicy::Required,
            "required value is missing",
        ),
        (UiBindingMissingValuePolicy::Error, "explicit error policy"),
    ] {
        let mut click = binding("Showcase/MissingRejected", UiEventKind::Click);
        click.targets = vec![target(
            UiBindingTarget::prop("text").with_missing_policy(policy),
            "prop.missing",
        )];
        let mut surface = bound_button_surface(vec![click]);

        let result = dispatch_primary_click(&mut surface);

        assert!(result.component_events.is_empty());
        assert_eq!(result.binding_reports.len(), 1);
        assert_eq!(result.binding_reports[0].rejected_count, 1);
        assert!(result.binding_reports[0].updates[0]
            .message
            .as_deref()
            .is_some_and(|message| message.contains(expected_message)));
    }
}

#[test]
fn optional_missing_value_policy_skips_only_the_unresolved_target() {
    let mut click = binding("Showcase/MissingOptional", UiEventKind::Click);
    click.targets = vec![
        target(
            UiBindingTarget::prop("text")
                .with_missing_policy(UiBindingMissingValuePolicy::Optional),
            "prop.missing",
        ),
        target(UiBindingTarget::class("resolved"), "true"),
    ];
    let mut surface = bound_button_surface(vec![click]);

    let result = dispatch_primary_click(&mut surface);

    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].rejected_count, 0);
    let transaction = result.binding_reports[0]
        .transaction
        .as_ref()
        .expect("resolved targets should still commit");
    assert_eq!(transaction.target_count, 1);
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert!(metadata.classes.iter().any(|class| class == "resolved"));
}

#[test]
fn default_and_fallback_missing_value_policy_publish_typed_values() {
    for (policy, expected) in [
        (
            UiBindingMissingValuePolicy::Default {
                value: UiValue::String("defaulted".to_string()),
            },
            "defaulted",
        ),
        (
            UiBindingMissingValuePolicy::Fallback {
                value: UiValue::String("fallback".to_string()),
            },
            "fallback",
        ),
    ] {
        let mut click = binding("Showcase/MissingSubstitute", UiEventKind::Click);
        click.targets = vec![target(
            UiBindingTarget::prop("text").with_missing_policy(policy),
            "prop.missing",
        )];
        let mut surface = bound_button_surface(vec![click]);

        let result = dispatch_primary_click(&mut surface);

        assert_eq!(result.component_events.len(), 1);
        assert_eq!(result.binding_reports[0].rejected_count, 0);
        assert!(result.binding_reports[0]
            .updates
            .iter()
            .any(|update| update.value == UiValue::String(expected.to_string())));
        let metadata = surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap();
        assert_eq!(
            metadata.attributes.get("text"),
            Some(&toml::Value::String(expected.to_string()))
        );
    }
}

#[test]
fn required_and_error_action_payload_missing_value_policy_suppress_publication() {
    for policy in [
        UiBindingMissingValuePolicy::Required,
        UiBindingMissingValuePolicy::Error,
    ] {
        let mut surface = bound_button_surface(vec![missing_payload_binding(policy)]);

        let result = dispatch_primary_click(&mut surface);

        assert_eq!(result.component_events.len(), 1);
        assert!(result.component_events[0].template_action.is_none());
        assert!(result.binding_reports.is_empty());
    }
}

#[test]
fn optional_action_payload_missing_value_policy_omits_only_the_field() {
    let mut surface = bound_button_surface(vec![missing_payload_binding(
        UiBindingMissingValuePolicy::Optional,
    )]);

    let result = dispatch_primary_click(&mut surface);

    let action = result.component_events[0]
        .template_action
        .as_ref()
        .expect("optional missing payload should retain the route");
    assert_eq!(action.route, "showcase.missing_payload");
    assert!(action.payload.is_empty());
}

#[test]
fn default_and_fallback_action_payload_missing_value_policy_publish_typed_values() {
    for (policy, expected) in [
        (
            UiBindingMissingValuePolicy::Default {
                value: UiValue::String("defaulted".to_string()),
            },
            "defaulted",
        ),
        (
            UiBindingMissingValuePolicy::Fallback {
                value: UiValue::String("fallback".to_string()),
            },
            "fallback",
        ),
    ] {
        let mut surface = bound_button_surface(vec![missing_payload_binding(policy)]);

        let result = dispatch_primary_click(&mut surface);

        assert_eq!(
            result.component_events[0]
                .template_action
                .as_ref()
                .and_then(|action| action.payload.get("status")),
            Some(&UiValue::String(expected.to_string()))
        );
    }
}

fn missing_payload_binding(policy: UiBindingMissingValuePolicy) -> UiBindingRef {
    let mut click = binding("Showcase/MissingPayload", UiEventKind::Click);
    click.action = Some(UiActionRef {
        route: Some("showcase.missing_payload".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "status".to_string(),
            toml::Value::String("=prop.missing".to_string()),
        )]),
        payload_missing_policy: policy,
    });
    click
}

fn dispatch_primary_click(
    surface: &mut crate::ui::surface::UiSurface,
) -> zircon_runtime_interface::ui::dispatch::UiPointerDispatchResult {
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap()
}
