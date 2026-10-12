use super::*;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn timeline_grey_dot_uses_projected_tone_color() {
    let node = TemplatePaneNodeData::default();

    assert_eq!(
        timeline_dot_background_color(&node, false, [10, 11, 12, 255]),
        Some([10, 11, 12, 255])
    );
}

#[test]
fn timeline_dot_declared_background_overrides_projected_tone() {
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));

    assert_eq!(
        timeline_dot_background_color(&node, false, [10, 11, 12, 255]),
        Some([20, 21, 22, 255])
    );
}
