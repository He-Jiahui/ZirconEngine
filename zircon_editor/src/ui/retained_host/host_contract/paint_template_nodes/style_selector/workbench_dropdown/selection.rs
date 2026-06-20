use super::super::resolved_state_for_node;
use super::brightness::apply_visual_brightness;
use super::colors::{dropdown_border, dropdown_chevron, dropdown_surface, dropdown_text};
use super::model::WorkbenchDropdownStyle;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_dropdown_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchDropdownStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Dropdown);
    let style = WorkbenchDropdownStyle {
        surface: dropdown_surface(node, state),
        border: dropdown_border(node, state),
        text: dropdown_text(node, state, label_is_placeholder),
        chevron: dropdown_chevron(node, state),
        state,
    };
    apply_visual_brightness(style, node.label_brightness)
}
