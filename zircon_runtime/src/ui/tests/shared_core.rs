use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    layout::{compute_virtual_list_window, solve_axis_constraints},
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
    tree::{
        UiHitTestIndex, UiRuntimeTreeAccessExt, UiRuntimeTreeFocusExt, UiRuntimeTreeInteractionExt,
        UiRuntimeTreeLayoutExt, UiRuntimeTreeScrollExt,
    },
};
use zircon_runtime_interface::ui::{
    component::{UiValue, UiValueKind},
    dispatch::{UiNavigationDispatchEffect, UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiReflectedPropertySource, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, Pivot, Position,
        StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint, UiScrollState,
        UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
        UiVirtualListWindow,
    },
    surface::{
        UiFocusState, UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
        UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
        UiVisualAssetRef,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

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
        }
    );
}

#[test]
fn render_extract_uses_label_when_schema_text_default_is_empty() {
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
text = ""
label = "Locate In Assets"
"#,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
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
fn pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers() {
    use std::sync::{Arc, Mutex};

    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 120.0))
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
    surface.rebuild();

    let seen_buttons = Arc::new(Mutex::new(Vec::new()));
    let seen_buttons_for_handler = Arc::clone(&seen_buttons);
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(1), UiPointerEventKind::Down, move |context| {
        seen_buttons_for_handler
            .lock()
            .unwrap()
            .push(context.route.button);
        UiPointerDispatchEffect::capture()
    });

    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Secondary),
        )
        .unwrap();

    assert_eq!(result.route.button, Some(UiPointerButton::Secondary));
    assert_eq!(
        seen_buttons.lock().unwrap().as_slice(),
        &[Some(UiPointerButton::Secondary)]
    );
}

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
fn focus_navigation_and_scroll_candidates_use_effective_visibility() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/visible_focus"))
                .with_frame(UiFrame::new(0.0, 0.0, 40.0, 20.0))
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(3),
                UiNodePath::new("root/hit_test_invisible_focus"),
            )
            .with_frame(UiFrame::new(50.0, 0.0, 40.0, 20.0))
            .with_visibility(UiVisibility::HitTestInvisible)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    let mut legacy_hidden_state = pointer_state();
    legacy_hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(4),
                UiNodePath::new("root/legacy_hidden_focus"),
            )
            .with_frame(UiFrame::new(100.0, 0.0, 40.0, 20.0))
            .with_state_flags(legacy_hidden_state),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(5), UiNodePath::new("root/scroll"))
                .with_frame(UiFrame::new(0.0, 40.0, 100.0, 50.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState::default())
                .with_visibility(UiVisibility::SelfHitTestInvisible)
                .with_state_flags({
                    let mut state = pointer_state();
                    state.focusable = false;
                    state
                }),
        )
        .unwrap();

    assert_eq!(surface.focus_node(UiNodeId::new(3)), Ok(()));
    assert!(surface.focus_node(UiNodeId::new(4)).is_err());
    let focus_order = surface.tree.focusable_nodes_in_navigation_order().unwrap();
    assert!(focus_order.contains(&UiNodeId::new(2)));
    assert!(focus_order.contains(&UiNodeId::new(3)));
    assert!(!focus_order.contains(&UiNodeId::new(4)));
    assert_eq!(
        surface
            .tree
            .first_scrollable_in_candidates(&[UiNodeId::new(5)])
            .unwrap(),
        Some(UiNodeId::new(5))
    );
}

#[test]
fn pointer_capture_routes_move_and_up_to_the_captured_node() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
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
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/left"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 120.0))
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
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/right"))
                .with_frame(UiFrame::new(120.0, 0.0, 100.0, 120.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.rebuild();

    let down = surface
        .route_pointer_event(UiPointerEventKind::Down, UiPoint::new(130.0, 20.0))
        .unwrap();
    assert_eq!(down.target, Some(UiNodeId::new(3)));
    assert_eq!(down.bubbled, vec![UiNodeId::new(3), UiNodeId::new(1)]);
    assert_eq!(down.hit_path.target, Some(UiNodeId::new(3)));
    assert_eq!(
        down.hit_path.root_to_leaf,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(
        down.hit_path.bubble_route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );

    surface.capture_pointer(UiNodeId::new(3)).unwrap();
    let moved = surface
        .route_pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0))
        .unwrap();
    assert_eq!(moved.target, Some(UiNodeId::new(3)));
    assert_eq!(moved.stacked, vec![UiNodeId::new(2), UiNodeId::new(1)]);
    assert_eq!(moved.entered, vec![UiNodeId::new(2)]);
    assert_eq!(moved.left, vec![UiNodeId::new(3)]);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));

    let up = surface
        .route_pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
        .unwrap();
    assert_eq!(up.target, Some(UiNodeId::new(3)));
    assert_eq!(surface.focus.captured, None);
}

#[test]
fn navigation_routes_from_focus_and_falls_back_to_roots() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface.focus = UiFocusState {
        focused: Some(UiNodeId::new(2)),
        captured: None,
        pressed: None,
        hovered: Vec::new(),
    };

    let focused = surface
        .route_navigation_event(UiNavigationEventKind::Next)
        .unwrap();
    assert_eq!(focused.target, Some(UiNodeId::new(2)));
    assert_eq!(focused.bubbled, vec![UiNodeId::new(2), UiNodeId::new(1)]);
    assert!(!focused.fallback_to_root);

    surface.focus.focused = None;
    let fallback = surface
        .route_navigation_event(UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(fallback.target, None);
    assert!(fallback.fallback_to_root);
    assert_eq!(fallback.root_targets, vec![UiNodeId::new(1)]);
}

#[test]
fn navigation_dispatcher_bubbles_from_focus_and_can_move_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/left")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/right")).with_state_flags(
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                },
            ),
        )
        .unwrap();
    surface.focus = UiFocusState {
        focused: Some(UiNodeId::new(2)),
        captured: None,
        pressed: None,
        hovered: Vec::new(),
    };

    let mut dispatcher = UiNavigationDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiNavigationEventKind::Next, |_context| {
        UiNavigationDispatchEffect::Unhandled
    });
    dispatcher.register(UiNodeId::new(1), UiNavigationEventKind::Next, |_context| {
        UiNavigationDispatchEffect::focus(UiNodeId::new(3))
    });

    let result = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();

    assert_eq!(result.route.target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.route.bubbled,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.handled_by, Some(UiNodeId::new(1)));
    assert_eq!(result.focus_changed_to, Some(UiNodeId::new(3)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
}

#[test]
fn navigation_dispatcher_falls_back_to_root_handlers_without_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_state_flags(UiStateFlags {
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

    let mut dispatcher = UiNavigationDispatcher::default();
    dispatcher.register(
        UiNodeId::new(1),
        UiNavigationEventKind::Activate,
        |_context| UiNavigationDispatchEffect::handled(),
    );

    let result = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Activate)
        .unwrap();

    assert!(result.route.fallback_to_root);
    assert_eq!(result.route.root_targets, vec![UiNodeId::new(1)]);
    assert_eq!(result.handled_by, Some(UiNodeId::new(1)));
    assert_eq!(surface.focus.focused, None);
}

#[test]
fn navigation_dispatcher_falls_back_to_shared_tab_order_when_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 80.0))
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
    for (node_id, x) in [(2, 0.0), (3, 80.0), (4, 160.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(UiFrame::new(x, 0.0, 60.0, 40.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }

    let dispatcher = UiNavigationDispatcher::default();

    let first = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();
    assert!(first.route.fallback_to_root);
    assert_eq!(first.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(first.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    surface.focus.focused = Some(UiNodeId::new(4));
    let wrapped = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
        .unwrap();
    assert_eq!(wrapped.route.target, Some(UiNodeId::new(4)));
    assert_eq!(wrapped.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(wrapped.handled_by, Some(UiNodeId::new(4)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    let previous = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Previous)
        .unwrap();
    assert_eq!(previous.route.target, Some(UiNodeId::new(2)));
    assert_eq!(previous.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(previous.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_falls_back_to_nearest_directional_focus_target() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 220.0))
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
    for (node_id, frame) in [
        (2, UiFrame::new(10.0, 10.0, 40.0, 40.0)),
        (3, UiFrame::new(90.0, 20.0, 40.0, 40.0)),
        (4, UiFrame::new(20.0, 100.0, 40.0, 40.0)),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(frame)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }
    surface.focus.focused = Some(UiNodeId::new(2));

    let dispatcher = UiNavigationDispatcher::default();

    let right = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Right)
        .unwrap();
    assert_eq!(right.route.target, Some(UiNodeId::new(2)));
    assert_eq!(right.focus_changed_to, Some(UiNodeId::new(3)));
    assert_eq!(right.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));

    surface.focus.focused = Some(UiNodeId::new(2));
    let down = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Down)
        .unwrap();
    assert_eq!(down.route.target, Some(UiNodeId::new(2)));
    assert_eq!(down.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(down.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_starts_directional_fallback_from_shared_endcaps_without_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 80.0))
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
    for (node_id, x) in [(2, 0.0), (3, 80.0), (4, 160.0)] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(node_id),
                    UiNodePath::new(format!("root/item_{node_id}")),
                )
                .with_frame(UiFrame::new(x, 0.0, 60.0, 40.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }

    let dispatcher = UiNavigationDispatcher::default();

    let right = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Right)
        .unwrap();
    assert!(right.route.fallback_to_root);
    assert_eq!(right.focus_changed_to, Some(UiNodeId::new(2)));
    assert_eq!(right.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    surface.clear_focus();
    let left = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Left)
        .unwrap();
    assert!(left.route.fallback_to_root);
    assert_eq!(left.focus_changed_to, Some(UiNodeId::new(4)));
    assert_eq!(left.handled_by, Some(UiNodeId::new(4)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
}

#[test]
fn navigation_dispatcher_keeps_focus_when_activate_or_cancel_is_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 40.0))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/item"))
                .with_frame(UiFrame::new(0.0, 0.0, 60.0, 40.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let dispatcher = UiNavigationDispatcher::default();

    let activate = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(activate.route.target, Some(UiNodeId::new(2)));
    assert_eq!(activate.focus_changed_to, None);
    assert_eq!(activate.handled_by, None);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));

    let cancel = surface
        .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Cancel)
        .unwrap();
    assert_eq!(cancel.route.target, Some(UiNodeId::new(2)));
    assert_eq!(cancel.focus_changed_to, None);
    assert_eq!(cancel.handled_by, None);
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
}

#[test]
fn virtual_list_window_tracks_visible_range_with_overscan() {
    let window = compute_virtual_list_window(120.0, 150.0, 50.0, 20, 1);
    assert_eq!(
        window,
        UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 7,
        }
    );

    let clamped = compute_virtual_list_window(0.0, 40.0, 50.0, 2, 3);
    assert_eq!(
        clamped,
        UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 2,
        }
    );
}

#[test]
fn scrollable_box_tracks_content_metrics_virtual_window_and_local_scroll_invalidation() {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 1,
                    }),
                }))
                .with_scroll_state(UiScrollState::default()),
        )
        .unwrap();

    for item in 0..5 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
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
    }

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(scroll.layout_cache.content_size, UiSize::new(200.0, 200.0));
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(
        scroll.scroll_state,
        Some(UiScrollState {
            offset: 0.0,
            viewport_extent: 90.0,
            content_extent: 200.0,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(14))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );

    surface
        .tree
        .set_scroll_offset(UiNodeId::new(2), 80.0)
        .unwrap();

    let root = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(!root.dirty.layout);

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert!(scroll.dirty.layout);
    assert!(scroll.dirty.hit_test);
    assert!(scroll.dirty.render);
    assert!(scroll.dirty.visible_range);
    assert_eq!(scroll.scroll_state.unwrap().offset, 80.0);

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 5,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(11))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, -40.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(12))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
}

#[test]
fn pointer_dispatcher_applies_block_passthrough_and_capture_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
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
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/bottom"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(0)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/top"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(10)
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
    surface.rebuild();

    let mut block_dispatcher = UiPointerDispatcher::default();
    block_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::blocked()
    });
    block_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let blocked = surface
        .dispatch_pointer_event(
            &block_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(blocked.blocked_by, Some(UiNodeId::new(3)));
    assert_eq!(blocked.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        blocked
            .invocations
            .iter()
            .map(|invocation| (invocation.node_id, invocation.effect))
            .collect::<Vec<_>>(),
        vec![
            (UiNodeId::new(3), UiPointerDispatchEffect::Blocked),
            (UiNodeId::new(2), UiPointerDispatchEffect::Handled),
        ]
    );

    let mut passthrough_dispatcher = UiPointerDispatcher::default();
    passthrough_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::passthrough()
    });
    passthrough_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });
    let passthrough = surface
        .dispatch_pointer_event(
            &passthrough_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(passthrough.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(passthrough.passthrough, vec![UiNodeId::new(3)]);

    let mut capture_dispatcher = UiPointerDispatcher::default();
    capture_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    let captured = surface
        .dispatch_pointer_event(
            &capture_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(captured.captured_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
}

#[test]
fn captured_pointer_dispatch_keeps_move_and_up_targeting_the_captured_node_outside_hit_bounds() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 120.0))
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/viewport"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.rebuild();

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Up, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.captured_by, Some(UiNodeId::new(2)));

    let moved = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(160.0, 160.0)),
        )
        .unwrap();
    assert_eq!(moved.route.target, Some(UiNodeId::new(2)));
    assert_eq!(moved.handled_by, Some(UiNodeId::new(2)));

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(160.0, 160.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(up.route.target, Some(UiNodeId::new(2)));
    assert_eq!(up.handled_by, Some(UiNodeId::new(2)));
}

#[test]
fn scroll_pointer_event_scrolls_the_nearest_scrollable_box_when_unhandled() {
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
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
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
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
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
    assert!(surface.tree.node(UiNodeId::new(2)).unwrap().dirty.layout);
}

#[test]
fn surface_property_mutation_marks_dirty_only_when_values_change() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_state_flags(pointer_state())
            .with_input_policy(UiInputPolicy::Receive),
    );

    let unchanged = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "enabled",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(unchanged.status, UiPropertyMutationStatus::Unchanged);
    assert!(!surface.tree.node(UiNodeId::new(1)).unwrap().dirty.any());

    let changed = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(changed.status, UiPropertyMutationStatus::Accepted);
    assert!(changed.invalidation.dirty.input);
    assert!(changed.invalidation.dirty.hit_test);
    let node = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(!node.state_flags.enabled);
    assert!(node.state_flags.dirty);
    assert!(node.dirty.input);

    let rejected = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "enabled",
            UiValue::String("false".to_string()),
        ))
        .unwrap();
    assert_eq!(rejected.status, UiPropertyMutationStatus::Rejected);
    assert!(rejected.message.unwrap().contains("boolean"));
}

#[test]
fn surface_property_mutation_updates_authored_metadata_and_reflector_snapshot() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/title"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 20.0))
                .with_state_flags(pointer_state())
                .with_input_policy(UiInputPolicy::Receive)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    control_id: Some("TitleLabel".to_string()),
                    attributes: toml::from_str("text = 'Inspect'").unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let report = surface
        .mutate_property(
            UiPropertyMutationRequest::new(
                UiNodeId::new(2),
                "text",
                UiValue::String("Reflect".to_string()),
            )
            .with_source(UiReflectedPropertySource::Authored),
        )
        .unwrap();
    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    assert!(report.invalidation.dirty.layout);
    assert!(report.invalidation.dirty.text);

    let snapshot = surface.reflector_snapshot(Some(
        zircon_runtime_interface::ui::surface::UiHitTestQuery::new(UiPoint::new(10.0, 10.0)),
    ));
    let reflected = snapshot.node(UiNodeId::new(2)).expect("reflected title");
    let text = reflected.properties.get("text").expect("text property");
    assert_eq!(reflected.display_name, "TitleLabel");
    assert_eq!(text.source, UiReflectedPropertySource::Authored);
    assert_eq!(text.value_kind, UiValueKind::String);
    assert_eq!(text.resolved_value, UiValue::String("Reflect".to_string()));
    assert!(text.invalidation.dirty.layout);
    assert_eq!(
        snapshot.hit_context.unwrap().hit_target,
        Some(UiNodeId::new(2))
    );
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
