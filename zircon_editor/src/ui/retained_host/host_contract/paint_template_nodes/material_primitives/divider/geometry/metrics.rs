use super::super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_THICKNESS: f32 =
    1.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_MIDDLE_HORIZONTAL_INSET: f32 = 16.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_INSET_HORIZONTAL_INSET: f32 = 72.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_MIDDLE_VERTICAL_INSET: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_WRAPPER_HORIZONTAL_PADDING: f32 = 9.6;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_WRAPPER_VERTICAL_PADDING: f32 = 9.6;

const DIVIDER_DEFAULT_FONT_SIZE: f32 = 12.0;
const DIVIDER_MIN_FONT_SIZE: f32 = 8.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DIVIDER_DEFAULT_FONT_SIZE
    };
    requested
        .min((available_height * 0.82).max(DIVIDER_MIN_FONT_SIZE))
        .max(DIVIDER_MIN_FONT_SIZE)
}
