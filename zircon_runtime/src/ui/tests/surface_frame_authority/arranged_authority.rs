use super::*;
use std::sync::Arc;

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
    assert!(frame_hit
        .path
        .bubble_route()
        .eq(frame.focus_path.bubble_route.iter().copied()));
}

#[test]
fn focus_only_publication_reuses_unchanged_heavy_domains() {
    let mut surface = overlapping_button_surface();
    let before = surface.surface_frame();

    surface.focus_node(FRONT_ID).unwrap();
    let after = surface.surface_frame();

    assert!(Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(Arc::ptr_eq(&before.render_extract, &after.render_extract));
    assert!(Arc::ptr_eq(&before.hit_grid, &after.hit_grid));
    assert!(!Arc::ptr_eq(&before.focus_state, &after.focus_state));
    assert!(!Arc::ptr_eq(&before.focus_path, &after.focus_path));
    assert!(Arc::ptr_eq(&before.pipeline_report, &after.pipeline_report));
    assert_eq!(
        before.domain_generations.layout,
        after.domain_generations.layout,
    );
    assert_eq!(
        before.domain_generations.render,
        after.domain_generations.render,
    );
    assert_eq!(
        before.domain_generations.hit_test,
        after.domain_generations.hit_test,
    );
    assert!(after.domain_generations.focus > before.domain_generations.focus);
    assert_eq!(
        before.domain_generations.pipeline,
        after.domain_generations.pipeline,
    );
}

#[test]
fn render_only_rebuild_preserves_layout_and_hit_domains() {
    let mut surface = overlapping_button_surface();
    let root_size = UiSize::new(180.0, 120.0);
    surface.rebuild_authored_frames(root_size);
    let before = surface.surface_frame();

    surface.focus_node(FRONT_ID).unwrap();
    let report = surface.rebuild_dirty(root_size).unwrap();
    let after = surface.surface_frame();

    assert!(report.render_rebuilt);
    assert!(!report.layout_recomputed);
    assert!(Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(!Arc::ptr_eq(&before.render_extract, &after.render_extract));
    assert!(Arc::ptr_eq(&before.hit_grid, &after.hit_grid));
    assert!(!Arc::ptr_eq(&before.focus_state, &after.focus_state));
    assert!(!Arc::ptr_eq(&before.focus_path, &after.focus_path));
    assert!(!Arc::ptr_eq(
        &before.pipeline_report,
        &after.pipeline_report
    ));
    assert_eq!(
        before.domain_generations.layout,
        after.domain_generations.layout,
    );
    assert!(after.domain_generations.render > before.domain_generations.render);
    assert_eq!(
        before.domain_generations.hit_test,
        after.domain_generations.hit_test,
    );
    assert!(after.domain_generations.pipeline > before.domain_generations.pipeline);
}

#[test]
fn layout_only_resize_reuses_stable_focus_domains() {
    let mut surface = taffy_flex_button_surface();
    let initial_size = UiSize::new(124.0, 40.0);
    let resized = UiSize::new(240.0, 80.0);
    surface.rebuild_dirty(initial_size).unwrap();
    surface.focus_node(FRONT_ID).unwrap();
    surface.rebuild_dirty(initial_size).unwrap();
    let before = surface.surface_frame();

    let report = surface.rebuild_dirty(resized).unwrap();
    let after = surface.surface_frame();

    assert!(report.layout_recomputed);
    assert!(after.domain_generations.layout > before.domain_generations.layout);
    assert!(Arc::ptr_eq(&before.focus_state, &after.focus_state));
    assert!(Arc::ptr_eq(&before.focus_path, &after.focus_path));
    assert_eq!(
        before.domain_generations.focus,
        after.domain_generations.focus
    );
    assert_eq!(after.focus_path.focused, Some(FRONT_ID));
    assert_eq!(after.focus_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
}

#[test]
fn render_only_publication_reuses_untouched_command_segments() {
    const CHILD_COUNT: u64 = 130;
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.segmented_render"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 512.0, 512.0))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    for child_offset in 0..CHILD_COUNT {
        let node_id = UiNodeId::new(100 + child_offset);
        surface
            .tree
            .insert_child(
                ROOT_ID,
                button_node(
                    node_id,
                    &format!("root/item_{child_offset}"),
                    &format!("item.{child_offset}"),
                    UiFrame::new(0.0, child_offset as f32 * 2.0, 32.0, 2.0),
                    0,
                ),
            )
            .unwrap();
    }
    let root_size = UiSize::new(512.0, 512.0);
    surface.rebuild_authored_frames(root_size);
    let before = surface.surface_frame();
    let stable_id = UiNodeId::new(101);
    let changed_id = UiNodeId::new(225);
    let stable_index = before
        .render_extract
        .list
        .commands
        .iter()
        .position(|command| command.node_id == stable_id)
        .unwrap();
    let changed_index = before
        .render_extract
        .list
        .commands
        .iter()
        .position(|command| command.node_id == changed_id)
        .unwrap();
    assert_ne!(
        stable_index / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
        changed_index / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
    );

    surface.focus_node(changed_id).unwrap();
    let report = surface.rebuild_dirty(root_size).unwrap();
    let after = surface.surface_frame();

    assert!(report.render_rebuilt);
    assert!(std::ptr::eq(
        &before.render_extract.list.commands[stable_index],
        &after.render_extract.list.commands[stable_index],
    ));
    assert!(!std::ptr::eq(
        &before.render_extract.list.commands[changed_index],
        &after.render_extract.list.commands[changed_index],
    ));
}

#[test]
fn window_only_publication_reuses_all_heavy_domains() {
    let mut surface = overlapping_button_surface();
    let before = surface.surface_frame();

    surface.window_state.focused = Some(true);
    let after = surface.surface_frame();

    assert!(Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(Arc::ptr_eq(&before.render_extract, &after.render_extract));
    assert!(Arc::ptr_eq(&before.hit_grid, &after.hit_grid));
    assert!(Arc::ptr_eq(&before.focus_state, &after.focus_state));
    assert!(Arc::ptr_eq(&before.focus_path, &after.focus_path));
    assert!(Arc::ptr_eq(&before.pipeline_report, &after.pipeline_report));
    assert_eq!(
        before.domain_generations.layout,
        after.domain_generations.layout,
    );
    assert_eq!(
        before.domain_generations.render,
        after.domain_generations.render,
    );
    assert_eq!(
        before.domain_generations.hit_test,
        after.domain_generations.hit_test,
    );
    assert_eq!(
        before.domain_generations.pipeline,
        after.domain_generations.pipeline,
    );
    assert!(after.domain_generations.window > before.domain_generations.window);
}
