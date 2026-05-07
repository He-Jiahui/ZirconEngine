use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, StretchMode, UiSize},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
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
fn material_button_long_text_expands_beyond_default_frame_width() {
    let desired = measure_material_leaf(
        "Button",
        r#"
text = "Launch Comprehensive Runtime Diagnostics"
font_size = 12
line_height = 14
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_width = 40
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(288.0, 40.0));
    assert!(
        desired.width > 40.0,
        "long Material button text must expand the desired frame instead of clipping to the default min width"
    );
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
fn material_icon_only_button_keeps_square_material_size() {
    let desired = measure_material_leaf(
        "IconButton",
        r#"
icon = "add-outline"
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
fn material_icon_button_ignores_accessibility_label_for_intrinsic_text() {
    let desired = measure_material_leaf(
        "IconButton",
        r#"
label = "Focus Console"
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
fn material_icon_button_without_visual_icon_keeps_label_accessibility_only() {
    let desired = measure_material_leaf(
        "IconButton",
        r#"
label = "Reveal"
font_size = 12
line_height = 14
layout_min_width = 24
layout_min_height = 40
layout_icon_size = 0
layout_padding_left = 0
layout_padding_right = 0
layout_padding_top = 0
layout_padding_bottom = 0
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(24.0, 40.0));
}

#[test]
fn material_menu_item_uses_list_row_height_and_horizontal_padding() {
    let desired = measure_material_leaf(
        "MenuItem",
        r#"
text = "Duplicate"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(86.0, 40.0));
}

#[test]
fn material_tab_uses_control_height_and_text_width_plus_padding() {
    let desired = measure_material_leaf(
        "Tab",
        r#"
text = "Inspector"
font_size = 12
line_height = 14
layout_padding_left = 24
layout_padding_right = 24
layout_padding_top = 10
layout_padding_bottom = 10
layout_min_width = 40
layout_min_height = 36
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(102.0, 36.0));
}

#[test]
fn plain_non_material_label_remains_text_only() {
    let desired = measure_material_leaf(
        "Label",
        r#"
text = "Plain label"
font_size = 12
line_height = 14
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(66.0, 14.0));
}

#[test]
fn material_label_with_layout_attributes_receives_conservative_padding() {
    let desired = measure_material_leaf(
        "Label",
        r#"
text = "Status"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(68.0, 40.0));
}

#[test]
fn material_table_row_uses_list_row_height_and_text_width_plus_padding() {
    let desired = measure_material_leaf(
        "TableRow",
        r#"
text = "Row 1024"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(desired, DesiredSize::new(80.0, 40.0));
}

#[test]
fn material_fields_measure_visible_value_placeholder_and_options_text() {
    let input_value = measure_material_leaf(
        "InputField",
        r#"
value = "runtime/material/search/query"
placeholder = "Search"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let text_placeholder = measure_material_leaf(
        "TextField",
        r#"
value = ""
placeholder = "Describe runtime material layout"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let combo_selected = measure_material_leaf(
        "ComboBox",
        r#"
value = "Slate Material"
options = ["Native", "Slate Material", "Compact"]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let combo_default_option = measure_material_leaf(
        "ComboBox",
        r#"
options = ["First Available Option", "Second"]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );

    assert_eq!(input_value, DesiredSize::new(206.0, 56.0));
    assert_eq!(text_placeholder, DesiredSize::new(224.0, 56.0));
    assert_eq!(combo_selected, DesiredSize::new(116.0, 56.0));
    assert_eq!(combo_default_option, DesiredSize::new(164.0, 56.0));
}

#[test]
fn material_numeric_fields_measure_numeric_value_text() {
    let number = measure_material_leaf(
        "NumberField",
        r#"
value = 12345
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let range = measure_material_leaf(
        "RangeField",
        r#"
value = 0.75
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );

    assert_eq!(number, DesiredSize::new(62.0, 56.0));
    assert_eq!(range, DesiredSize::new(56.0, 56.0));
}

#[test]
fn material_options_measure_scalar_and_object_visible_text() {
    let numeric_option = measure_material_leaf(
        "ComboBox",
        r#"
options = [42, 7]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let object_label_fallback = measure_material_leaf(
        "ComboBox",
        r#"
options = [{ text = "", label = "Fallback Label" }]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let object_numeric_value = measure_material_leaf(
        "ComboBox",
        r#"
options = [{ text = "", label = "", value = 42 }]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );
    let bool_option = measure_material_leaf(
        "ComboBox",
        r#"
options = [true, false]
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 56
"#,
        intrinsic_constraints(),
    );

    assert_eq!(numeric_option, DesiredSize::new(44.0, 56.0));
    assert_eq!(object_label_fallback, DesiredSize::new(116.0, 56.0));
    assert_eq!(object_numeric_value, DesiredSize::new(44.0, 56.0));
    assert_eq!(bool_option, DesiredSize::new(56.0, 56.0));
}

#[test]
fn material_vector_fields_measure_visible_value_text() {
    let vector = measure_material_leaf(
        "Vector3Field",
        r#"
value = [0.0, 1.0, 0.0]
value_text = "0, 1, 0"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 28
"#,
        intrinsic_constraints(),
    );
    let color = measure_material_leaf(
        "ColorField",
        r##"
value = "#4d89ff"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_padding_top = 4
layout_padding_bottom = 4
layout_min_height = 28
"##,
        intrinsic_constraints(),
    );

    assert_eq!(vector, DesiredSize::new(74.0, 28.0));
    assert_eq!(color, DesiredSize::new(74.0, 28.0));
}

#[test]
fn asset_value_nodes_render_as_image_or_icon_not_text() {
    let image_command = render_material_leaf_command(
        "Image",
        r#"
value = "ui/editor/showcase_checker.svg"
image = "ui/editor/showcase_checker.svg"
"#,
    );
    let icon_command = render_material_leaf_command(
        "Icon",
        r#"
value = "ionicons/options-outline.svg"
icon = "options-outline"
"#,
    );
    let svg_icon_command = render_material_leaf_command(
        "SvgIcon",
        r#"
value = "ionicons/options-outline.svg"
source = "ionicons/options-outline.svg"
"#,
    );

    assert_eq!(image_command.kind, UiRenderCommandKind::Image);
    assert_eq!(image_command.text.as_deref(), None);
    assert_eq!(
        image_command.image,
        Some(UiVisualAssetRef::Image(
            "ui/editor/showcase_checker.svg".to_string()
        ))
    );
    assert_eq!(icon_command.kind, UiRenderCommandKind::Image);
    assert_eq!(icon_command.text.as_deref(), None);
    assert_eq!(
        icon_command.image,
        Some(UiVisualAssetRef::Icon("options-outline".to_string()))
    );
    assert_eq!(svg_icon_command.kind, UiRenderCommandKind::Image);
    assert_eq!(svg_icon_command.text.as_deref(), None);
    assert_eq!(
        svg_icon_command.image,
        Some(UiVisualAssetRef::Image(
            "ionicons/options-outline.svg".to_string()
        ))
    );
}

#[test]
fn icon_button_label_is_accessibility_text_not_rendered_text() {
    let icon_button_command = render_material_leaf_command(
        "IconButton",
        r#"
label = "Focus Console"
icon = "search-outline"
layout_min_width = 40
layout_min_height = 40
layout_icon_size = 18
layout_padding_left = 0
layout_padding_right = 0
layout_padding_top = 0
layout_padding_bottom = 0
"#,
    );

    assert_eq!(icon_button_command.kind, UiRenderCommandKind::Image);
    assert_eq!(icon_button_command.text.as_deref(), None);
}

#[test]
fn common_native_material_roles_use_authored_layout_metrics() {
    let progress = measure_material_leaf(
        "ProgressBar",
        r#"
layout_min_height = 8
"#,
        intrinsic_constraints(),
    );
    let spinner = measure_material_leaf(
        "Spinner",
        r#"
text = "Loading"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
layout_icon_size = 18
"#,
        intrinsic_constraints(),
    );
    let menu = measure_material_leaf(
        "ContextActionMenu",
        r#"
text = "Inspect"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(progress, DesiredSize::new(0.0, 8.0));
    assert_eq!(spinner, DesiredSize::new(74.0, 40.0));
    assert_eq!(menu, DesiredSize::new(74.0, 40.0));
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

fn render_material_leaf_command(
    component: &str,
    attributes: &str,
) -> zircon_runtime_interface::ui::surface::UiRenderCommand {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout.render"));
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
                .with_constraints(intrinsic_constraints())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .render_extract
        .list
        .commands
        .into_iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap_or_else(|| panic!("render extract should include `{component}` command"))
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
