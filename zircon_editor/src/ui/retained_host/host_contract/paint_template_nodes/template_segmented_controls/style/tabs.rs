use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind as SegmentedStyleKind,
    WorkbenchSegmentedControlStyle,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tab_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let style = tab_style(node);
    if node.checked || node.selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::Tab)
}
