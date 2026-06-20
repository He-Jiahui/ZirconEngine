use super::super::super::template_status_glyphs::normalized_status_mark_width;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_mark_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    normalized_status_mark_width(node.icon_stroke_width)
}
