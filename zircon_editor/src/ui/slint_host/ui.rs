#[path = "ui/apply_presentation.rs"]
mod apply_presentation_impl;
mod pane_data_conversion;
#[cfg(test)]
#[path = "ui/reference_component_tests.rs"]
mod reference_component_tests;
#[cfg(test)]
#[path = "ui/structure_component_tests.rs"]
mod structure_component_tests;
#[path = "ui/template_node_conversion.rs"]
mod template_node_conversion;
#[cfg(test)]
mod tests;

pub(crate) use apply_presentation_impl::apply_presentation;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_animation_editor_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_console_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_hierarchy_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_inspector_pane_from_host_pane;
