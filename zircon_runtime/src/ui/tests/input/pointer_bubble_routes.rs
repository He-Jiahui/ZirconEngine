//! 指针预览、目标、冒泡和默认动作按阶段传播；已处理预览截断后续节点，点击与滚动共用一次分发。

use super::*;

#[test]
fn unified_pointer_dispatch_reports_phase_route_steps() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_cursor_result(
        &result,
        zircon_runtime_interface::ui::tree::UiCursor::Default,
    );
    assert_eq!(result.diagnostics.route_steps.len(), 4);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert_eq!(result.diagnostics.route_steps[2].ignored_effect_count, 1);
    assert!(!result.diagnostics.route_steps[2].stopped);
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
}

#[test]
fn pointer_preview_tunnel_handler_stops_before_target_and_bubble_handlers() {
    let mut surface = route_surface();
    let target_calls = Arc::new(AtomicUsize::new(0));
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register_phase(
        UiNodeId::new(1),
        UiPointerEventKind::Down,
        UiDispatchPhase::PreviewTunnel,
        |context| {
            assert_eq!(context.node_id, UiNodeId::new(1));
            assert_eq!(context.phase, UiDispatchPhase::PreviewTunnel);
            assert_eq!(context.route.target, Some(UiNodeId::new(2)));
            UiPointerDispatchEffect::handled()
        },
    );
    let target_calls_for_handler = Arc::clone(&target_calls);
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, move |_| {
        target_calls_for_handler.fetch_add(1, Ordering::SeqCst);
        UiPointerDispatchEffect::handled()
    });

    let result = surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(target_calls.load(Ordering::SeqCst), 0);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(1)));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::PreviewTunnel));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert!(result.diagnostics.route_steps[0].stopped);
}

#[test]
fn unified_pointer_press_release_report_bubble_route_steps_and_component_events() {
    let mut surface = press_release_route_surface();

    let down = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&down);
    assert_eq!(down.component_events.len(), 1);
    assert_eq!(down.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        down.component_events[0].event,
        UiComponentEvent::Press { pressed: true }
    );
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
    assert!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );

    let up = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&up);
    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[0].event,
        UiComponentEvent::Press { pressed: false }
    );
    assert_eq!(up.component_events[1].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[1].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(surface.focus.pressed, None);
    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );
}

#[test]
fn unified_pointer_double_click_reports_bubble_route_steps_and_default_binding() {
    let mut surface = double_click_route_surface();

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event_with_click_count(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0), 2),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_cursor_result(
        &result,
        zircon_runtime_interface::ui::tree::UiCursor::Default,
    );
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "double_activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.click_count, 2),
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn unified_pointer_scroll_reports_bubble_route_steps_and_precise_delta() {
    let mut surface = scroll_route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            scroll_event(UiPoint::new(20.0, 20.0), 50.0),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(20)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=50"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(20)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(20), UiNodeId::new(2), UiNodeId::new(1)]
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.event.scroll_delta, 50.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(0.0, 50.0))
            );
        }
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 5);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert!(!result.diagnostics.route_steps[3].stopped);
    assert_eq!(
        result.diagnostics.route_steps[4].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[4].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[4].effect_count, 1);
    assert_cursor_result(
        &result,
        zircon_runtime_interface::ui::tree::UiCursor::Default,
    );
    assert!(result.diagnostics.route_steps[4].stopped);
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

fn pass_overlay_surface() -> UiSurface {
    use zircon_runtime_interface::ui::tree::UiPointerEvents;
    let mut surface = press_release_route_surface();
    let metadata = surface.tree.nodes[&UiNodeId::new(2)]
        .template_metadata
        .clone();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .template_metadata = metadata;
    let front = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    front.pointer_events = UiPointerEvents::Pass;
    front.z_index = 10;
    let behind = surface.tree.nodes.get_mut(&UiNodeId::new(3)).unwrap();
    behind.layout_cache.frame = UiFrame::new(10.0, 10.0, 80.0, 30.0);
    surface.rebuild();
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(2))
    );
    surface
}

#[test]
fn pass_overlay_ordinary_replies_bubble_to_real_parent_without_back_stack() {
    use zircon_runtime_interface::ui::tree::UiPointerEvents;
    for effect in [
        UiPointerDispatchEffect::Handled,
        UiPointerDispatchEffect::Blocked,
    ] {
        for pass in [true, false] {
            let mut surface = pass_overlay_surface();
            if !pass {
                surface
                    .tree
                    .nodes
                    .get_mut(&UiNodeId::new(2))
                    .unwrap()
                    .pointer_events = UiPointerEvents::Auto;
                surface.rebuild();
            }
            let behind_calls = Arc::new(AtomicUsize::new(0));
            let mut dispatcher = UiPointerDispatcher::default();
            dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, move |_| effect);
            dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, |_| {
                UiPointerDispatchEffect::Handled
            });
            let calls = behind_calls.clone();
            dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                UiPointerDispatchEffect::Handled
            });
            let result = surface
                .dispatch_pointer_event(
                    &dispatcher,
                    UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                        .with_button(UiPointerButton::Primary),
                )
                .unwrap();
            assert_eq!(result.route.target, Some(UiNodeId::new(2)));
            assert_eq!(result.invocations[0].effect, effect);
            if pass {
                assert_eq!(result.handled_by, Some(UiNodeId::new(1)));
                assert_eq!(result.blocked_by, None);
                assert_eq!(behind_calls.load(Ordering::SeqCst), 0);
            } else if effect == UiPointerDispatchEffect::Handled {
                assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
                assert_eq!(result.invocations.len(), 1);
            } else {
                assert_eq!(result.blocked_by, Some(UiNodeId::new(2)));
            }
        }
    }
}

#[test]
fn pass_default_actions_use_parent_but_nonpass_children_keep_their_authority() {
    use zircon_runtime_interface::ui::tree::UiPointerEvents;
    let mut surface = pass_overlay_surface();
    let down = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert_eq!(down.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(1)));
    assert!(down
        .component_events
        .iter()
        .any(|e| e.target == UiNodeId::new(1)
            && matches!(e.event, UiComponentEvent::Press { pressed: true })));
    assert!(
        !down
            .component_events
            .iter()
            .any(|e| e.target == UiNodeId::new(2)
                && matches!(e.event, UiComponentEvent::Press { .. }))
    );
    let up = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert!(up.component_events.iter().any(
        |e| e.target == UiNodeId::new(1) && matches!(e.event, UiComponentEvent::Commit { .. })
    ));
    assert!(!up.component_events.iter().any(
        |e| e.target == UiNodeId::new(3) && matches!(e.event, UiComponentEvent::Commit { .. })
    ));

    let mut surface = pass_overlay_surface();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .pointer_events = UiPointerEvents::Pass;
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .pointer_events = UiPointerEvents::Auto;
    surface.rebuild();
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::Handled
    });
    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(result.invocations.len(), 1);
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
}

#[test]
fn pass_preview_and_explicit_controls_keep_authority_and_capture_is_exclusive() {
    for effect in [
        UiPointerDispatchEffect::CapturePointer,
        UiPointerDispatchEffect::ReleasePointerCapture,
        UiPointerDispatchEffect::SetFocus,
        UiPointerDispatchEffect::ClearFocus,
    ] {
        let mut surface = pass_overlay_surface();
        let mut dispatcher = UiPointerDispatcher::default();
        dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, move |_| effect);
        dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, |_| {
            panic!("explicit control must stop")
        });
        let result = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                    .with_button(UiPointerButton::Primary),
            )
            .unwrap();
        assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
        assert_eq!(result.invocations.len(), 1);
        assert!(!result
            .component_events
            .iter()
            .any(|e| matches!(&e.envelope.event, UiComponentEvent::Press { .. })));
        match effect {
            UiPointerDispatchEffect::CapturePointer => {
                assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)))
            }
            UiPointerDispatchEffect::ReleasePointerCapture => {
                assert_eq!(result.released_capture, Some(UiNodeId::new(2)))
            }
            UiPointerDispatchEffect::SetFocus => {
                assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)))
            }
            UiPointerDispatchEffect::ClearFocus => assert_eq!(surface.focus.focused, None),
            _ => unreachable!(),
        }
    }
    for effect in [
        UiPointerDispatchEffect::Handled,
        UiPointerDispatchEffect::Blocked,
    ] {
        let mut surface = pass_overlay_surface();
        let mut dispatcher = UiPointerDispatcher::default();
        dispatcher.register_phase(
            UiNodeId::new(2),
            UiPointerEventKind::Down,
            UiDispatchPhase::PreviewTunnel,
            move |_| effect,
        );
        dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, |_| {
            panic!("preview must stop")
        });
        let result = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                    .with_button(UiPointerButton::Primary),
            )
            .unwrap();
        assert_eq!(result.invocations.len(), 1);
        assert_eq!(result.invocations[0].phase, UiDispatchPhase::PreviewTunnel);
        assert!(!result
            .component_events
            .iter()
            .any(|e| matches!(&e.envelope.event, UiComponentEvent::Press { .. })));
    }
    let mut surface = pass_overlay_surface();
    surface.capture_pointer(UiNodeId::new(2)).unwrap();
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Move, |_| {
        panic!("capture must not visit behind stack")
    });
    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert_eq!(result.route.captured, Some(UiNodeId::new(2)));
    assert!(result.invocations.is_empty());
}

fn assert_pass_cursor_index(
    result: &UiInputDispatchResult,
    expected: zircon_runtime_interface::ui::tree::UiCursor,
) {
    let cursors: Vec<_> = result
        .reply
        .effects
        .iter()
        .enumerate()
        .filter_map(|(index, effect)| {
            if let UiDispatchEffect::RequestCursor { cursor } = effect {
                Some((index, *cursor))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cursors.len(), 1);
    let (index, cursor) = cursors[0];
    assert_eq!(cursor, expected);
    let applied: Vec<_> = result
        .applied_effects
        .iter()
        .filter(|effect| effect.effect_index == index)
        .collect();
    assert_eq!(applied.len(), 1);
    assert_eq!(
        applied[0].effect,
        UiDispatchEffect::RequestCursor { cursor: expected }
    );
    let hosts: Vec<_> = result
        .host_requests
        .iter()
        .filter(|request| matches!(request.request, UiDispatchHostRequestKind::Cursor(_)))
        .collect();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].effect_index, index);
    assert_eq!(
        hosts[0].request,
        UiDispatchHostRequestKind::Cursor(expected)
    );
    assert!(!result
        .rejected_effects
        .iter()
        .any(|effect| effect.effect_index == index));
}

#[test]
fn pass_overlay_autofocus_skips_pass_node_and_preserves_auto_positive_control() {
    use zircon_runtime_interface::ui::tree::UiPointerEvents;
    for pass in [true, false] {
        let mut surface = pass_overlay_surface();
        if !pass {
            surface
                .tree
                .nodes
                .get_mut(&UiNodeId::new(2))
                .unwrap()
                .pointer_events = UiPointerEvents::Auto;
            surface.rebuild();
        }
        assert!(surface.tree.nodes[&UiNodeId::new(2)].state_flags.focusable);
        let result = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
            )
            .unwrap();
        let owner = UiNodeId::new(if pass { 1 } else { 2 });
        let routing = result.pointer_routing.as_ref().unwrap();
        assert_eq!(routing.physical_hit_path.target, Some(UiNodeId::new(2)));
        assert_eq!(routing.route_target, Some(UiNodeId::new(2)));
        assert_eq!(surface.focus.focused, Some(owner));
        assert_eq!(surface.focus.pressed, Some(owner));
    }
}

#[test]
fn pass_overlay_text_selection_uses_editable_parent_and_physical_cursor() {
    use zircon_runtime_interface::ui::tree::{UiCursor, UiPointerEvents};
    for pass in [true, false] {
        let parent = UiNodeId::new(2);
        let overlay = UiNodeId::new(4);
        let mut surface = editable_route_surface("abcd", 0);
        surface.tree.nodes.get_mut(&parent).unwrap().cursor = Some(UiCursor::Text);
        let frame = surface.tree.nodes[&parent].layout_cache.frame;
        let mut child = UiTreeNode::new(overlay, UiNodePath::new("root/editable/overlay"))
            .with_frame(frame)
            .with_z_index(10)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(input_state());
        child.pointer_events = if pass {
            UiPointerEvents::Pass
        } else {
            UiPointerEvents::Auto
        };
        child.cursor = Some(UiCursor::Pointer);
        surface.tree.insert_child(parent, child).unwrap();
        surface.rebuild();
        let command = surface
            .render_extract
            .list
            .commands
            .iter()
            .find(|command| {
                command.node_id == parent
                    && command.text.as_deref() == Some("abcd")
                    && command.text_layout.is_some()
            })
            .expect("editable parent publishes its actual text layout");
        let line = &command.text_layout.as_ref().unwrap().lines[0];
        let point = UiPoint::new(
            line.frame.x + line.glyph_advances.iter().sum::<f32>() + 1.0,
            line.frame.y + line.frame.height * 0.5,
        );
        assert!(frame.contains_point(point));
        assert_eq!(surface.hit_test(point).top_hit, Some(overlay));
        let caret = |surface: &UiSurface| {
            surface.tree.nodes[&parent]
                .template_metadata
                .as_ref()
                .unwrap()
                .attributes
                .get("caret_offset")
                .and_then(toml::Value::as_integer)
                .unwrap()
        };
        assert_eq!(caret(&surface), 0);
        let result = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(UiPointerEventKind::Down, point),
            )
            .unwrap();
        let routing = result.pointer_routing.as_ref().unwrap();
        assert_eq!(routing.physical_hit_path.target, Some(overlay));
        assert_eq!(routing.route_target, Some(overlay));
        assert_eq!(editable_attr_string(&surface, "value"), "abcd");
        assert_pass_cursor_index(&result, UiCursor::Pointer);
        if pass {
            assert_eq!(caret(&surface), 4);
            assert_eq!(surface.focus.focused, Some(parent));
            assert_eq!(surface.focus.captured, Some(parent));
            assert_eq!(
                surface.input.pointer_capture_owner(UiPointerId::new(7)),
                Some(parent)
            );
            assert_eq!(routing.capture_target, Some(parent));
            assert!(result.reply.effects.iter().any(|effect| matches!(effect,
                UiDispatchEffect::CapturePointer { target, pointer_id, .. } if *target == parent && *pointer_id == UiPointerId::new(7))));
            assert!(!result.reply.effects.iter().any(|effect| matches!(effect,
                UiDispatchEffect::CapturePointer { target, .. } if *target == overlay)));
        } else {
            assert_eq!(caret(&surface), 0);
            assert_eq!(surface.focus.focused, Some(overlay));
            assert_eq!(
                surface.input.pointer_capture_owner(UiPointerId::new(7)),
                None
            );
            assert!(!result.reply.effects.iter().any(|effect| matches!(effect,
                UiDispatchEffect::CapturePointer { target, .. } if *target == parent)));
        }
    }
}

#[test]
fn pass_overlay_rich_link_activation_uses_parent_and_keeps_original_hit_path() {
    use crate::ui::text::layout_text;
    use zircon_runtime_interface::ui::{
        surface::{
            UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat,
            UiTextOverflow, UiTextWrap,
        },
        tree::{UiCursor, UiPointerEvents},
    };
    for pass in [true, false] {
        let parent = UiNodeId::new(1);
        let overlay = UiNodeId::new(2);
        let mut surface = route_surface();
        let node = surface.tree.nodes.get_mut(&overlay).unwrap();
        node.pointer_events = if pass {
            UiPointerEvents::Pass
        } else {
            UiPointerEvents::Auto
        };
        node.cursor = Some(UiCursor::Pointer);
        node.z_index = 10;
        surface.tree.nodes.get_mut(&parent).unwrap().cursor = Some(UiCursor::Text);
        surface.rebuild();
        let frame = surface.tree.nodes[&overlay].layout_cache.frame;
        let mut style = UiResolvedStyle::default();
        style.rich_text_format = UiRichTextFormat::HtmlSubsetV1;
        style.wrap = UiTextWrap::None;
        style.text_overflow = UiTextOverflow::Clip;
        let markup = "<a href=\"res://docs/pass-parent.md\">help</a>";
        let layout = layout_text(markup, &style, frame, None);
        let line = &layout.lines[0];
        let point = UiPoint::new(line.frame.x + 1.0, line.frame.y + line.frame.height * 0.5);
        assert!(frame.contains_point(point));
        let mut extract = std::mem::take(&mut surface.render_extract);
        extract
            .list
            .commands
            .retain(|command| command.node_id != parent);
        extract.list.commands.push(UiRenderCommand {
            node_id: parent,
            kind: UiRenderCommandKind::Text,
            frame,
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: Some(layout),
            text: Some(markup.to_string()),
            image: None,
            opacity: 1.0,
        });
        surface.render_extract = surface
            .render_cache
            .update(&surface.render_extract, extract, false)
            .extract;
        assert_eq!(surface.hit_test(point).top_hit, Some(overlay));
        let down = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(UiPointerEventKind::Down, point),
            )
            .unwrap();
        assert_eq!(
            down.pointer_routing
                .as_ref()
                .unwrap()
                .physical_hit_path
                .target,
            Some(overlay)
        );
        assert_eq!(
            surface.focus.pressed,
            Some(if pass { parent } else { overlay })
        );
        let up = surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                pointer_event(UiPointerEventKind::Up, point),
            )
            .unwrap();
        assert_eq!(
            up.pointer_routing
                .as_ref()
                .unwrap()
                .physical_hit_path
                .target,
            Some(overlay)
        );
        assert_eq!(
            up.pointer_routing.as_ref().unwrap().route_target,
            Some(overlay)
        );
        assert_pass_cursor_index(&up, UiCursor::Pointer);
        let links: Vec<_> = up
            .host_requests
            .iter()
            .filter(|request| {
                matches!(
                    request.request,
                    UiDispatchHostRequestKind::ActivateLink { .. }
                )
            })
            .collect();
        if pass {
            assert_eq!(links.len(), 1);
            assert!(
                matches!(&links[0].request, UiDispatchHostRequestKind::ActivateLink { target, link_target }
                if *target == parent && link_target.matches_display("res://docs/pass-parent.md"))
            );
            assert_eq!(up.reply.handler, Some(parent));
            assert_eq!(up.reply.phase, Some(UiDispatchPhase::DefaultAction));
            assert_eq!(
                up.diagnostics.handled_phase.as_deref(),
                Some("pointer.rich_link_activation")
            );
            let index = links[0].effect_index;
            assert!(
                matches!(&up.reply.effects[index], UiDispatchEffect::RequestLinkActivation { target, link_target }
                if *target == parent && link_target.matches_display("res://docs/pass-parent.md"))
            );
            assert_eq!(
                up.applied_effects
                    .iter()
                    .filter(|effect| effect.effect_index == index
                        && effect.effect == up.reply.effects[index])
                    .count(),
                1
            );
            assert_eq!(up.host_requests.len(), 2);
        } else {
            assert!(links.is_empty());
            assert_eq!(up.host_requests.len(), 1);
        }
    }
}

#[test]
fn pass_projection_stops_at_auto_owner_and_preserves_normal_ancestor_focus() {
    use crate::ui::text::layout_text;
    use zircon_runtime_interface::ui::{
        surface::{
            UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat,
            UiTextOverflow, UiTextWrap,
        },
        tree::{UiCursor, UiPointerEvents},
    };

    for pass in [true, false] {
        let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.pass-authority"));
        let frame = UiFrame::new(0.0, 0.0, 120.0, 40.0);
        surface.tree.insert_root(
            UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
                .with_frame(frame)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        );
        for id in 2..=4 {
            let mut flags = input_state();
            flags.clickable = false;
            flags.focusable = false;
            let mut node = UiTreeNode::new(
                UiNodeId::new(id),
                UiNodePath::new(format!("root/authority/{id}")),
            )
            .with_frame(frame)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(flags);
            node.pointer_events = if pass && id >= 3 {
                UiPointerEvents::Pass
            } else {
                UiPointerEvents::Auto
            };
            node.z_index = id as i32;
            surface
                .tree
                .insert_child(UiNodeId::new(id - 1), node)
                .unwrap();
        }
        surface.rebuild();
        let mut style = UiResolvedStyle::default();
        style.rich_text_format = UiRichTextFormat::HtmlSubsetV1;
        style.wrap = UiTextWrap::None;
        style.text_overflow = UiTextOverflow::Clip;
        let markup = "<a href=\"res://docs/blocked-ancestor.md\">help</a>";
        let layout = layout_text(markup, &style, frame, None);
        let line = &layout.lines[0];
        let point = UiPoint::new(line.frame.x + 1.0, line.frame.y + line.frame.height * 0.5);
        let mut extract = std::mem::take(&mut surface.render_extract);
        extract
            .list
            .commands
            .retain(|command| command.node_id != UiNodeId::new(1));
        extract.list.commands.push(UiRenderCommand {
            node_id: UiNodeId::new(1),
            kind: UiRenderCommandKind::Text,
            frame,
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: Some(layout),
            text: Some(markup.to_string()),
            image: None,
            opacity: 1.0,
        });
        surface.render_extract = surface
            .render_cache
            .update(&surface.render_extract, extract, false)
            .extract;
        let bubbles = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&bubbles);
        let mut dispatcher = UiPointerDispatcher::default();
        dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
            UiPointerDispatchEffect::Unhandled
        });
        dispatcher.register(UiNodeId::new(4), UiPointerEventKind::Up, |context| {
            assert_eq!(context.route.click_target, None);
            UiPointerDispatchEffect::Unhandled
        });
        for kind in [UiPointerEventKind::Down, UiPointerEventKind::Up] {
            let result = surface
                .dispatch_input_event(
                    &dispatcher,
                    &UiNavigationDispatcher::default(),
                    pointer_event(kind, point),
                )
                .unwrap();
            let routing = result.pointer_routing.as_ref().unwrap();
            assert_eq!(routing.physical_hit_path.target, Some(UiNodeId::new(4)));
            assert_eq!(routing.route_target, Some(UiNodeId::new(4)));
            assert_eq!(
                routing.physical_hit_path.root_to_leaf,
                vec![
                    UiNodeId::new(1),
                    UiNodeId::new(2),
                    UiNodeId::new(3),
                    UiNodeId::new(4)
                ]
            );
            assert_eq!(
                surface.focus.focused,
                if pass { None } else { Some(UiNodeId::new(1)) }
            );
            if kind == UiPointerEventKind::Down {
                assert_eq!(
                    surface.focus.pressed,
                    Some(UiNodeId::new(if pass { 2 } else { 4 }))
                );
            } else {
                assert_eq!(surface.focus.pressed, None);
            }
            assert!(!result.component_events.iter().any(|report| matches!(
                &report.event,
                UiComponentEvent::Commit { property, .. } if property == "activated"
            )));
            assert!(!result.host_requests.iter().any(|request| matches!(
                request.request,
                UiDispatchHostRequestKind::ActivateLink { .. }
            )));
            assert_pass_cursor_index(&result, UiCursor::Default);
        }
        assert_eq!(bubbles.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn nonclickable_owner_keeps_press_and_explicit_capture_without_click() {
    use zircon_runtime_interface::ui::tree::UiCursor;

    let mut surface = route_surface();
    for node in surface.tree.nodes.values_mut() {
        node.state_flags.focusable = false;
        node.state_flags.clickable = false;
    }
    surface.rebuild();
    let moves = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&moves);
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::CapturePointer
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, move |context| {
        assert_eq!(context.route.target, Some(UiNodeId::new(2)));
        assert_eq!(context.route.captured, Some(UiNodeId::new(2)));
        observed.fetch_add(1, Ordering::SeqCst);
        UiPointerDispatchEffect::Unhandled
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Up, |context| {
        assert_eq!(context.route.click_target, None);
        UiPointerDispatchEffect::Unhandled
    });
    for (kind, point) in [
        (UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        (UiPointerEventKind::Move, UiPoint::new(500.0, 500.0)),
        (UiPointerEventKind::Up, UiPoint::new(500.0, 500.0)),
    ] {
        let result = surface
            .dispatch_input_event(
                &dispatcher,
                &UiNavigationDispatcher::default(),
                pointer_event(kind, point),
            )
            .unwrap();
        assert_eq!(
            result.pointer_routing.as_ref().unwrap().route_target,
            Some(UiNodeId::new(2))
        );
        assert_eq!(surface.focus.focused, None);
        if kind == UiPointerEventKind::Up {
            assert_eq!(surface.focus.pressed, None);
            assert_eq!(surface.focus.captured, None);
            assert_eq!(
                surface.input.pointer_capture_owner(UiPointerId::new(7)),
                None
            );
        } else {
            assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
            assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
        }
        assert!(!result.component_events.iter().any(|report| {
            matches!(
                &report.event,
                UiComponentEvent::Commit { property, .. } if property == "activated"
            ) || matches!(&report.event, UiComponentEvent::Focus { .. })
        }));
        assert!(!result.host_requests.iter().any(|request| matches!(
            request.request,
            UiDispatchHostRequestKind::ActivateLink { .. }
        )));
        assert_pass_cursor_index(&result, UiCursor::Default);
    }
    assert_eq!(moves.load(Ordering::SeqCst), 1);
}
