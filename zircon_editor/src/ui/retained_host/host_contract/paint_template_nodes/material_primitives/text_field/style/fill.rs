use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::resolved_style_color;

const MUI_FIELD_FILLED_BACKGROUND: [u8; 4] = [255, 255, 255, 23];
const MUI_FIELD_FILLED_HOVER_BACKGROUND: [u8; 4] = [255, 255, 255, 31];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_FILLED_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if node.hovered {
            MUI_FIELD_FILLED_HOVER_BACKGROUND
        } else {
            MUI_FIELD_FILLED_BACKGROUND
        }
    })
}
