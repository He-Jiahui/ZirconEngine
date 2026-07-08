use super::palette::workbench_segmented_control_palette;
use super::state::is_unavailable_segmented_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_text_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        palette.disabled_text
    } else {
        palette.selected_text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn idle_text_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        palette.disabled_text
    } else {
        palette.idle_text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn group_label_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        return palette.disabled_text;
    }
    let base = if node.label_color.a > 0 {
        [
            node.label_color.r,
            node.label_color.g,
            node.label_color.b,
            node.label_color.a,
        ]
    } else {
        palette.group_label
    };
    color_with_brightness(base, node.label_brightness)
}

fn color_with_brightness(mut color: [u8; 4], brightness: f32) -> [u8; 4] {
    let brightness = if brightness.is_finite() && brightness > 0.0 {
        brightness
    } else {
        1.0
    };
    for channel in &mut color[0..3] {
        *channel = ((*channel as f32 * brightness).round()).clamp(0.0, 255.0) as u8;
    }
    color
}
