use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

use super::super::dialog::projected_dialog_actions;
use super::super::showcase_actions::preferred_showcase_action_buttons;

pub(super) fn projected_actions(
    control_id: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
    component_descriptor: Option<&UiComponentDescriptor>,
) -> Vec<host_contract::TemplatePaneActionData> {
    let dialog_actions = projected_dialog_actions(component_role, attributes);
    if !dialog_actions.is_empty() {
        dialog_actions
    } else if component_descriptor.is_some() {
        preferred_showcase_action_buttons(control_id, bindings)
    } else {
        Vec::new()
    }
}
