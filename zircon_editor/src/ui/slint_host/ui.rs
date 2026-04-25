#[path = "ui/apply_presentation.rs"]
mod apply_presentation_impl;
#[path = "ui/pane_data_conversion.rs"]
mod pane_data_conversion;
#[cfg(test)]
mod tests;

pub(crate) use apply_presentation_impl::apply_presentation;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_slint_animation_editor_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_slint_console_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_slint_hierarchy_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_slint_inspector_pane_from_host_pane;
