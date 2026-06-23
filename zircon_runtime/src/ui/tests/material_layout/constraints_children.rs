use super::*;

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
