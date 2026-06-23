use super::*;

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
