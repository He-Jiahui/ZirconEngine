use super::*;

#[test]
fn overlay_layout_consumes_slot_padding_alignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.overlay"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::Overlay),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/badge")).with_constraints(
                BoxConstraints {
                    width: AxisConstraint::default(),
                    height: fixed_constraint(12.0),
                },
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(8.0, 6.0, 12.0, 10.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Fill, UiAlignment::End)),
    );

    surface.compute_layout(UiSize::new(160.0, 80.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(8.0, 58.0, 140.0, 12.0)
    );
}

#[test]
fn overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.overlay.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::Overlay)
            .with_clip_to_bounds(true)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/back",
                "back.overlay",
                BoxConstraints {
                    width: fixed_constraint(90.0),
                    height: fixed_constraint(50.0),
                },
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                3,
                "root/front",
                "front.overlay",
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
                10,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(4.0, 4.0, 4.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Start))
            .with_z_order(20),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(10.0, 8.0, 10.0, 8.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::End, UiAlignment::End))
            .with_z_order(1),
    );

    surface.compute_layout(UiSize::new(120.0, 80.0)).unwrap();
    let frame = surface.surface_frame();
    let front = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("front overlay child should be arranged");
    let back = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("back overlay child should be arranged");

    assert_eq!(front.frame, UiFrame::new(70.0, 52.0, 40.0, 20.0));
    assert_eq!(front.clip_frame, UiFrame::new(70.0, 52.0, 40.0, 20.0));
    assert_eq!(back.z_index, 20);
    assert_eq!(front.z_index, 11);
    assert_eq!(
        surface.tree.node(UiNodeId::new(2)).unwrap().z_index,
        0,
        "slot z_order should affect arranged output without mutating node z_index"
    );
    assert_eq!(render_z_for(&frame, UiNodeId::new(2)), Some(back.z_index));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(front.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(front.frame));
    assert_eq!(hit_z_for(&frame, UiNodeId::new(3)), Some(front.z_index));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(75.0, 53.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(2)));
    assert_eq!(hit.stacked, vec![UiNodeId::new(2), UiNodeId::new(3)]);
    assert_eq!(
        hit.path.bubble_route().collect::<Vec<_>>(),
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.scroll.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                gap: 0.0,
                scrollbar_visibility: UiScrollbarVisibility::Auto,
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 40.0,
                    overscan: 0,
                }),
            }))
            .with_scroll_state(UiScrollState {
                offset: 80.0,
                viewport_extent: 0.0,
                content_extent: 0.0,
            })
            .with_clip_to_bounds(true)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                pointer_node(
                    10 + item,
                    format!("root/item_{item}"),
                    format!("scroll.item.{item}"),
                    BoxConstraints {
                        width: fixed_constraint(200.0),
                        height: fixed_constraint(40.0),
                    },
                    item as i32,
                ),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();
    let frame = surface.surface_frame();
    let visible = frame
        .arranged_tree
        .get(UiNodeId::new(12))
        .expect("scrolled item should be arranged in the virtual window");

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 2,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(visible.frame, UiFrame::new(0.0, 0.0, 200.0, 40.0));
    assert_eq!(visible.clip_frame, UiFrame::new(0.0, 0.0, 200.0, 40.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(12)),
        Some(visible.frame)
    );
    assert_eq!(
        hit_frame_for(&frame, UiNodeId::new(12)),
        Some(visible.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(10)), None);

    let hit = hit_test_surface_frame(&frame, UiPoint::new(20.0, 20.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(12)));
}
