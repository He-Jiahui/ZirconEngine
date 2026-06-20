use super::model::WorkbenchTextFieldStyle;
use super::state::resolved_text_field_state;
use super::surface::{text_field_border, text_field_surface};
use super::text::{text_field_stepper, text_field_text};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_text_field_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchTextFieldStyle {
    let state = resolved_text_field_state(node);
    WorkbenchTextFieldStyle {
        surface: text_field_surface(node, state),
        border: text_field_border(node, state),
        text: text_field_text(state, label_is_placeholder),
        stepper: text_field_stepper(state),
        state,
    }
}
