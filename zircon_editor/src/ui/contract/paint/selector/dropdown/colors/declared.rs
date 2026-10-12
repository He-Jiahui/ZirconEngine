use super::super::super::super::template_style_color::resolved_style_color;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiStyleColor;

pub(super) fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

pub(super) fn declared_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}
