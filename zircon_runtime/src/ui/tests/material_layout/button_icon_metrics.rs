use super::*;

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
