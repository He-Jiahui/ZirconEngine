use super::*;

#[test]
fn horizontal_box_deserializes_and_arranges_children_with_gap_and_cross_axis_stretch() {
    let container: UiContainerKind =
        serde_json::from_str(r#"{"HorizontalBox":{"gap":10.0}}"#).unwrap();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/label")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(50.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/fill")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: stretch_constraint(10.0, 10.0, 100, 1.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(90.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 50.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(60.0, 0.0, 30.0, 80.0)
    );
}

#[test]
fn vertical_box_resolves_main_axis_stretch_and_cross_axis_fill() {
    let container: UiContainerKind =
        serde_json::from_str(r#"{"VerticalBox":{"gap":8.0}}"#).unwrap();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/header")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: stretch_constraint(30.0, 40.0, 100, 1.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/content")).with_constraints(
                BoxConstraints {
                    width: stretch_constraint(10.0, 10.0, 100, 1.0),
                    height: stretch_constraint(30.0, 40.0, 100, 3.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(120.0, 200.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(30.0, 88.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 30.0, 68.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 76.0, 120.0, 124.0)
    );
}

#[test]
fn wrap_box_arranges_children_into_rows_by_available_width() {
    let container = UiContainerKind::WrapBox(UiWrapBoxConfig {
        horizontal_gap: 8.0,
        vertical_gap: 6.0,
        item_min_width: 50.0,
    });
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    for (id, width, height) in [(2, 50.0, 20.0), (3, 50.0, 30.0), (4, 50.0, 24.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("root/{id}")))
                    .with_constraints(BoxConstraints {
                        width: fixed_constraint(width),
                        height: fixed_constraint(height),
                    }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(120.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 50.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(58.0, 0.0, 50.0, 30.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 36.0, 50.0, 24.0)
    );
}

#[test]
fn wrap_box_keeps_children_on_one_row_when_width_allows() {
    let container: UiContainerKind = serde_json::from_str(
        r#"{"WrapBox":{"horizontal_gap":8.0,"vertical_gap":6.0,"item_min_width":50.0}}"#,
    )
    .unwrap();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    for id in 2..=4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("root/{id}")))
                    .with_constraints(BoxConstraints {
                        width: fixed_constraint(40.0),
                        height: fixed_constraint(20.0),
                    }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(180.0, 80.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(4))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(116.0, 0.0, 50.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .content_size,
        UiSize::new(166.0, 20.0)
    );
}

#[test]
fn wrap_box_content_size_tracks_wrapped_rows() {
    let container = UiContainerKind::WrapBox(UiWrapBoxConfig {
        horizontal_gap: 8.0,
        vertical_gap: 6.0,
        item_min_width: 50.0,
    });
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: AxisConstraint {
                    min: 0.0,
                    max: 120.0,
                    preferred: 0.0,
                    priority: 100,
                    weight: 1.0,
                    stretch_mode: StretchMode::Fixed,
                },
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    for (id, width, height) in [(2, 50.0, 20.0), (3, 50.0, 30.0), (4, 50.0, 24.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("root/{id}")))
                    .with_constraints(BoxConstraints {
                        width: fixed_constraint(width),
                        height: fixed_constraint(height),
                    }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(120.0, 100.0)).unwrap();

    let root = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert_eq!(root.layout_cache.content_size, UiSize::new(108.0, 60.0));
    assert_eq!(
        root.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 120.0, 100.0)
    );
}

#[test]
fn wrap_box_measurement_uses_width_bounds_before_root_arrange() {
    let container = UiContainerKind::WrapBox(UiWrapBoxConfig {
        horizontal_gap: 8.0,
        vertical_gap: 6.0,
        item_min_width: 50.0,
    });
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(container)
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: AxisConstraint {
                    min: 0.0,
                    max: 108.0,
                    preferred: 0.0,
                    priority: 100,
                    weight: 1.0,
                    stretch_mode: StretchMode::Fixed,
                },
                height: AxisConstraint {
                    min: 0.0,
                    max: 100.0,
                    preferred: 0.0,
                    priority: 100,
                    weight: 1.0,
                    stretch_mode: StretchMode::Fixed,
                },
            }),
    );
    for (id, width, height) in [(2, 50.0, 20.0), (3, 50.0, 30.0), (4, 50.0, 24.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("root/{id}")))
                    .with_constraints(BoxConstraints {
                        width: fixed_constraint(width),
                        height: fixed_constraint(height),
                    }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(240.0, 160.0)).unwrap();

    let root = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert_eq!(
        root.layout_cache.desired_size,
        DesiredSize::new(108.0, 60.0)
    );
    assert_eq!(root.layout_cache.content_size, UiSize::new(166.0, 30.0));
    assert_eq!(
        root.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 240.0, 160.0)
    );
}
