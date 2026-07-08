use super::super::super::super::data::TemplatePaneNodeData;
#[cfg(test)]
use super::super::super::style_selector::WORKBENCH_SEGMENT_IDLE_BACKGROUND;
use super::super::super::style_selector::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind as SegmentedStyleKind,
    WorkbenchSegmentedControlStyle,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_IDLE_BACKGROUND: [u8; 4] = WORKBENCH_SEGMENT_IDLE_BACKGROUND;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_SELECTED_BACKGROUND:
    [u8; 4] = crate::ui::retained_host::host_contract::paint_theme::PALETTE.surface_pressed;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    segmented_control_style(node)
        .background
        .unwrap_or(SEGMENT_IDLE_BACKGROUND)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_text_color(
    node: &TemplatePaneNodeData,
    selected: bool,
) -> [u8; 4] {
    let style = segmented_control_style(node);
    if selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    segmented_control_style(node).selected_border_width
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    segmented_control_style(node).selected_surface
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_height(
    node: &TemplatePaneNodeData,
) -> f32 {
    segmented_control_style(node).selected_underline_height
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    segmented_control_style(node).selected_underline
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_group_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    segmented_control_style(node).group_label
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_control_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::SegmentedControl)
}
