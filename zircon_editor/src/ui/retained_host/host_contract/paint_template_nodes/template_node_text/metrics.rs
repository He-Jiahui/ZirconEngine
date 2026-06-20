use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TEXT_HORIZONTAL_INSET: f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TEXT_VERTICAL_INSET:
    f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MIN_TEXT_RECT_HEIGHT:
    f32 = 12.0;

const DEFAULT_TEMPLATE_FONT_SIZE: f32 = 12.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DEFAULT_TEMPLATE_FONT_SIZE
    };
    requested.min(available_height.max(1.0)).max(1.0)
}
