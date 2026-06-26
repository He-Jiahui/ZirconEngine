use super::*;

#[test]
fn shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes() {
    let resolved = solve_axis_constraints(
        900.0,
        &[
            stretch_constraint(200.0, 300.0, 100, 3.0),
            stretch_constraint(180.0, 220.0, 50, 1.0),
            stretch_constraint(180.0, 220.0, 50, 1.0),
        ],
    );

    assert_eq!(resolved.len(), 3);
    assert!(resolved[0].resolved > 300.0);
    assert_eq!(resolved[1].resolved, 220.0);
    assert_eq!(resolved[2].resolved, 220.0);
}

#[test]
fn layout_invalidation_bubbles_until_parent_directed_boundary() {
    let mut tree = UiTree::new(UiTreeId::new("runtime.ui"));
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/container"))
            .with_layout_boundary(LayoutBoundary::ParentDirected),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(2),
        UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/container/label"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    )
    .unwrap();

    tree.mark_layout_dirty(UiNodeId::new(3)).unwrap();

    assert!(tree.node(UiNodeId::new(3)).unwrap().dirty.layout);
    assert!(tree.node(UiNodeId::new(2)).unwrap().dirty.layout);
    assert!(!tree.node(UiNodeId::new(1)).unwrap().dirty.layout);
}

#[test]
fn layout_pass_measures_content_driven_roots_and_arranges_anchored_children() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/fill"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(64.0, 64.0, 100, 1.0),
                    height: stretch_constraint(32.0, 32.0, 100, 1.0),
                })
                .with_anchor(Anchor::new(0.0, 0.0))
                .with_pivot(Pivot::new(0.0, 0.0))
                .with_position(Position::new(0.0, 0.0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/badge"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(120.0),
                    height: fixed_constraint(40.0),
                })
                .with_anchor(Anchor::new(0.5, 0.5))
                .with_pivot(Pivot::new(0.5, 0.5))
                .with_position(Position::new(10.0, -5.0)),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(400.0, 300.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(120.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 400.0, 300.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(64.0, 32.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 400.0, 300.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(150.0, 125.0, 120.0, 40.0)
    );
}

#[test]
fn layout_pass_measures_label_leaf_from_text_intrinsic_size() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/title"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    control_id: Some("TitleLabel".to_string()),
                    classes: Vec::new(),
                    attributes: toml::from_str(
                        r#"
text = "Inspect"
font_size = 10.0
line_height = 12.0
"#,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    let label = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        label.layout_cache.desired_size,
        DesiredSize::new(35.0, 12.0)
    );
    assert_eq!(label.layout_cache.frame.height, 12.0);
    assert!(label.layout_cache.frame.width >= 35.0);
}

#[test]
fn layout_pass_measures_button_leaf_as_text_plus_padding() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/apply"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("ApplyDraft".to_string()),
                    classes: Vec::new(),
                    attributes: toml::from_str(
                        r#"
text = "Apply"
font_size = 10.0
line_height = 12.0
"#,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    let button = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        button.layout_cache.desired_size,
        DesiredSize::new(43.0, 20.0)
    );
    assert_eq!(button.layout_cache.frame.height, 20.0);
    assert!(button.layout_cache.frame.width >= 43.0);
}

#[test]
fn container_deserializes_and_arranges_anchored_children_like_shared_free_layout() {
    let container: UiContainerKind = serde_json::from_str(r#""Container""#).unwrap();
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/fill"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(64.0, 64.0, 100, 1.0),
                    height: stretch_constraint(32.0, 32.0, 100, 1.0),
                })
                .with_anchor(Anchor::new(0.0, 0.0))
                .with_pivot(Pivot::new(0.0, 0.0))
                .with_position(Position::new(0.0, 0.0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/badge"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(120.0),
                    height: fixed_constraint(40.0),
                })
                .with_anchor(Anchor::new(0.5, 0.5))
                .with_pivot(Pivot::new(0.5, 0.5))
                .with_position(Position::new(10.0, -5.0)),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(400.0, 300.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(120.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 400.0, 300.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(150.0, 125.0, 120.0, 40.0)
    );
}
