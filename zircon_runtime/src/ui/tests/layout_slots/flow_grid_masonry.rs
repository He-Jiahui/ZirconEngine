use super::*;

#[test]
fn wrap_flow_slot_padding_alignment_feeds_shared_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.flow.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::WrapBox(zircon_runtime_interface::ui::layout::UiWrapBoxConfig {
                horizontal_gap: 5.0,
                vertical_gap: 3.0,
                item_min_width: 20.0,
            }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/flow_first",
                "flow.first",
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(10.0),
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
                "root/flow_second",
                "flow.second",
                BoxConstraints {
                    width: fixed_constraint(25.0),
                    height: fixed_constraint(12.0),
                },
                5,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Flow)
            .with_padding(UiMargin::new(2.0, 1.0, 4.0, 3.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End))
            .with_order(2),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Flow)
            .with_padding(UiMargin::new(1.0, 2.0, 1.0, 2.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Fill))
            .with_order(1),
    );

    surface.compute_layout(UiSize::new(70.0, 40.0)).unwrap();
    let frame = surface.surface_frame();
    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("flow child should be arranged");
    let second = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("flow child should be arranged");

    assert_eq!(second.frame, UiFrame::new(1.0, 2.0, 25.0, 12.0));
    assert_eq!(first.frame, UiFrame::new(34.0, 3.0, 30.0, 10.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(2)),
        Some(first.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(2)), Some(first.frame));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(second.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(second.frame));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(4.0, 5.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
}

#[test]
fn grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.grid.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::GridBox(UiGridBoxConfig {
                columns: 2,
                rows: 2,
                column_gap: 4.0,
                row_gap: 6.0,
            }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/grid_a",
                "grid.a",
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(20.0),
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
                "root/grid_b",
                "grid.b",
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
                5,
            ),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0))
            .with_padding(UiMargin::new(2.0, 2.0, 2.0, 2.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Center)),
    );
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1))
            .with_padding(UiMargin::new(4.0, 4.0, 4.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Start)),
    );

    surface.compute_layout(UiSize::new(124.0, 82.0)).unwrap();
    let frame = surface.surface_frame();
    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("grid child should be arranged");
    let second = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("grid child should be arranged");

    assert_eq!(first.frame, UiFrame::new(15.0, 9.0, 30.0, 20.0));
    assert_eq!(second.frame, UiFrame::new(68.0, 48.0, 40.0, 20.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(2)),
        Some(first.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(2)), Some(first.frame));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(second.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(second.frame));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(70.0, 50.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
}

#[test]
fn masonry_shortest_column_layout_feeds_arranged_render_hit_from_one_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.masonry.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::MasonryBox(UiMasonryBoxConfig {
                columns: 2,
                gap: 4.0,
                sequential: false,
            }),
        ),
    );
    for (id, height) in [(2, 20.0), (3, 40.0), (4, 30.0), (5, 10.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                pointer_node(
                    id,
                    format!("root/masonry_{id}"),
                    format!("masonry.{id}"),
                    BoxConstraints {
                        width: fixed_constraint(48.0),
                        height: fixed_constraint(height),
                    },
                    id as i32,
                ),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(100.0, 80.0)).unwrap();
    let frame = surface.surface_frame();
    let report = &frame.layout_engine_report;
    let masonry_selection = report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("masonry route should be reported");

    assert_eq!(
        masonry_selection.request.family,
        UiLayoutEngineFamily::Masonry
    );
    assert_eq!(
        masonry_selection.selected_backend,
        UiLayoutEngineBackend::Zircon
    );
    assert_eq!(masonry_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .content_size,
        UiSize::new(100.0, 54.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 48.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(52.0, 0.0, 48.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 24.0, 48.0, 30.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(5))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(52.0, 44.0, 48.0, 10.0)
    );
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(5)),
        Some(UiFrame::new(52.0, 44.0, 48.0, 10.0))
    );
    assert_eq!(
        hit_frame_for(&frame, UiNodeId::new(5)),
        Some(UiFrame::new(52.0, 44.0, 48.0, 10.0))
    );

    let hit = hit_test_surface_frame(&frame, UiPoint::new(54.0, 45.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(5)));
}

#[test]
fn masonry_sequential_layout_preserves_ordered_column_assignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.masonry.sequential"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::MasonryBox(UiMasonryBoxConfig {
                columns: 2,
                gap: 4.0,
                sequential: true,
            }),
        ),
    );
    for (id, height) in [(2, 60.0), (3, 10.0), (4, 10.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(id),
                    UiNodePath::new(format!("root/item_{id}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(48.0),
                    height: fixed_constraint(height),
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 64.0, 48.0, 10.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .content_size,
        UiSize::new(100.0, 74.0)
    );
}
