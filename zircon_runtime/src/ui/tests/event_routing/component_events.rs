use super::*;
use std::collections::BTreeMap;
use zircon_runtime_interface::ui::binding::{UiBindingDirtyDomain, UiBindingMutationOutcome};
use zircon_runtime_interface::ui::template::{
    UiActionRef, UiBindingMissingValuePolicy, UiBindingTarget, UiBindingTargetAssignment,
};

mod missing_policy;
mod performance;

#[test]
fn click_component_events_preserve_every_matching_binding_on_target() {
    let mut surface = bound_button_surface(vec![
        binding("Showcase/ButtonPrimary", UiEventKind::Click),
        binding("Showcase/ButtonAudit", UiEventKind::Click),
    ]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let up = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].binding_id, "Showcase/ButtonPrimary");
    assert_eq!(up.component_events[1].binding_id, "Showcase/ButtonAudit");
    for event in &up.component_events {
        assert_eq!(event.node_id, UiNodeId::new(2));
        assert_eq!(event.event_kind, UiEventKind::Click);
        assert_eq!(event.reason, UiPointerComponentEventReason::DefaultClick);
        assert_eq!(event.envelope.document_id, "runtime.ui.events");
        assert_eq!(event.envelope.control_id, "MaterialButton");
        assert_eq!(event.envelope.event_kind, UiComponentEventKind::Commit);
        assert_eq!(
            event.envelope.event,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            }
        );
    }
}

#[test]
fn compiled_event_index_preserves_sparse_binding_order() {
    let mut bindings = (0..64)
        .map(|index| binding(&format!("Showcase/Hover{index:02}"), UiEventKind::Hover))
        .collect::<Vec<_>>();
    bindings[7] = binding("Showcase/ClickEarly", UiEventKind::Click);
    bindings[55] = binding("Showcase/ClickLate", UiEventKind::Click);
    let mut surface = bound_button_surface(bindings);

    assert_eq!(
        surface.compiled_binding_event_source_count_for_test(UiNodeId::new(2), UiEventKind::Click),
        Some(2)
    );
    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), 2);
    assert_eq!(result.component_events[0].binding_id, "Showcase/ClickEarly");
    assert_eq!(result.component_events[1].binding_id, "Showcase/ClickLate");
}

#[test]
fn compiled_event_index_retains_typed_component_event() {
    let surface = bound_button_surface(vec![binding("Showcase/OpenPopup", UiEventKind::Click)]);
    let sources = surface
        .compiled_binding_event_sources_for_benchmark(UiNodeId::new(2), UiEventKind::Click)
        .collect::<Vec<_>>();

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].2, Some(UiComponentEventKind::OpenPopup));
}

#[test]
fn compiled_event_dispatch_ignores_authored_metadata_drift() {
    let mut surface = bound_button_surface(vec![binding(
        "Showcase/CompiledIdentity",
        UiEventKind::Click,
    )]);
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.bindings[0].id = "Showcase/AuthoredDrift".to_string();
    metadata.bindings[0].event = UiEventKind::Hover;

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].binding_id,
        "Showcase/CompiledIdentity"
    );
    assert!(result.component_events[0].compiled_binding.is_some());
}

#[test]
fn single_binding_event_preserves_owned_payload_allocation() {
    let surface =
        bound_button_surface(vec![binding("Showcase/SinglePayload", UiEventKind::Change)]);
    let payload = "single".repeat(1_024);
    let payload_pointer = payload.as_ptr();
    let mut events = Vec::new();
    surface
        .push_pointer_component_events_for_test(
            &mut events,
            UiNodeId::new(2),
            UiEventKind::Change,
            UiComponentEvent::KeyboardText { text: payload },
            UiPointerComponentEventReason::DirectBinding,
        )
        .unwrap();

    let UiComponentEvent::KeyboardText { text } = &events[0].envelope.event else {
        panic!("single binding should retain the keyboard text payload");
    };
    assert_eq!(text.as_ptr(), payload_pointer);

    let surface = bound_button_surface(vec![
        binding("Showcase/FirstPayload", UiEventKind::Change),
        binding("Showcase/LastPayload", UiEventKind::Change),
    ]);
    let payload = "multiple".repeat(1_024);
    let payload_pointer = payload.as_ptr();
    let mut events = Vec::new();
    surface
        .push_pointer_component_events_for_test(
            &mut events,
            UiNodeId::new(2),
            UiEventKind::Change,
            UiComponentEvent::KeyboardText { text: payload },
            UiPointerComponentEventReason::DirectBinding,
        )
        .unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].binding_id, "Showcase/FirstPayload");
    assert_eq!(events[1].binding_id, "Showcase/LastPayload");
    let UiComponentEvent::KeyboardText { text: first } = &events[0].envelope.event else {
        panic!("first binding should retain the keyboard text payload");
    };
    let UiComponentEvent::KeyboardText { text: last } = &events[1].envelope.event else {
        panic!("last binding should retain the keyboard text payload");
    };
    assert_eq!(first, last);
    assert_ne!(first.as_ptr(), payload_pointer);
    assert_eq!(last.as_ptr(), payload_pointer);
}

#[test]
fn pointer_binding_targets_commit_atomically_and_override_action_payload() {
    let mut click = binding("Showcase/AtomicTargets", UiEventKind::Click);
    click.action = Some(UiActionRef {
        route: Some("showcase.atomic_targets".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "status".to_string(),
            toml::Value::String("original".to_string()),
        )]),
        payload_missing_policy: Default::default(),
    });
    click.targets = vec![
        target(UiBindingTarget::prop("text"), r#""Bound""#),
        target(UiBindingTarget::class("highlighted"), "true"),
        target(UiBindingTarget::visibility(), "false"),
        target(UiBindingTarget::enabled(), "false"),
        target(UiBindingTarget::action_payload("status"), r#""assigned""#),
    ];
    let mut surface = bound_button_surface(vec![click]);
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("text".to_string(), toml::Value::String("Ready".to_string()));

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), 1);
    assert!(result.component_events[0].compiled_binding.is_some());
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].updates.len(), 8);
    assert_eq!(result.binding_reports[0].applied_count, 8);
    assert_eq!(result.binding_reports[0].rejected_count, 0);
    let execution = result.binding_reports[0]
        .execution_receipt
        .as_ref()
        .expect("target execution should publish a bounded receipt");
    assert_eq!(execution.asset_id, "runtime.ui.events");
    assert_eq!(execution.binding_id, "Showcase/AtomicTargets");
    assert_ne!(execution.generation, 0);
    assert_eq!(execution.execution_count, 1);
    assert_eq!(execution.miss_count, 0);
    assert_eq!(execution.error_count, 0);
    let transaction = result.binding_reports[0]
        .transaction
        .as_ref()
        .expect("target-bearing binding should publish a transaction receipt");
    assert_eq!(transaction.target_count, 5);
    assert_eq!(transaction.applied_target_count, 5);
    assert_eq!(transaction.unchanged_target_count, 0);
    assert_eq!(transaction.revision, transaction.base_generation + 1);
    assert_eq!(transaction.outcome, UiBindingMutationOutcome::Committed);
    assert!(transaction.impact.contains(&UiBindingDirtyDomain::Layout));
    assert!(transaction.impact.contains(&UiBindingDirtyDomain::Style));
    assert!(transaction.impact.contains(&UiBindingDirtyDomain::Input));
    assert!(transaction
        .impact
        .contains(&UiBindingDirtyDomain::Interaction));
    assert!(result.binding_reports[0].updates.iter().any(|update| {
        update.previous == Some(UiValue::String("Ready".to_string()))
            && update.value == UiValue::String("Bound".to_string())
    }));
    assert_eq!(
        result.component_events[0]
            .template_action
            .as_ref()
            .and_then(|action| action.payload.get("status")),
        Some(&UiValue::String("assigned".to_string()))
    );
    let node = surface.tree.node(UiNodeId::new(2)).unwrap();
    let metadata = node.template_metadata.as_ref().unwrap();
    assert_eq!(
        metadata.attributes.get("text"),
        Some(&toml::Value::String("Bound".to_string()))
    );
    assert!(metadata.classes.iter().any(|class| class == "highlighted"));
    assert!(!node.state_flags.visible);
    assert!(!node.state_flags.enabled);
}

#[test]
fn pointer_binding_target_commit_rejection_rolls_back_prior_target() {
    let mut click = binding("Showcase/RolledBackTargets", UiEventKind::Click);
    click.targets = vec![
        target(UiBindingTarget::class("highlighted"), "true"),
        target(UiBindingTarget::prop("visible"), r#""not-a-boolean""#),
    ];
    let mut surface = bound_button_surface(vec![click]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(result.component_events.is_empty());
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].rejected_count, 1);
    let execution = result.binding_reports[0]
        .execution_receipt
        .as_ref()
        .expect("failed target execution should publish a bounded receipt");
    assert_eq!(execution.asset_id, "runtime.ui.events");
    assert_eq!(execution.binding_id, "Showcase/RejectedTargets");
    assert_ne!(execution.generation, 0);
    assert_eq!(execution.execution_count, 1);
    assert_eq!(execution.miss_count, 0);
    assert_eq!(execution.error_count, 1);
    let transaction = result.binding_reports[0]
        .transaction
        .as_ref()
        .expect("commit rejection should publish a rollback receipt");
    assert_eq!(transaction.target_count, 2);
    assert_eq!(transaction.applied_target_count, 0);
    assert_eq!(transaction.unchanged_target_count, 0);
    assert_eq!(transaction.revision, transaction.base_generation);
    assert!(transaction.impact.is_empty());
    assert_eq!(transaction.outcome, UiBindingMutationOutcome::RolledBack);
    let node = surface.tree.node(UiNodeId::new(2)).unwrap();
    let metadata = node.template_metadata.as_ref().unwrap();
    assert!(!metadata.classes.iter().any(|class| class == "highlighted"));
    assert!(node.state_flags.visible);
}

#[test]
fn pointer_binding_target_filter_reuses_event_buffer_and_preserves_order() {
    let mut targeted = binding("Showcase/BufferedTarget", UiEventKind::Click);
    targeted.targets = vec![target(UiBindingTarget::prop("text"), r#""Buffered""#)];
    let mut surface = bound_button_surface(vec![
        targeted,
        binding("Showcase/BufferedPassthrough", UiEventKind::Click),
    ]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let initial = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let mut events = initial.component_events;
    events.reserve(64);
    let buffer = events.as_ptr();
    let capacity = events.capacity();

    let reports = surface.apply_pointer_binding_targets(&mut events).unwrap();

    assert_eq!(reports.len(), 1);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].binding_id, "Showcase/BufferedTarget");
    assert_eq!(events[1].binding_id, "Showcase/BufferedPassthrough");
    assert_eq!(events.capacity(), capacity);
    assert_eq!(events.as_ptr(), buffer);
}

#[test]
fn compiled_action_payload_does_not_reparse_mutated_authoring_toml() {
    let mut click = binding("Showcase/CompiledPayload", UiEventKind::Click);
    click.action = Some(UiActionRef {
        route: Some("showcase.compiled_payload".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "status".to_string(),
            toml::Value::String("compiled".to_string()),
        )]),
        payload_missing_policy: Default::default(),
    });
    let mut surface = bound_button_surface(vec![click]);
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .bindings[0]
        .action
        .as_mut()
        .unwrap()
        .payload
        .insert(
            "status".to_string(),
            toml::Value::String("tampered-after-compile".to_string()),
        );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let event = result
        .component_events
        .iter()
        .find(|event| event.binding_id == "Showcase/CompiledPayload")
        .expect("compiled binding should emit a component event");
    assert!(event.compiled_binding.is_some());
    assert_eq!(
        event
            .template_action
            .as_ref()
            .and_then(|action| action.payload.get("status")),
        Some(&UiValue::String("compiled".to_string()))
    );
}

#[test]
fn compiled_action_payload_dense_overrides_preserve_every_field() {
    let mut click = binding("Showcase/DensePayloadOverrides", UiEventKind::Click);
    click.action = Some(UiActionRef {
        route: Some("showcase.dense_payload_overrides".to_string()),
        action: None,
        payload: BTreeMap::from([
            (
                "primary".to_string(),
                toml::Value::String("original-primary".to_string()),
            ),
            (
                "secondary".to_string(),
                toml::Value::String("original-secondary".to_string()),
            ),
        ]),
        payload_missing_policy: Default::default(),
    });
    click.targets = vec![
        target(
            UiBindingTarget::action_payload("secondary"),
            r#""assigned-secondary""#,
        ),
        target(
            UiBindingTarget::action_payload("primary"),
            r#""assigned-primary""#,
        ),
    ];
    let mut surface = bound_button_surface(vec![click]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let payload = &result.component_events[0]
        .template_action
        .as_ref()
        .expect("target-bearing binding should publish its compiled action")
        .payload;
    assert_eq!(
        payload.get("primary"),
        Some(&UiValue::String("assigned-primary".to_string()))
    );
    assert_eq!(
        payload.get("secondary"),
        Some(&UiValue::String("assigned-secondary".to_string()))
    );
    let transaction = result.binding_reports[0]
        .transaction
        .as_ref()
        .expect("payload-only execution should publish a transaction receipt");
    assert_eq!(transaction.target_count, 2);
    assert_eq!(transaction.applied_target_count, 2);
    assert_eq!(transaction.unchanged_target_count, 0);
    assert_eq!(transaction.revision, transaction.base_generation);
    assert_eq!(transaction.impact, vec![UiBindingDirtyDomain::Interaction]);
}

#[test]
fn pointer_binding_target_prepare_failure_rolls_back_and_suppresses_event() {
    let mut click = binding("Showcase/RejectedTargets", UiEventKind::Click);
    click.targets = vec![
        target(UiBindingTarget::prop("text"), r#""changed""#),
        target(UiBindingTarget::class("highlighted"), "prop.missing"),
    ];
    let mut surface = bound_button_surface(vec![click]);
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert(
            "text".to_string(),
            toml::Value::String("stable".to_string()),
        );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(result.component_events.is_empty());
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].rejected_count, 1);
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(
        metadata.attributes.get("text"),
        Some(&toml::Value::String("stable".to_string()))
    );
    assert!(!metadata.classes.iter().any(|class| class == "highlighted"));
}

#[test]
fn pointer_binding_target_fast_path_skips_staging_for_one_thousand_unassigned_bindings() {
    const BINDING_COUNT: usize = 1_000;

    let bindings = (0..BINDING_COUNT)
        .map(|index| binding(&format!("Scale/Click{index}"), UiEventKind::Click))
        .collect();
    let mut surface = bound_button_surface(bindings);
    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), BINDING_COUNT);
    assert!(result.binding_reports.is_empty());
    println!(
        "PERF-RUNTIME74-BINDING-TARGET candidate_bindings={BINDING_COUNT} binding_index_scans={BINDING_COUNT} target_bindings=0 target_binding_clones=0 staged_surface_clones=0 binding_reports=0"
    );
}

fn target(target: UiBindingTarget, expression: &str) -> UiBindingTargetAssignment {
    UiBindingTargetAssignment {
        target,
        expression: expression.to_string(),
    }
}

#[test]
fn focus_component_events_emit_focus_and_blur_for_matching_bindings() {
    let mut surface = two_button_surface(
        Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some("FirstButton".to_string()),
            bindings: vec![
                binding("Showcase/FirstFocus", UiEventKind::Focus),
                binding("Showcase/FirstBlur", UiEventKind::Blur),
            ],
            ..Default::default()
        }),
        Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some("SecondButton".to_string()),
            bindings: vec![binding("Showcase/SecondFocus", UiEventKind::Focus)],
            ..Default::default()
        }),
    );

    let first_down = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(first_down.component_events.len(), 1);
    assert_eq!(
        first_down.component_events[0].binding_id,
        "Showcase/FirstFocus"
    );
    assert_eq!(
        first_down.component_events[0].event_kind,
        UiEventKind::Focus
    );
    assert_eq!(
        first_down.component_events[0].reason,
        UiPointerComponentEventReason::FocusGained
    );

    let second_down = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 60.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(second_down.component_events.len(), 2);
    assert_eq!(
        second_down.component_events[0].binding_id,
        "Showcase/FirstBlur"
    );
    assert_eq!(
        second_down.component_events[0].event_kind,
        UiEventKind::Blur
    );
    assert_eq!(
        second_down.component_events[0].reason,
        UiPointerComponentEventReason::FocusLost
    );
    assert_eq!(
        second_down.component_events[1].binding_id,
        "Showcase/SecondFocus"
    );
    assert_eq!(
        second_down.component_events[1].event_kind,
        UiEventKind::Focus
    );
    assert_eq!(
        second_down.component_events[1].reason,
        UiPointerComponentEventReason::FocusGained
    );
}

#[test]
fn release_capture_effect_clears_only_current_captor() {
    let mut surface = two_button_surface(None, None);
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.focus.pressed = Some(UiNodeId::new(2));
    let mut dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::release_capture()
    });

    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, None);
    assert_eq!(result.released_capture, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.capture_released);

    surface.focus.captured = Some(UiNodeId::new(3));
    let ignored = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(ignored.released_capture, None);
    assert!(ignored.invocations.is_empty());
    assert!(!ignored.diagnostics.capture_released);
}

#[test]
fn release_outside_pressed_target_reports_default_click_rejected() {
    let mut surface =
        bound_button_surface(vec![binding("Showcase/ButtonClick", UiEventKind::Click)]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(140.0, 80.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.route.pressed, Some(UiNodeId::new(2)));
    assert_eq!(result.route.click_target, None);
    assert!(result.diagnostics.default_click_rejected);
    assert!(result.component_events.is_empty());
}

#[test]
fn scroll_fallback_reports_scroll_defaulted_when_unhandled() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
}

#[test]
fn routed_pointer_input_preserves_scroll_delta_and_reuses_the_default_scroll_authority() {
    let mut surface = scrollable_surface();
    let route = surface
        .route_pointer_input_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(route.scroll_delta, 50.0);
    assert_eq!(
        surface.apply_default_pointer_scroll(&route).unwrap(),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
}

#[test]
fn scroll_fallback_does_not_handle_when_scroll_offset_is_unchanged() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(0.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, None);
    assert!(!result.diagnostics.scroll_defaulted);
}

#[test]
fn scroll_fallback_continues_to_ancestor_when_nearest_scrollable_is_clamped() {
    let mut surface = nested_scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(20.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        20.0
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}

#[test]
fn pointer_dispatch_result_counts_component_events() {
    let mut surface = bound_button_surface(vec![
        binding("Showcase/ButtonPress", UiEventKind::Press),
        binding("Showcase/ButtonFocus", UiEventKind::Focus),
    ]);

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), 2);
    assert_eq!(result.diagnostics.component_event_count, 2);
    assert_eq!(result.component_events[0].event_kind, UiEventKind::Press);
    assert_eq!(result.component_events[1].event_kind, UiEventKind::Focus);
}
