use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_style_color::resolved_style_color;
use super::tone::material_tone_color;

const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];
const WORKBENCH_PROGRESS_TRACK: [u8; 4] = PALETTE.surface_inset;
const WORKBENCH_PROGRESS_FILL: [u8; 4] = PALETTE.separator_strong;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if is_workbench_progress_node(node) {
            WORKBENCH_PROGRESS_TRACK
        } else {
            MATERIAL_PROGRESS_TRACK
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node))
        .unwrap_or_else(|| {
            if is_workbench_progress_node(node) {
                WORKBENCH_PROGRESS_FILL
            } else {
                PALETTE.accent
            }
        })
}

fn is_workbench_progress_node(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str().starts_with("Workbench")
        || node.component_variant.as_str().contains("workbench")
}
