use super::*;

#[test]
fn zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = zircon_size_box_button_surface();
    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();
    let point = UiPoint::new(40.0, 60.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(
        root_selection.request.family,
        UiLayoutEngineFamily::Container
    );
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Zircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Zircon SizeBox fallback");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Zircon SizeBox frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Zircon SizeBox frame");

    assert_eq!(arranged_front.frame, UiFrame::new(30.0, 54.0, 40.0, 16.0));
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
        .expect("pointer dispatch should route through the Zircon SizeBox-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}
