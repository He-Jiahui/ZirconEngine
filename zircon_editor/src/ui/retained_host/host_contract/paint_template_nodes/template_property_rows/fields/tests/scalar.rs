use super::*;

#[test]
fn selected_scalar_property_value_keeps_neutral_field_border() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    let palette = workbench_row_palette();
    let border = value_field_border_color(&node, palette);

    assert_eq!(border, palette.property_field_border);
    assert_ne!(border, palette.property_field_focus_border);
}

#[test]
fn focused_scalar_property_value_uses_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    let palette = workbench_row_palette();

    assert_eq!(
        value_field_border_color(&node, palette),
        palette.property_field_focus_border
    );
}

#[test]
fn pressed_scalar_property_value_uses_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.pressed = true;
    let palette = workbench_row_palette();

    assert_eq!(
        value_field_border_color(&node, palette),
        palette.property_field_focus_border
    );
}
