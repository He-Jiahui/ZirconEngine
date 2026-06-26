use super::*;

#[test]
fn hit_testing_respects_z_order_input_policy_and_clip_chain() {
    let mut tree = UiTree::new(UiTreeId::new("runtime.ui"));
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/background"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_z_index(0)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/overlay_ignore"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
            .with_z_index(100)
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/clipped_parent"))
            .with_frame(UiFrame::new(0.0, 0.0, 60.0, 60.0))
            .with_clip_to_bounds(true)
            .with_z_index(10)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(4),
        UiTreeNode::new(
            UiNodeId::new(5),
            UiNodePath::new("root/clipped_parent/child"),
        )
        .with_frame(UiFrame::new(20.0, 20.0, 100.0, 100.0))
        .with_z_index(30)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: false,
            pressed: false,
            checked: false,
            dirty: false,
        }),
    )
    .unwrap();

    let mut hit_test = UiHitTestIndex::default();
    hit_test.rebuild(&tree);

    let clipped = hit_test.hit_test(&tree, UiPoint::new(80.0, 80.0));
    assert_eq!(clipped.top_hit, Some(UiNodeId::new(2)));
    assert_eq!(clipped.stacked, vec![UiNodeId::new(2), UiNodeId::new(1)]);

    let unclipped = hit_test.hit_test(&tree, UiPoint::new(40.0, 40.0));
    assert_eq!(unclipped.top_hit, Some(UiNodeId::new(5)));
    assert_eq!(
        unclipped.stacked,
        vec![
            UiNodeId::new(5),
            UiNodeId::new(4),
            UiNodeId::new(2),
            UiNodeId::new(1)
        ]
    );
}

#[test]
fn surface_rebuild_derives_render_and_hit_from_same_arranged_geometry() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    let root_frame = UiFrame::new(0.0, 0.0, 160.0, 80.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 5.0, 80.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("primary".to_string()),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let arranged = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    let hit_entry = surface
        .hit_test
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(arranged.frame, UiFrame::new(10.0, 5.0, 80.0, 20.0));
    assert_eq!(command.frame, arranged.frame);
    assert_eq!(hit_entry.frame, arranged.frame);
    assert_eq!(
        surface.hit_test(UiPoint::new(50.0, 15.0)).top_hit,
        Some(UiNodeId::new(2))
    );

    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(30.0, 25.0, 90.0, 24.0);
    surface.rebuild();

    let moved = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    let moved_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    let moved_hit_entry = surface
        .hit_test
        .grid
        .entries
        .iter()
        .find(|entry| entry.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(moved.frame, UiFrame::new(30.0, 25.0, 90.0, 24.0));
    assert_eq!(moved_command.frame, moved.frame);
    assert_eq!(moved_hit_entry.frame, moved.frame);
    assert_eq!(surface.hit_test(UiPoint::new(50.0, 15.0)).top_hit, None);
    assert_eq!(
        surface.hit_test(UiPoint::new(50.0, 35.0)).top_hit,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn hit_grid_respects_slate_visibility_and_clip_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    let root_frame = UiFrame::new(0.0, 0.0, 120.0, 120.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for (node_id, path, frame, z, visibility) in [
        (
            UiNodeId::new(2),
            "root/base",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            0,
            UiVisibility::Visible,
        ),
        (
            UiNodeId::new(3),
            "root/hidden",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            20,
            UiVisibility::Hidden,
        ),
        (
            UiNodeId::new(4),
            "root/collapsed",
            UiFrame::new(10.0, 10.0, 30.0, 20.0),
            30,
            UiVisibility::Collapsed,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_z_index(z)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_visibility(visibility)
                    .with_state_flags(pointer_state()),
            )
            .unwrap();
    }

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(5), UiNodePath::new("root/hit_test_invisible"))
                .with_frame(UiFrame::new(60.0, 10.0, 40.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_visibility(UiVisibility::HitTestInvisible)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(5),
            UiTreeNode::new(
                UiNodeId::new(6),
                UiNodePath::new("root/hit_test_invisible/child"),
            )
            .with_frame(UiFrame::new(65.0, 15.0, 25.0, 20.0))
            .with_z_index(50)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(7),
                UiNodePath::new("root/self_hit_test_invisible"),
            )
            .with_frame(UiFrame::new(50.0, 60.0, 50.0, 40.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_visibility(UiVisibility::SelfHitTestInvisible)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(7),
            UiTreeNode::new(
                UiNodeId::new(8),
                UiNodePath::new("root/self_hit_test_invisible/child"),
            )
            .with_frame(UiFrame::new(55.0, 65.0, 30.0, 20.0))
            .with_z_index(60)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(pointer_state()),
        )
        .unwrap();

    surface.rebuild();

    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 18.0)).top_hit,
        Some(UiNodeId::new(2)),
        "hidden and collapsed overlap should not displace the visible base button"
    );
    assert_eq!(
        surface.hit_test(UiPoint::new(70.0, 22.0)).top_hit,
        None,
        "HitTestInvisible should block its own subtree from hit testing"
    );
    assert_eq!(
        surface.hit_test(UiPoint::new(70.0, 72.0)).top_hit,
        Some(UiNodeId::new(8)),
        "SelfHitTestInvisible should skip the parent but preserve child hits"
    );
    assert!(surface
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == UiNodeId::new(5)));
    assert!(!surface
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == UiNodeId::new(3)));
}

#[test]
fn legacy_visible_false_is_normalized_into_hidden_visibility_for_surface_outputs() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 80.0))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    let mut hidden_state = pointer_state();
    hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/legacy_hidden"))
                .with_frame(UiFrame::new(10.0, 10.0, 60.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(hidden_state),
        )
        .unwrap();

    surface.rebuild();

    let arranged = surface.arranged_tree.get(UiNodeId::new(2)).unwrap();
    assert_eq!(arranged.visibility, UiVisibility::Hidden);
    assert!(surface
        .render_extract
        .list
        .commands
        .iter()
        .all(|command| command.node_id != UiNodeId::new(2)));
    assert!(surface
        .hit_test
        .grid
        .entries
        .iter()
        .all(|entry| entry.node_id != UiNodeId::new(2)));
    assert_eq!(surface.hit_test(UiPoint::new(20.0, 15.0)).top_hit, None);
}
