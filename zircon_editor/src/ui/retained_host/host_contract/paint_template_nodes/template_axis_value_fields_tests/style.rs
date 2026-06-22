use crate::ui::retained_host::primitives::Color;

use super::super::super::template_axis_value_field_style::axis_field_text_color;
use super::support::axis_node;

#[test]
fn axis_value_field_uses_declared_value_color_when_present() {
    let mut node = axis_node("WorkbenchTransformPositionX", "128.4");
    node.value_color = Color::from_rgb_u8(146, 158, 164);

    assert_eq!(axis_field_text_color(&node), [146, 158, 164, 255]);
}
