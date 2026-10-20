//! 悬停进入和离开使用直接命中路由；组件事件与逐步诊断应反映同一目标切换。

use super::*;

#[test]
fn unified_pointer_hover_enter_leave_report_direct_route_steps_and_component_events() {
    let mut surface = hover_route_surface();

    let enter = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_direct_hover_step(
        &enter,
        Some(UiNodeId::new(2)),
        UiNodeId::new(2),
        UiNodeId::new(2),
    );
    assert_eq!(
        enter.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        enter.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        enter.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(enter.component_events.len(), 1);
    assert_eq!(enter.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        enter.component_events[0].event,
        UiComponentEvent::Hover { hovered: true }
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.pressed, None);
    assert_eq!(surface.focus.focused, None);

    let switch = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 60.0)),
        )
        .unwrap();

    assert_direct_hover_step(
        &switch,
        Some(UiNodeId::new(3)),
        UiNodeId::new(3),
        UiNodeId::new(2),
    );
    assert_eq!(
        switch.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(switch.component_events.len(), 2);
    assert_eq!(switch.component_events[0].target, UiNodeId::new(3));
    assert_eq!(
        switch.component_events[0].event,
        UiComponentEvent::Hover { hovered: true }
    );
    assert_eq!(switch.component_events[1].target, UiNodeId::new(2));
    assert_eq!(
        switch.component_events[1].event,
        UiComponentEvent::Hover { hovered: false }
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );

    let leave = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(200.0, 120.0)),
        )
        .unwrap();

    assert_direct_hover_step(&leave, None, UiNodeId::new(3), UiNodeId::new(3));
    assert_eq!(leave.diagnostics.route_trace.direct_target, None);
    assert_eq!(leave.diagnostics.route_trace.target, None);
    assert_eq!(
        leave.diagnostics.route_trace.root_targets,
        vec![UiNodeId::new(1)]
    );
    assert_eq!(leave.component_events.len(), 1);
    assert_eq!(leave.component_events[0].target, UiNodeId::new(3));
    assert_eq!(
        leave.component_events[0].event,
        UiComponentEvent::Hover { hovered: false }
    );
    assert!(surface.focus.hovered.is_empty());
}

// route_target 记录当前命中；切换或离开时 handler 可来自最后一个离开事件，因此允许与命中目标不同。
fn assert_direct_hover_step(
    result: &UiInputDispatchResult,
    route_target: Option<UiNodeId>,
    step_target: UiNodeId,
    handler: UiNodeId,
) {
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(result.diagnostics.route_target, route_target);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(handler));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(result.diagnostics.route_steps[0].target, Some(step_target));
    assert_eq!(result.diagnostics.route_steps[0].handler, Some(handler));
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 1);
    assert_cursor_result(
        result,
        zircon_runtime_interface::ui::tree::UiCursor::Default,
    );
    assert!(result.diagnostics.route_steps[0].stopped);
}

fn hover_route_surface() -> UiSurface {
    let mut surface = route_surface();
    for (node_id, control_id) in [
        (UiNodeId::new(2), "FirstHover"),
        (UiNodeId::new(3), "SecondHover"),
    ] {
        let target = surface.tree.nodes.get_mut(&node_id).unwrap();
        target.template_metadata = Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            bindings: vec![binding(&format!("{control_id}/Hover"), UiEventKind::Hover)],
            ..Default::default()
        });
    }
    surface.rebuild();
    surface
}

#[test]
fn mouse_cursor_uses_physical_path_inheritance_and_explicit_default_without_touch_effects() {
    use zircon_runtime_interface::ui::tree::{UiCursor, UiPointerEvents};
    let mut surface = route_surface();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .cursor = Some(UiCursor::Text);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .pointer_events = UiPointerEvents::Pass;
    surface.rebuild();
    for (child_cursor, expected) in [
        (None, UiCursor::Text),
        (Some(UiCursor::Default), UiCursor::Default),
        (Some(UiCursor::Pointer), UiCursor::Pointer),
    ] {
        surface
            .tree
            .nodes
            .get_mut(&UiNodeId::new(2))
            .unwrap()
            .cursor = child_cursor;
        let result = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
            )
            .unwrap();
        assert_cursor_result(&result, expected);
        assert_eq!(
            result
                .pointer_routing
                .as_ref()
                .unwrap()
                .physical_hit_path
                .target,
            Some(UiNodeId::new(2))
        );
    }
    surface.capture_pointer(UiNodeId::new(3)).unwrap();
    let captured = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert_cursor_result(&captured, UiCursor::Pointer);
    assert_eq!(
        captured.pointer_routing.as_ref().unwrap().capture_target,
        Some(UiNodeId::new(3))
    );
    for (kind, point) in [
        (UiPointerEventKind::Move, UiPoint::new(500.0, 500.0)),
        (UiPointerEventKind::Cancel, UiPoint::new(20.0, 20.0)),
    ] {
        let result = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(kind, point),
            )
            .unwrap();
        // Cancel may additionally publish the original capture-release effects.
        let cursors: Vec<_> = result
            .reply
            .effects
            .iter()
            .enumerate()
            .filter_map(|(index, e)| match e {
                UiDispatchEffect::RequestCursor { cursor } => Some((index, *cursor)),
                _ => None,
            })
            .collect();
        assert_eq!(cursors.len(), 1);
        assert_eq!(cursors[0].1, UiCursor::Default);
        assert!(result
            .host_requests
            .iter()
            .any(|r| r.effect_index == cursors[0].0
                && r.request == UiDispatchHostRequestKind::Cursor(UiCursor::Default)));
    }
    for source in [
        UiPointerSource::Touch,
        UiPointerSource::Pen,
        UiPointerSource::Unknown,
    ] {
        let mut event = pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0));
        if let UiInputEvent::Pointer(pointer) = &mut event {
            pointer.metadata.pointer_source = source;
        }
        let result = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                event,
            )
            .unwrap();
        assert!(!result
            .reply
            .effects
            .iter()
            .any(|e| matches!(e, UiDispatchEffect::RequestCursor { .. })));
        assert!(!result
            .host_requests
            .iter()
            .any(|r| matches!(r.request, UiDispatchHostRequestKind::Cursor(_))));
    }
}
