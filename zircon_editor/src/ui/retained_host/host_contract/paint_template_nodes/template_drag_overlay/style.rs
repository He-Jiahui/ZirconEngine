use super::super::super::data::TemplatePaneNodeData;

const PREVIEW_SURFACE: [u8; 4] = [21, 48, 53, 255];
const PREVIEW_SURFACE_BLOCKED: [u8; 4] = [72, 32, 36, 255];
const PREVIEW_BORDER: [u8; 4] = [53, 199, 208, 255];
const PREVIEW_BORDER_BLOCKED: [u8; 4] = [239, 112, 102, 255];
const PREVIEW_TEXT: [u8; 4] = [206, 224, 226, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.drop_allowed {
        PREVIEW_SURFACE
    } else {
        PREVIEW_SURFACE_BLOCKED
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_accent_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.drop_allowed {
        PREVIEW_BORDER
    } else {
        PREVIEW_BORDER_BLOCKED
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_text_color(
) -> [u8; 4] {
    PREVIEW_TEXT
}
