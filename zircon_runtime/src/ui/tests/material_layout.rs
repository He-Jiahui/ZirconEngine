use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, StretchMode, UiSize},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn material_button_measures_text_plus_layout_padding() {
    let desired = measure_material_leaf(
        "Button",
        r#"
text = "Apply"
font_size = 10
line_height = 12
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_width = 40
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(73.0, 40.0));
}

#[test]
fn material_button_with_icon_adds_icon_size_and_spacing() {
    let desired = measure_material_leaf(
        "Button",
        r#"
text = "Apply"
icon = "check"
font_size = 10
line_height = 12
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_width = 40
layout_min_height = 40
layout_icon_size = 18
layout_spacing = 8
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(99.0, 40.0));
}

#[test]
fn material_icon_button_without_text_uses_icon_and_minimum_outer_size() {
    let desired = measure_material_leaf(
        "IconButton",
        r#"
layout_min_width = 40
layout_min_height = 40
layout_icon_size = 18
layout_padding_left = 0
layout_padding_right = 0
layout_padding_top = 0
layout_padding_bottom = 0
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(40.0, 40.0));
}

#[test]
fn material_button_respects_authored_fixed_constraints() {
    let desired = measure_material_leaf(
        "Button",
        r#"
text = "Apply"
font_size = 10
line_height = 12
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_width = 40
layout_min_height = 40
"#,
        BoxConstraints {
            width: fixed_constraint(120.0),
            height: fixed_constraint(44.0),
        },
    );

    assert_eq!(desired, DesiredSize::new(120.0, 44.0));
}

#[test]
fn material_button_with_child_content_receives_padding_and_minimum_height() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout.children"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(intrinsic_constraints())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    attributes: toml::from_str(
                        r#"
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_height = 40
"#,
                    )
                    .unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/button/label"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(intrinsic_constraints())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    attributes: toml::from_str(
                        r#"
text = "Apply"
font_size = 10
line_height = 12
"#,
                    )
                    .unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();

    let button = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        button.layout_cache.desired_size,
        DesiredSize::new(73.0, 40.0)
    );
}

#[test]
fn material_list_field_and_switch_controls_keep_min_height() {
    let list_row = measure_material_leaf(
        "ListRow",
        r#"
text = "Go"
font_size = 10
line_height = 12
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );
    let text_field = measure_material_leaf(
        "TextField",
        r#"
text = ""
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let switch = measure_material_leaf(
        "Switch",
        r#"
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(list_row.height, 40.0);
    assert_eq!(text_field.height, 56.0);
    assert_eq!(switch.height, 40.0);
}

fn measure_material_leaf(
    component: &str,
    attributes: &str,
    constraints: BoxConstraints,
) -> DesiredSize {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(constraints)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .desired_size
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

fn intrinsic_constraints() -> BoxConstraints {
    BoxConstraints {
        width: intrinsic_axis_constraint(),
        height: intrinsic_axis_constraint(),
    }
}

fn intrinsic_axis_constraint() -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
