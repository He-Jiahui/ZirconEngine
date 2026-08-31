use super::*;

#[test]
fn taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_button_surface();
    surface.compute_layout(UiSize::new(124.0, 40.0)).unwrap();
    let point = UiPoint::new(48.0, 12.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy flex pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy frame");

    assert_eq!(arranged_front.frame, UiFrame::new(44.0, 0.0, 80.0, 40.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(
        frame_hit.path.bubble_route().collect::<Vec<_>>(),
        vec![FRONT_ID, ROOT_ID]
    );

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(
            context.route.hit_path.bubble_route().collect::<Vec<_>>(),
            vec![FRONT_ID, ROOT_ID]
        );
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_linear_slot_sizing_button_surface();
    surface.compute_layout(UiSize::new(180.0, 40.0)).unwrap();
    let point = UiPoint::new(140.0, 12.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_back = frame
        .arranged_tree
        .get(BACK_ID)
        .expect("back control should be arranged by Taffy slot sizing");
    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by Taffy slot sizing");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy slot-sizing frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy slot-sizing frame");

    assert_eq!(arranged_back.frame, UiFrame::new(0.0, 0.0, 120.0, 40.0));
    assert_eq!(arranged_front.frame, UiFrame::new(120.0, 0.0, 60.0, 40.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(
        frame_hit.path.bubble_route().collect::<Vec<_>>(),
        vec![FRONT_ID, ROOT_ID]
    );

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(
            context.route.hit_path.bubble_route().collect::<Vec<_>>(),
            vec![FRONT_ID, ROOT_ID]
        );
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy slot-sizing hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_vertical_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_vertical_flex_linear_slot_sizing_button_surface();
    surface.compute_layout(UiSize::new(60.0, 180.0)).unwrap();
    let point = UiPoint::new(30.0, 150.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_back = frame
        .arranged_tree
        .get(BACK_ID)
        .expect("back control should be arranged by Taffy vertical slot sizing");
    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by Taffy vertical slot sizing");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy vertical slot-sizing frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy vertical slot-sizing frame");

    assert_eq!(arranged_back.frame, UiFrame::new(0.0, 0.0, 60.0, 120.0));
    assert_eq!(arranged_front.frame, UiFrame::new(0.0, 120.0, 60.0, 60.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(
        frame_hit.path.bubble_route().collect::<Vec<_>>(),
        vec![FRONT_ID, ROOT_ID]
    );

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(
            context.route.hit_path.bubble_route().collect::<Vec<_>>(),
            vec![FRONT_ID, ROOT_ID]
        );
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy vertical slot-sizing hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_flex_slot_policy_fallback_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_slot_policy_fallback_button_surface();
    surface.compute_layout(UiSize::new(124.0, 40.0)).unwrap();
    let point = UiPoint::new(30.0, 24.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Zircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Zircon flex fallback");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Zircon flex fallback frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Zircon flex fallback frame");

    assert_eq!(arranged_front.frame, UiFrame::new(10.0, 19.0, 40.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(
        frame_hit.path.bubble_route().collect::<Vec<_>>(),
        vec![FRONT_ID, ROOT_ID]
    );

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(
            context.route.hit_path.bubble_route().collect::<Vec<_>>(),
            vec![FRONT_ID, ROOT_ID]
        );
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Zircon flex fallback hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}
