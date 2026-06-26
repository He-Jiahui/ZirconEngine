use super::*;

#[test]
fn explicit_collapsed_visibility_preserves_layout_collapse_with_legacy_visible_false() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(100.0),
                height: fixed_constraint(100.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/top")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    let mut legacy_hidden_state = pointer_state();
    legacy_hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/collapsed"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                })
                .with_visibility(UiVisibility::Collapsed)
                .with_state_flags(legacy_hidden_state),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/bottom")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(100.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    let collapsed = surface.arranged_tree.get(UiNodeId::new(3)).unwrap();
    assert_eq!(collapsed.visibility, UiVisibility::Collapsed);
    assert_eq!(collapsed.frame, UiFrame::default());
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .expect("bottom node should be arranged")
            .layout_cache
            .frame
            .y,
        20.0
    );
}

#[test]
fn taffy_vertical_layout_skips_collapsed_child_without_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.taffy.collapsed"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_constraints(BoxConstraints {
                width: taffy_fixed_constraint(100.0),
                height: taffy_fixed_constraint(100.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/top")).with_constraints(
                BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/collapsed"))
                .with_visibility(UiVisibility::Collapsed)
                .with_constraints(BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(3),
            UiTreeNode::new(UiNodeId::new(30), UiNodePath::new("root/collapsed/child"))
                .with_constraints(BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/bottom")).with_constraints(
                BoxConstraints {
                    width: taffy_fixed_constraint(100.0),
                    height: taffy_fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 100.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(30))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 20.0, 100.0, 20.0)
    );

    let root_selection = surface
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("root should record a layout engine selection");
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(surface.layout_engine_report.fallback_count, 0);
    assert_eq!(surface.layout_engine_report.taffy_tree_node_count, 3);
}
