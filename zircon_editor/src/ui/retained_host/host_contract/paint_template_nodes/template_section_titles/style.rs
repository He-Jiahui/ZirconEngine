use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SECTION_FONT_SIZE: f32 =
    13.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SECTION_LINE_HEIGHT:
    f32 = SECTION_FONT_SIZE * 1.2;
const SECTION_TEXT: [u8; 4] = [225, 236, 240, 255];
const SECTION_TEXT_MUTED: [u8; 4] = [186, 201, 207, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SECTION_MESH_TEXT:
    [u8; 4] = [176, 186, 191, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.label_color) {
        color
    } else if node.control_id == "WorkbenchMeshLabel" {
        SECTION_MESH_TEXT
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        SECTION_TEXT_MUTED
    } else {
        SECTION_TEXT
    }
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
