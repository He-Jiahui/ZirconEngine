use super::super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_style_color(
    color: Option<&zircon_runtime_interface::ui::style::UiStyleColor>,
) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}
