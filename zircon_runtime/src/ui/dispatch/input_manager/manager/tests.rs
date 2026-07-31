use std::{collections::BTreeMap, time::Duration};

use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiComponentEventReport, UiDispatchDisposition, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence, UiInputTimestamp,
        UiPointerSource, UiTextInputEvent,
    },
    dispatch::{UiDispatchHostRequestKind, UiTooltipTimerInputEventKind},
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    surface::UiPointerButton,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::UiWidgetContract,
};

use crate::ui::surface::UiSurface;

use super::UiInputManager;

#[test]
fn frame_visible_timer_deadline_returns_earliest_non_negative_delay() {
    let now = UiInputTimestamp::from_micros(1_000);
    let mut manager = UiInputManager::default();

    manager
        .timers
        .arm_typeahead_expiration(UiNodeId::new(1), now, 90);
    manager
        .timers
        .arm_submenu_hover_expiration(UiNodeId::new(2), "file", now, 40);
    manager
        .timers
        .arm_tooltip_expiration(UiNodeId::new(3), "status.hint", now, 60);
    manager
        .timers
        .arm_toast_expiration(UiNodeId::new(4), "saved", now, 80);

    assert_eq!(
        manager.next_frame_visible_delay(now),
        Some(Duration::from_millis(40))
    );
    assert_eq!(
        manager.next_frame_visible_delay(UiInputTimestamp::from_micros(41_001)),
        Some(Duration::ZERO),
        "an overdue frame-visible timer must request an immediate frame"
    );
}

#[test]
fn double_click_candidate_is_not_a_frame_visible_deadline() {
    let now = UiInputTimestamp::from_micros(1_000);
    let mut manager = UiInputManager::default();

    assert_eq!(manager.next_frame_visible_delay(now), None);
    manager.timers.arm_double_click_candidate(
        UiNodeId::new(1),
        None,
        UiPointerSource::Mouse,
        Some(UiPointerButton::Primary),
        1,
        now,
    );

    assert_eq!(
        manager.next_frame_visible_delay(now),
        None,
        "double-click state is input classification, not frame-visible work"
    );
}

#[test]
fn hovered_menu_option_arms_replaces_and_clears_submenu_hover_timer() {
    let target = UiNodeId::new(2);
    for component in ["MenuList", "ContextMenu", "DropdownPopup"] {
        let mut surface = submenu_hover_surface(component);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(50),
            &hover_changed_result(target, "file"),
        );

        assert_eq!(
            manager.timers().submenu_hover_expiration(target),
            Some(UiInputTimestamp::from_micros(80_050)),
            "{component} should arm submenu hover from hovered_option_id"
        );
        assert_eq!(
            manager.timers().submenu_hover_option_id(target),
            Some("file"),
            "{component} should retain the hovered submenu option id"
        );

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(70),
            &hover_changed_result(target, "edit"),
        );

        assert_eq!(
            manager.timers().submenu_hover_expiration(target),
            Some(UiInputTimestamp::from_micros(80_070)),
            "{component} should replace an existing submenu hover timer"
        );
        assert_eq!(
            manager.timers().submenu_hover_option_id(target),
            Some("edit"),
            "{component} should replace the pending submenu option id"
        );

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(90),
            &hover_changed_result(target, ""),
        );

        assert_eq!(
            manager.timers().submenu_hover_expiration(target),
            None,
            "{component} should clear submenu hover when hover leaves an option"
        );
        assert_eq!(manager.timers().submenu_hover_option_id(target), None);
    }
}

#[test]
fn popup_menu_shells_expose_typeahead_and_submenu_timer_contracts() {
    let target = UiNodeId::new(2);
    for component in ["MenuList", "ContextMenu", "DropdownPopup"] {
        let surface = submenu_hover_surface(component);
        assert_eq!(
            surface.typeahead_timeout_ms_for_component_node(target),
            Some(120),
            "{component} should use authored typeahead timing"
        );
        assert_eq!(
            surface.submenu_hover_delay_ms_for_component_node(target),
            Some(80),
            "{component} should use authored submenu hover timing"
        );
    }
}

#[test]
fn toast_queue_value_arms_replaces_and_clears_auto_hide_timer() {
    let target = UiNodeId::new(2);
    let mut surface = toast_surface("surface-save", 4000);
    let mut manager = UiInputManager::default();

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(50),
        &component_event_result(
            target,
            UiComponentEvent::ValueChanged {
                property: "toast_queue".to_string(),
                value: UiValue::String("save|message=Saved|autoHideDuration=40".to_string()),
            },
        ),
    );

    assert_eq!(
        manager.timers().toast_expiration(target),
        Some(UiInputTimestamp::from_micros(40_050))
    );
    assert_eq!(manager.timers().toast_id(target), Some("save"));

    let mut next_toast = BTreeMap::new();
    next_toast.insert("id".to_string(), UiValue::String("export".to_string()));
    next_toast.insert("auto_hide_duration_ms".to_string(), UiValue::Int(80));
    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(70),
        &component_event_result(
            target,
            UiComponentEvent::ValueChanged {
                property: "toast_queue".to_string(),
                value: UiValue::Array(vec![UiValue::Map(next_toast)]),
            },
        ),
    );

    assert_eq!(
        manager.timers().toast_expiration(target),
        Some(UiInputTimestamp::from_micros(80_070))
    );
    assert_eq!(manager.timers().toast_id(target), Some("export"));

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(90),
        &component_event_result(target, UiComponentEvent::ClosePopup),
    );

    assert_eq!(manager.timers().toast_expiration(target), None);
    assert_eq!(manager.timers().toast_id(target), None);
}

#[test]
fn toast_auto_hide_tick_dispatches_expired_commit_event() {
    let target = UiNodeId::new(2);
    let mut surface = toast_surface("save", 40);
    let mut manager = UiInputManager::default();

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(10),
        &component_event_result(
            target,
            UiComponentEvent::ValueChanged {
                property: "current_toast_id".to_string(),
                value: UiValue::String("save".to_string()),
            },
        ),
    );

    assert_eq!(
        manager.timers().toast_expiration(target),
        Some(UiInputTimestamp::from_micros(40_010))
    );

    let early = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(40_009))
        .unwrap();
    assert!(early.is_empty());

    let expired = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(40_010))
        .unwrap();

    assert_eq!(expired.len(), 1);
    let expired = &expired[0];
    assert_eq!(expired.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(expired.reply.handler, Some(target));
    assert_eq!(
        expired.diagnostics.handled_phase.as_deref(),
        Some("toast_timer.component_event")
    );
    assert_eq!(
        expired.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(expired.diagnostics.route_target, Some(target));
    assert_eq!(expired.component_events.len(), 1);
    assert_eq!(expired.component_events[0].target, target);
    assert_eq!(
        expired.component_events[0].event,
        UiComponentEvent::Commit {
            property: "expired_toast_id".to_string(),
            value: UiValue::String("save".to_string()),
        }
    );
    match &expired.event {
        UiInputEvent::ToastTimer(timer) => {
            assert_eq!(
                timer.metadata.timestamp,
                UiInputTimestamp::from_micros(40_010)
            );
            assert_eq!(timer.target, target);
            assert_eq!(timer.toast_id, "save");
        }
        other => panic!("expected toast timer input event, got {other:?}"),
    }
    assert_eq!(manager.timers().toast_expiration(target), None);
}

#[test]
fn tooltip_hover_arms_and_clears_manager_timer_candidate() {
    let target = UiNodeId::new(2);
    let mut surface = tooltip_surface("status.hint", 40);
    let mut manager = UiInputManager::default();

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(25),
        &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
    );

    assert_eq!(
        manager.timers().tooltip_expiration(target),
        Some(UiInputTimestamp::from_micros(40_025))
    );
    assert_eq!(manager.timers().tooltip_id(target), Some("status.hint"));
    assert_eq!(
        surface.input.tooltip.as_ref().map(|tooltip| (
            tooltip.tooltip_id.as_str(),
            tooltip.owner,
            tooltip.visible
        )),
        Some(("status.hint", Some(target), false))
    );

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(30),
        &component_event_result(target, UiComponentEvent::Hover { hovered: false }),
    );

    assert_eq!(manager.timers().tooltip_expiration(target), None);
    assert_eq!(manager.timers().tooltip_id(target), None);
    assert_eq!(surface.input.tooltip, None);
}

#[test]
fn tooltip_hover_timer_tick_dispatches_elapsed_default_action() {
    let target = UiNodeId::new(2);
    let mut surface = tooltip_surface("status.hint", 40);
    let mut manager = UiInputManager::default();

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(10),
        &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
    );

    let early = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(40_009))
        .unwrap();
    assert!(early.is_empty());

    let expired = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(40_010))
        .unwrap();

    assert_eq!(expired.len(), 1);
    let expired = &expired[0];
    assert_eq!(expired.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(expired.diagnostics.route_target, Some(target));
    assert_eq!(
        expired.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        expired.diagnostics.handled_phase.as_deref(),
        Some("tooltip.effect")
    );
    assert!(matches!(
        expired.host_requests[0].request,
        UiDispatchHostRequestKind::Tooltip {
            kind: zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Show,
            ref tooltip_id,
        } if tooltip_id == "status.hint"
    ));
    assert_eq!(
        surface.input.tooltip.as_ref().map(|tooltip| (
            tooltip.tooltip_id.as_str(),
            tooltip.owner,
            tooltip.visible
        )),
        Some(("status.hint", Some(target), true))
    );
    match &expired.event {
        UiInputEvent::TooltipTimer(tooltip) => {
            assert_eq!(tooltip.kind, UiTooltipTimerInputEventKind::Elapsed);
            assert_eq!(tooltip.tooltip_id, "status.hint");
            assert_eq!(tooltip.owner, Some(target));
            assert!(tooltip.metadata.synthetic);
        }
        other => panic!("expected tooltip timer input event, got {other:?}"),
    }
    assert_eq!(manager.timers().tooltip_expiration(target), None);
}

#[test]
fn tooltip_candidate_clears_on_following_input_activity() {
    let target = UiNodeId::new(2);
    let mut surface = tooltip_surface("status.hint", 40);
    let mut manager = UiInputManager::default();

    manager.arm_timers_from_component_events(
        &mut surface,
        UiInputTimestamp::from_micros(10),
        &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
    );
    assert_eq!(manager.timers().tooltip_id(target), Some("status.hint"));
    assert!(surface.input.tooltip.is_some());

    manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(20),
                    UiInputSequence::new(20),
                ),
                text: "x".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(manager.timers().tooltip_expiration(target), None);
    assert_eq!(manager.timers().tooltip_id(target), None);
    assert_eq!(surface.input.tooltip, None);
}

fn submenu_hover_surface(component: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.submenu_hover"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("menu")));
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some("SceneMenu".to_string()),
        attributes: toml::from_str(
            r#"
typeahead_timeout_ms = 120
submenu_hover_delay_ms = 80
"#,
        )
        .unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn toast_surface(toast_id: &str, duration_ms: i64) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.toast"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("toast")));
    let mut attributes = BTreeMap::new();
    attributes.insert(
        "current_toast_id".to_string(),
        toml::Value::String(toast_id.to_string()),
    );
    attributes.insert(
        "auto_hide_duration_ms".to_string(),
        toml::Value::Integer(duration_ms),
    );
    attributes.insert("open".to_string(), toml::Value::Boolean(true));
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: "Snackbar".to_string(),
        control_id: Some("StatusToast".to_string()),
        bindings: vec![binding("Snackbar/Commit", "Change")],
        attributes,
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn tooltip_surface(tooltip_id: &str, delay_ms: i64) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.tooltip"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("button")));
    let mut attributes = BTreeMap::new();
    attributes.insert(
        "tooltip_delay_ms".to_string(),
        toml::Value::Integer(delay_ms),
    );
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("StatusButton".to_string()),
        widget: UiWidgetContract {
            tooltip: Some(tooltip_id.to_string()),
            ..UiWidgetContract::default()
        },
        attributes,
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn hover_changed_result(target: UiNodeId, option_id: &str) -> UiInputDispatchResult {
    component_event_result(
        target,
        UiComponentEvent::ValueChanged {
            property: "hovered_option_id".to_string(),
            value: UiValue::String(option_id.to_string()),
        },
    )
}

fn component_event_result(target: UiNodeId, event: UiComponentEvent) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Text(UiTextInputEvent {
            metadata: UiInputEventMetadata::new(
                UiInputTimestamp::from_micros(0),
                UiInputSequence::new(0),
            ),
            text: String::new(),
        }),
        UiDispatchReply::handled(),
    );
    result.component_events.push(UiComponentEventReport {
        target,
        event,
        delivered: true,
        drag: None,
        template_action: None,
    });
    result
}

fn binding(id: &str, event: &str) -> zircon_runtime_interface::ui::template::UiBindingRef {
    zircon_runtime_interface::ui::template::UiBindingRef {
        id: id.to_string(),
        event: match event {
            "Change" => zircon_runtime_interface::ui::binding::UiEventKind::Change,
            other => panic!("unsupported binding event {other}"),
        },
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}
