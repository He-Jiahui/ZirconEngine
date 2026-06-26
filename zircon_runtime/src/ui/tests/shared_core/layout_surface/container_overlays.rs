use super::*;

#[test]
fn overlay_deserializes_and_measures_to_the_largest_child_extent() {
    let container: UiContainerKind = serde_json::from_str(r#""Overlay""#).unwrap();
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/background"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(180.0),
                    height: fixed_constraint(100.0),
                })
                .with_anchor(Anchor::new(0.0, 0.0))
                .with_pivot(Pivot::new(0.0, 0.0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/foreground"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(30.0),
                })
                .with_anchor(Anchor::new(1.0, 1.0))
                .with_pivot(Pivot::new(1.0, 1.0)),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(200.0, 120.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(180.0, 100.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 180.0, 100.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(120.0, 90.0, 80.0, 30.0)
    );
}

#[test]
fn space_ignores_child_content_and_behaves_as_layout_spacer() {
    let container: UiContainerKind = serde_json::from_str(r#""Space""#).unwrap();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(
                serde_json::from_str(r#"{"gap":4.0}"#).unwrap(),
            ))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/space"))
                .with_container(container)
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(24.0),
                    height: stretch_constraint(0.0, 0.0, 100, 1.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(UiNodeId::new(20), UiNodePath::new("root/space/ignored"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(90.0),
                    height: fixed_constraint(50.0),
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/content")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(60.0),
                    height: fixed_constraint(30.0),
                },
            ),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(120.0, 40.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(24.0, 0.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 24.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(20))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(28.0, 0.0, 60.0, 30.0)
    );
}
