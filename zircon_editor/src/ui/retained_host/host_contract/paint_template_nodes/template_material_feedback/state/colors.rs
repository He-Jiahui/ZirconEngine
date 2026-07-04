use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_style_color::resolved_style_color;
use super::super::palette::material_feedback_palette;
use super::tone::material_tone_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = material_feedback_palette();
    if node.disabled {
        return palette.disabled_track;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if is_workbench_progress_node(node) {
            palette.workbench_track
        } else {
            palette.track
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = material_feedback_palette();
    if node.disabled {
        return palette.disabled_fill;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node, &palette))
        .unwrap_or_else(|| {
            if is_workbench_progress_node(node) {
                palette.workbench_fill
            } else {
                palette.accent
            }
        })
}

fn is_workbench_progress_node(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str().starts_with("Workbench")
        || node.component_variant.as_str().contains("workbench")
}
