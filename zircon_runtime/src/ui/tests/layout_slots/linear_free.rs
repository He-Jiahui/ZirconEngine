use super::*;

#[test]
fn linear_layout_consumes_slot_padding_order_and_alignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.linear"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 4.0 }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/later")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(20.0),
                    height: fixed_constraint(10.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/first")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: AxisConstraint::default(),
                },
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Linear)
            .with_padding(UiMargin::new(5.0, 4.0, 7.0, 6.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End))
            .with_order(2),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Linear)
            .with_padding(UiMargin::new(1.0, 2.0, 3.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Fill))
            .with_order(1),
    );

    surface.compute_layout(UiSize::new(200.0, 60.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(70.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(1.0, 2.0, 30.0, 54.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(43.0, 44.0, 20.0, 10.0)
    );
}

#[test]
fn free_layout_consumes_explicit_slot_padding_alignment_and_preserves_default_anchor_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.free"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/anchored"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(20.0),
                })
                .with_anchor(zircon_runtime_interface::ui::layout::Anchor::new(0.5, 0.5))
                .with_pivot(zircon_runtime_interface::ui::layout::Pivot::new(0.5, 0.5))
                .with_position(zircon_runtime_interface::ui::layout::Position::new(
                    10.0, -5.0,
                )),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/slotted"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(10.0),
                })
                .with_position(zircon_runtime_interface::ui::layout::Position::new(
                    2.0, 3.0,
                )),
        )
        .unwrap();
    surface.tree.push_layout_slot(UiSlot::new(
        UiNodeId::new(1),
        UiNodeId::new(2),
        UiSlotKind::Free,
    ));
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Free)
            .with_padding(UiMargin::new(10.0, 5.0, 20.0, 15.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::End, UiAlignment::Center)),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(95.0, 35.0, 30.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(142.0, 43.0, 40.0, 10.0)
    );
}

#[test]
fn free_layout_consumes_canvas_slot_placement_before_child_default_anchor() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.free.canvas"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/slotted_size",
                "free.canvas.size",
                BoxConstraints::default(),
                0,
            )
            .with_anchor(Anchor::new(0.25, 0.25))
            .with_pivot(Pivot::new(0.5, 0.5))
            .with_position(Position::new(99.0, 77.0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                3,
                "root/auto_size",
                "free.canvas.auto",
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(10.0),
                },
                1,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Free).with_canvas_placement(
            UiCanvasSlotPlacement::new(
                Anchor::new(1.0, 0.5),
                Pivot::new(1.0, 0.5),
                Position::new(-10.0, 5.0),
            )
            .with_offset(UiMargin::new(2.0, 3.0, 80.0, 30.0)),
        ),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Free).with_canvas_placement(
            UiCanvasSlotPlacement::new(
                Anchor::new(0.5, 0.0),
                Pivot::new(0.5, 0.0),
                Position::new(4.0, 8.0),
            )
            .with_offset(UiMargin::new(10.0, 2.0, 200.0, 200.0))
            .with_auto_size(true),
        ),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();
    let frame = surface.surface_frame();
    let slotted = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("slotted child should be arranged");
    let auto = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("auto-sized child should be arranged");

    assert_eq!(slotted.frame, UiFrame::new(112.0, 43.0, 80.0, 30.0));
    assert_eq!(auto.frame, UiFrame::new(99.0, 10.0, 30.0, 10.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(2)),
        Some(slotted.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(2)), Some(slotted.frame));
    assert_eq!(
        hit_test_surface_frame(&frame, UiPoint::new(115.0, 45.0)).top_hit,
        Some(UiNodeId::new(2))
    );
}
