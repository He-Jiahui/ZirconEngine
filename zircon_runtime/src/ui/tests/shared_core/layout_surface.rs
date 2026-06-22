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

#[test]
fn render_extract_carries_visual_contract_fields_for_visible_nodes() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/launch"))
                .with_frame(UiFrame::new(12.0, 8.0, 96.0, 32.0))
                .with_z_index(7)
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
                    component: "IconButton".to_string(),
                    control_id: Some("LaunchButton".to_string()),
                    classes: vec!["primary".to_string()],
                    attributes: toml::from_str(
                        r##"
text = "Launch"
icon = "rocket-outline"
opacity = 0.75
font = "res://fonts/default.font.toml"
font_family = "Fira Mono"
font_size = 18.0
line_height = 24.0
text_align = "center"
wrap = "word"
text_render_mode = "sdf"

[background]
color = "#112233"

[foreground]
color = "#ddeeff"

[border]
color = "#334455"
width = 2.0
radius = 6.0
"##,
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
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 80.0, 28.0));

    surface.rebuild();

    let root_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(1))
        .unwrap();
    assert_eq!(root_command.kind, UiRenderCommandKind::Group);
    assert_eq!(root_command.style, UiResolvedStyle::default());
    assert_eq!(root_command.text.as_deref(), None);
    assert_eq!(root_command.image, None);
    assert_eq!(root_command.opacity, 1.0);

    let launch_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(launch_command.kind, UiRenderCommandKind::Quad);
    assert_eq!(
        launch_command.clip_frame,
        Some(UiFrame::new(0.0, 0.0, 80.0, 28.0))
    );
    assert_eq!(launch_command.z_index, 7);
    assert_eq!(launch_command.text.as_deref(), Some("Launch"));
    assert_eq!(
        launch_command.image,
        Some(UiVisualAssetRef::Icon("rocket-outline".to_string()))
    );
    assert_eq!(launch_command.opacity, 0.75);
    assert_eq!(
        launch_command.style,
        UiResolvedStyle {
            background_color: Some("#112233".to_string()),
            foreground_color: Some("#ddeeff".to_string()),
            border_color: Some("#334455".to_string()),
            border_width: 2.0,
            corner_radius: 6.0,
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Fira Mono".to_string()),
            font_size: 18.0,
            line_height: 24.0,
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::Word,
            text_direction: Default::default(),
            text_overflow: Default::default(),
            rich_text: false,
            text_render_mode: UiTextRenderMode::Sdf,
            ..UiResolvedStyle::default()
        }
    );
}

#[test]
fn render_extract_accepts_flat_style_color_aliases() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 90.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/workbench-button"))
                .with_frame(UiFrame::new(12.0, 12.0, 112.0, 32.0))
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("WorkbenchPrimaryButton".to_string()),
                    classes: vec![
                        "workbench-control-button".to_string(),
                        "workbench-primary-button".to_string(),
                    ],
                    attributes: toml::from_str(
                        r##"
label = "Primary"
background_color = "#12383d"
foreground_color = "#e8edf2"
border_color = "#35c7d0"
border_width = 1.0
radius = 8.0
"##,
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

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(command.kind, UiRenderCommandKind::Quad);
    assert_eq!(command.text.as_deref(), Some("Primary"));
    assert_eq!(
        command.style,
        UiResolvedStyle {
            background_color: Some("#12383d".to_string()),
            foreground_color: Some("#e8edf2".to_string()),
            border_color: Some("#35c7d0".to_string()),
            border_width: 1.0,
            corner_radius: 8.0,
            painter_family: UiPainterFamily::Button,
            painter_state: UiPainterResolvedState::Normal,
            ..UiResolvedStyle::default()
        }
    );
}

#[test]
fn render_extract_uses_label_when_schema_text_default_is_placeholder() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/locate"))
                .with_frame(UiFrame::new(8.0, 8.0, 124.0, 32.0))
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
                    control_id: Some("LocateSelectedAsset".to_string()),
                    classes: Vec::new(),
                    attributes: toml::from_str(
                        r#"
text = "Button"
label = "Locate In Assets"
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

    surface.rebuild();

    let locate_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(locate_command.text.as_deref(), Some("Locate In Assets"));
}

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
