use super::*;

#[test]
fn taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_wrap_button_surface();
    surface.compute_layout(UiSize::new(90.0, 44.0)).unwrap();
    let point = UiPoint::new(24.0, 28.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Wrap);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy wrap pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy wrap frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy wrap frame");

    assert_eq!(arranged_front.frame, UiFrame::new(0.0, 22.0, 50.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
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
        .expect("pointer dispatch should route through the Taffy wrap-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_grid_slot_button_surface();
    surface.compute_layout(UiSize::new(124.0, 82.0)).unwrap();
    let point = UiPoint::new(80.0, 65.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Grid);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy grid pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy grid frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy grid frame");

    assert_eq!(arranged_front.frame, UiFrame::new(73.0, 61.0, 40.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
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
        .expect("pointer dispatch should route through the Taffy grid-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}
