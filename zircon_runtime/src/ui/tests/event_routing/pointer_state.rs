use super::*;

#[test]
fn primary_release_inside_pressed_target_marks_click_target_and_clears_press_state() {
    let mut surface = button_surface();

    let down = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(down.target, Some(UiNodeId::new(2)));
    assert_eq!(down.pressed, Some(UiNodeId::new(2)));
    assert_eq!(down.click_target, None);
    assert!(matches!(
        &down.routing_path,
        zircon_runtime_interface::ui::surface::UiPointerRoutingPath::HitPath
    ));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, Some(UiNodeId::new(2)));
    assert!(up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn primary_release_outside_pressed_target_does_not_mark_click_target() {
    let mut surface = button_surface();

    surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();
    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn captured_release_uses_hit_path_not_capture_target_for_click_target() {
    let mut surface = button_surface();
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.target, Some(UiNodeId::new(2)));
    assert_eq!(up.hit_path.target, None);
    assert!(matches!(
        &up.routing_path,
        zircon_runtime_interface::ui::surface::UiPointerRoutingPath::ExplicitRootToLeaf(_)
    ));
    assert_eq!(
        up.bubble_route().collect::<Vec<_>>(),
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn pointer_route_can_use_virtual_pointer_hit_from_custom_surface_mapper() {
    let mut surface = button_surface();
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));
    let route = surface
        .route_pointer_event_with_query_and_button(
            UiPointerEventKind::Down,
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(route.point, UiPoint::new(20.0, 20.0));
    assert_eq!(route.target, Some(UiNodeId::new(2)));
    assert_eq!(route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
}

#[test]
fn pointer_dispatch_uses_virtual_pointer_query_for_component_events() {
    let mut surface =
        bound_button_surface(vec![binding("Showcase/ButtonPress", UiEventKind::Press)]);
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));

    let result = surface
        .dispatch_pointer_event_with_query(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(140.0, 80.0))
                .with_button(UiPointerButton::Primary),
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
        )
        .unwrap();

    assert_eq!(result.route.target, Some(UiNodeId::new(2)));
    assert_eq!(result.route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].binding_id,
        "Showcase/ButtonPress"
    );
}

#[test]
fn pointer_dispatch_result_reports_same_target_hover_as_idle_diagnostic() {
    let mut surface = button_surface();

    let first = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert!(!first.diagnostics.ignored_same_target_hover);
    assert_eq!(first.diagnostics.hover_entered, 1);

    let second = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(25.0, 25.0)),
        )
        .unwrap();

    assert!(second.diagnostics.pointer_routed);
    assert!(second.diagnostics.ignored_same_target_hover);
    assert_eq!(second.diagnostics.hover_entered, 0);
    assert_eq!(second.diagnostics.hover_left, 0);
}

#[test]
fn repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface() {
    let mut surface = button_surface();

    let first = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert_eq!(
        first.requested_damage,
        vec![UiFrame::new(10.0, 10.0, 80.0, 30.0)]
    );
    assert_render_only_dirty(surface.dirty_flags());
    let first_hover_rebuild = surface.rebuild_dirty(UiSize::new(160.0, 100.0)).unwrap();
    assert!(first_hover_rebuild.render_rebuilt);
    assert!(!first_hover_rebuild.layout_recomputed);
    assert!(!first_hover_rebuild.arranged_rebuilt);
    assert!(!first_hover_rebuild.hit_grid_rebuilt);
    let steady_rebuild = surface.last_rebuild_report;

    for offset in 0..100 {
        let point = UiPoint::new(21.0 + (offset % 8) as f32, 21.0);
        let result = surface
            .dispatch_pointer_event(
                &crate::ui::dispatch::UiPointerDispatcher::default(),
                UiPointerEvent::new(UiPointerEventKind::Move, point),
            )
            .unwrap();
        assert!(result.diagnostics.ignored_same_target_hover);
        assert!(result.requested_damage.is_empty());
        assert!(result.component_events.is_empty());
        assert_eq!(surface.last_rebuild_report, steady_rebuild);
        assert!(!surface.dirty_flags().any());
    }
}

#[test]
fn pointer_dispatch_syncs_pressed_state_as_render_only_dirty() {
    let mut surface = button_surface();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );
    assert_render_only_dirty(surface.dirty_flags());

    let rebuild = surface.rebuild_dirty(UiSize::new(160.0, 100.0)).unwrap();
    assert_eq!(rebuild.dirty_node_count, 1);
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
    assert!(!surface.dirty_flags().any());

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );
    assert_render_only_dirty(surface.dirty_flags());
}

#[test]
fn pointer_dispatch_clears_previous_pressed_state_when_primary_press_moves_target() {
    let mut surface = two_button_surface(None, None);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.clear_dirty_flags();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 60.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );
    assert!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .state_flags
            .pressed
    );
    assert_render_only_dirty(surface.dirty_flags());
    assert_eq!(
        surface
            .tree
            .nodes
            .values()
            .filter(|node| node.dirty.render)
            .count(),
        2
    );
}

#[test]
fn pointer_dispatch_reduces_hover_focus_and_press_into_component_state_store() {
    let mut surface = two_button_surface(None, None);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    let first_state = surface.component_state(UiNodeId::new(2)).unwrap();
    assert!(first_state.flags.hovered);
    assert!(!first_state.flags.focused);
    assert!(!first_state.flags.pressed);
    assert_render_only_dirty(surface.dirty_flags());
    surface.clear_dirty_flags();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let first_state = surface.component_state(UiNodeId::new(2)).unwrap();
    assert!(first_state.flags.hovered);
    assert!(first_state.flags.focused);
    assert!(first_state.flags.pressed);
    surface.clear_dirty_flags();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 60.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let first_state = surface.component_state(UiNodeId::new(2)).unwrap();
    let second_state = surface.component_state(UiNodeId::new(3)).unwrap();
    assert!(!first_state.flags.hovered);
    assert!(!first_state.flags.focused);
    assert!(!first_state.flags.pressed);
    assert!(second_state.flags.hovered);
    assert!(second_state.flags.focused);
    assert!(second_state.flags.pressed);
    assert_render_only_dirty(surface.dirty_flags());
}
