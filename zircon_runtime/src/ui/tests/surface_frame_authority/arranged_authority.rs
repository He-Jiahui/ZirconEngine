use super::*;

#[test]
fn surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority() {
    let mut surface = overlapping_button_surface();
    let point = UiPoint::new(48.0, 36.0);
    let frame = surface.surface_frame();

    assert_eq!(frame.tree_id, UiTreeId::new("surface.frame.authority"));
    assert_eq!(frame.arranged_tree.tree_id, frame.tree_id);
    assert_eq!(frame.render_extract.tree_id, frame.tree_id);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should be rendered from the arranged tree");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should be entered into the hit grid");

    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(render_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(
        hit_front.clip_frame,
        arranged_front
            .frame
            .intersection(arranged_front.clip_frame)
            .expect("front arranged frame should intersect its clip")
    );
    assert_eq!(hit_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.paint_order, arranged_front.paint_order);
    assert_eq!(hit_front.control_id.as_deref(), Some("front.button"));

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.stacked, vec![FRONT_ID, BACK_ID]);
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the same surface hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.target, frame_hit.path.target);
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn surface_frame_focus_path_uses_arranged_authority() {
    let mut surface = overlapping_button_surface();
    surface.focus_node(FRONT_ID).unwrap();

    let frame = surface.surface_frame();
    let frame_hit = hit_test_surface_frame(&frame, UiPoint::new(48.0, 36.0));

    assert_eq!(frame.focus_state.focused, Some(FRONT_ID));
    assert_eq!(frame.focus_path.focused, Some(FRONT_ID));
    assert_eq!(frame.focus_path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame.focus_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
    assert_eq!(surface.focused_route(), frame.focus_path.bubble_route);
    assert_eq!(frame_hit.path.root_to_leaf, frame.focus_path.root_to_leaf);
    assert_eq!(frame_hit.path.bubble_route, frame.focus_path.bubble_route);
}
