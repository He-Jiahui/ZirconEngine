use super::*;
use std::{collections::BTreeSet, sync::Arc};

#[test]
fn disjoint_ancestor_clips_remain_empty_in_published_render_and_hit_frames() {
    for (root_width, parent_width) in [
        (20.0, None),
        (100.0, Some(20.0)),
        (40.0, None),
        (100.0, Some(40.0)),
    ] {
        let mut surface = clipped_leaf_surface(root_width, parent_width);
        let frame = surface.surface_frame();
        let leaf = frame.arranged_tree.get(FRONT_ID).unwrap();
        assert_eq!(leaf.clip_frame, UiFrame::new(40.0, 40.0, 0.0, 0.0));
        assert_eq!(
            frame
                .render_extract
                .list
                .commands
                .iter()
                .find(|command| command.node_id == FRONT_ID)
                .unwrap()
                .clip_frame,
            Some(leaf.clip_frame),
        );
        assert_eq!(
            hit_test_surface_frame(&frame, UiPoint::new(45.0, 45.0)).top_hit,
            None,
        );
        assert_eq!(surface.hit_test(UiPoint::new(45.0, 45.0)).top_hit, None);

        for node_id in [Some(ROOT_ID), parent_width.map(|_| BACK_ID), Some(FRONT_ID)]
            .into_iter()
            .flatten()
        {
            surface.tree.node_mut(node_id).unwrap().clip_to_bounds = false;
        }
        surface.rebuild_authored_frames(UiSize::new(100.0, 100.0));
        let unclipped = surface.surface_frame();
        let leaf = unclipped.arranged_tree.get(FRONT_ID).unwrap();
        assert_eq!(leaf.clip_frame, leaf.frame);
        assert_eq!(
            hit_test_surface_frame(&unclipped, UiPoint::new(45.0, 45.0)).top_hit,
            Some(FRONT_ID),
        );
    }
}

#[test]
fn clipping_parent_geometry_patch_preserves_empty_clip_and_retained_frame() {
    let mut surface = clipped_leaf_surface(100.0, Some(60.0));
    let before = surface.surface_frame();
    let point = UiPoint::new(45.0, 45.0);
    assert_eq!(
        hit_test_surface_frame(&before, point).top_hit,
        Some(FRONT_ID)
    );
    let topology_generation = surface.tree.layout_order_generation();
    surface.tree.node_mut(BACK_ID).unwrap().layout_cache.frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);

    let publication = surface.publish_authored_geometry(
        UiSize::new(100.0, 100.0),
        &BTreeSet::from([BACK_ID]),
        topology_generation,
    );
    assert!(matches!(
        publication,
        crate::ui::surface::UiAuthoredGeometryPublication::Local(_)
    ));

    let after = surface.surface_frame();
    let leaf = after.arranged_tree.get(FRONT_ID).unwrap();
    assert_eq!(leaf.clip_frame, UiFrame::new(40.0, 40.0, 0.0, 0.0));
    assert_eq!(
        after
            .render_extract
            .list
            .commands
            .iter()
            .find(|command| command.node_id == FRONT_ID)
            .unwrap()
            .clip_frame,
        Some(leaf.clip_frame),
    );
    assert_eq!(hit_test_surface_frame(&after, point).top_hit, None);
    assert_eq!(surface.hit_test(point).top_hit, None);
    assert_eq!(
        hit_test_surface_frame(&before, point).top_hit,
        Some(FRONT_ID)
    );
    let full_surface = clipped_leaf_surface(100.0, Some(20.0));
    let full_frame = full_surface.surface_frame();
    assert_eq!(
        after.arranged_tree.get(FRONT_ID),
        full_frame.arranged_tree.get(FRONT_ID),
    );
    assert_eq!(
        hit_test_surface_frame(&after, point),
        hit_test_surface_frame(&full_frame, point),
    );
}

#[test]
fn clipped_component_children_keep_empty_scissors_through_paint_conversion() {
    for (component, attributes) in [
        ("AgentChat", "messages = [\"user|hidden\"]"),
        ("ChatComposer", "composer_text = \"hidden\""),
        (
            "TreeView",
            "text = \"header\"\ncollection_items = [\"selected|0|hidden\"]",
        ),
        ("InputField", "text = \"hidden\"\ncontent = \"hidden\""),
    ] {
        let mut surface = clipped_leaf_surface(20.0, None);
        let widget = if component == "InputField" {
            zircon_runtime_interface::ui::widget::UiWidgetContract {
                behavior: zircon_runtime_interface::ui::widget::UiWidgetBehavior::TextInput,
                value_property: Some("content".into()),
                ..Default::default()
            }
        } else {
            Default::default()
        };
        let leaf = surface.tree.node_mut(FRONT_ID).unwrap();
        leaf.layout_cache.frame = UiFrame::new(40.25, 40.25, 160.0, 80.0);
        leaf.template_metadata = Some(UiTemplateNodeMetadata {
            component: component.into(),
            attributes: toml::from_str(attributes).unwrap(),
            widget,
            ..Default::default()
        });
        surface.rebuild_authored_frames(UiSize::new(100.0, 100.0));
        let frame = surface.surface_frame();
        let leaf_commands = frame
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == FRONT_ID)
            .collect::<Vec<_>>();
        assert!(
            leaf_commands.iter().any(|command| {
                command.kind == zircon_runtime_interface::ui::surface::UiRenderCommandKind::Text
                    && command.text.as_deref() == Some("hidden")
            }),
            "{component} must render a child text command",
        );
        for command in leaf_commands {
            let clip = command
                .clip_frame
                .expect("clipped child must retain a scissor");
            assert!(clip.width <= 0.0 || clip.height <= 0.0, "{component}");
            for dpi_scale in [1.0, 1.25, 1.5, 2.0] {
                let elements = command.to_paint_elements_with_metrics(
                    0,
                    zircon_runtime_interface::ui::layout::UiLayoutMetrics {
                        dpi_scale,
                        ..Default::default()
                    },
                );
                assert!(!elements.is_empty(), "{component}");
                for element in elements {
                    let clip = element.clip.expect("paint must retain an empty scissor");
                    assert!(
                        clip.frame.width <= 0.0 || clip.frame.height <= 0.0,
                        "{component} at DPI {dpi_scale}",
                    );
                }
            }
        }
    }
}

fn clipped_leaf_surface(root_width: f32, parent_width: Option<f32>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.clipping"));
    let mut root = UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
        .with_frame(UiFrame::new(0.0, 0.0, root_width, root_width))
        .with_input_policy(UiInputPolicy::Ignore)
        .with_state_flags(root_state());
    root.clip_to_bounds = true;
    surface.tree.insert_root(root);
    let leaf_parent = if let Some(width) = parent_width {
        let mut parent = UiTreeNode::new(BACK_ID, UiNodePath::new("root/parent"))
            .with_frame(UiFrame::new(0.0, 0.0, width, width))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state());
        parent.clip_to_bounds = true;
        surface.tree.insert_child(ROOT_ID, parent).unwrap();
        BACK_ID
    } else {
        ROOT_ID
    };
    let mut leaf = button_node(
        FRONT_ID,
        "root/leaf",
        "leaf.button",
        UiFrame::new(40.0, 40.0, 20.0, 20.0),
        1,
    );
    leaf.clip_to_bounds = true;
    surface.tree.insert_child(leaf_parent, leaf).unwrap();
    surface.rebuild_authored_frames(UiSize::new(100.0, 100.0));
    surface
}

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
