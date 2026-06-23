use super::*;

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
fn text_field_placeholder_measures_without_becoming_editable_value() {
    let command = render_material_leaf_command(
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
    );

    assert_eq!(
        command.text.as_deref(),
        Some("Describe runtime material layout")
    );
    assert_eq!(command.frame.width, 224.0);
    assert_eq!(
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .map(|editable| editable.text.as_str()),
        Some("")
    );
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
