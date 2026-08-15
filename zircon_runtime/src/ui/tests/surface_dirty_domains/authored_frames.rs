use super::*;

#[test]
fn authored_frame_publication_seeds_local_incremental_geometry() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.authored_frames"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_layout_boundary(LayoutBoundary::ParentDirected),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(button_id(), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(8.0, 10.0, 40.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();

    surface.rebuild_authored_frames(root_size());
    assert!(!surface.dirty_flags().any());
    assert_eq!(
        surface.hit_test(UiPoint::new(12.0, 14.0)).top_hit,
        Some(button_id())
    );

    let button = surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist");
    button.constraints = BoxConstraints {
        width: fixed_constraint(20.0),
        height: fixed_constraint(20.0),
    };
    button.position = Position::new(72.0, 10.0);
    surface
        .invalidate_node(button_id(), UiInvalidationReason::Layout)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(surface.hit_test(UiPoint::new(12.0, 14.0)).top_hit, None);
    assert_eq!(
        surface.hit_test(UiPoint::new(76.0, 14.0)).top_hit,
        Some(button_id())
    );
}
