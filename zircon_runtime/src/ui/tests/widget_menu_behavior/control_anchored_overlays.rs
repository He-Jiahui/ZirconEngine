use super::*;

#[test]
fn control_anchored_popup_routes_rendered_menu_items_and_rejects_old_placeholder_hits() {
    let mut surface = control_anchored_menu_surface();
    surface.focus_node(UiNodeId::new(4)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild();

    let dispatcher = UiPointerDispatcher::default();
    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(110.0, 55.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.route.target, Some(UiNodeId::new(3)));
    assert_eq!(down.route.hit_path.target, Some(UiNodeId::new(3)));
    assert_eq!(down.route.point, UiPoint::new(110.0, 55.0));
    assert_popup_open(&surface, true);

    let inside_rendered_popup = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(110.0, 55.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_popup_open(&surface, false);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert!(inside_rendered_popup.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(3)
            && matches!(
                &event.envelope.event,
                UiComponentEvent::Commit { property, value }
                    if property == "activated" && value == &UiValue::Bool(true)
            )
    }));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild();

    let inside_old_placeholder = click_point(&mut surface, UiPoint::new(20.0, 16.0));
    assert_popup_open(&surface, false);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert!(inside_old_placeholder.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert!(inside_old_placeholder.component_events.iter().all(|event| {
        !matches!(
            &event.envelope.event,
            UiComponentEvent::Commit { property, .. } if property == "activated"
        )
    }));
}

#[test]
fn control_anchored_popup_frame_hit_grid_is_the_instance_authority() {
    let mut surface = control_anchored_menu_surface();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild();

    let frame = surface.surface_frame();
    let rendered_popup_point = UiPoint::new(110.0, 55.0);
    let frame_hit = hit_test_surface_frame(&frame, rendered_popup_point);
    let instance_hit = surface.hit_test(rendered_popup_point);

    assert_eq!(frame_hit, instance_hit);
    assert_eq!(frame_hit.top_hit, Some(UiNodeId::new(3)));
    assert!(frame_hit
        .path
        .bubble_route()
        .any(|node_id| node_id == UiNodeId::new(2)));

    let old_placeholder_hit = hit_test_surface_frame(&frame, UiPoint::new(20.0, 16.0));
    assert!(!old_placeholder_hit
        .path
        .bubble_route()
        .any(|node_id| node_id == UiNodeId::new(2)));

    let virtual_pointer = UiVirtualPointerPosition::new(
        rendered_popup_point,
        UiPoint::new(rendered_popup_point.x - 1.0, rendered_popup_point.y - 1.0),
    );
    let query =
        UiHitTestQuery::new(UiPoint::new(300.0, 200.0)).with_virtual_pointer(virtual_pointer);
    let frame_query_hit = hit_test_surface_frame_with_query(&frame, query.clone());
    let instance_query_hit = surface.hit_test_with_query(query);

    assert_eq!(frame_query_hit, instance_query_hit);
    assert_eq!(frame_query_hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(frame_query_hit.path.virtual_pointer, Some(virtual_pointer));
}

#[test]
fn parent_input_policy_incremental_patch_updates_descendant_frame_authority() {
    let mut surface = control_anchored_menu_surface();
    surface
        .tree
        .node_mut(UiNodeId::new(1))
        .expect("root should exist")
        .input_policy = UiInputPolicy::Receive;
    surface
        .tree
        .node_mut(UiNodeId::new(4))
        .expect("trigger should exist")
        .input_policy = UiInputPolicy::Inherit;
    surface.rebuild();

    let trigger_point = UiPoint::new(110.0, 12.0);
    assert_eq!(
        surface.hit_test(trigger_point).top_hit,
        Some(UiNodeId::new(4))
    );

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "input_policy",
            UiValue::Enum("ignore".to_string()),
        ))
        .unwrap();
    surface.rebuild_dirty(UiSize::new(180.0, 110.0)).unwrap();

    let frame_hit = hit_test_surface_frame(&surface.surface_frame(), trigger_point);
    let instance_hit = surface.hit_test(trigger_point);
    assert_eq!(frame_hit, instance_hit);
    assert!(!frame_hit.stacked.contains(&UiNodeId::new(4)));
}

#[test]
fn control_anchored_popup_escape_dismissal_restores_trigger_focus() {
    let mut surface = control_anchored_menu_surface();
    surface.focus_node(UiNodeId::new(4)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Escape", 27)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_popup_open(&surface, false);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2) && matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
}

#[test]
fn control_anchored_dropdown_routes_actual_option_overlay_and_dismisses_old_placeholder() {
    let mut surface = control_anchored_dropdown_surface();
    surface.focus_node(UiNodeId::new(4)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild();

    let dispatcher = UiPointerDispatcher::default();
    let inside_option_overlay = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(110.0, 55.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(inside_option_overlay.route.target, Some(UiNodeId::new(2)));
    assert_eq!(
        inside_option_overlay.route.hit_path.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(inside_option_overlay.route.point, UiPoint::new(110.0, 55.0));
    assert_popup_open(&surface, true);

    let mut release_surface = control_anchored_dropdown_surface();
    release_surface.focus_node(UiNodeId::new(4)).unwrap();
    release_surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    release_surface.rebuild();
    let inside_option_overlay_release = release_surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(110.0, 55.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(
        inside_option_overlay_release.route.target,
        Some(UiNodeId::new(2))
    );
    assert_popup_open(&release_surface, true);
    assert!(inside_option_overlay_release
        .component_events
        .iter()
        .all(|event| { !matches!(&event.envelope.event, UiComponentEvent::ClosePopup) }));

    let old_placeholder = click_point(&mut surface, UiPoint::new(20.0, 16.0));
    assert_popup_open(&surface, false);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert!(old_placeholder.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
}
