mod classify;
mod family;
mod layout;
mod roles;
mod visual_language;
mod workbench;

pub(in crate::ui::retained_host::host_contract) use classify::{
    is_any_component_family, is_component_family, template_component_family,
};
pub(in crate::ui::retained_host::host_contract) use family::TemplateComponentFamily;
pub(in crate::ui::retained_host::host_contract) use visual_language::uses_workbench_visual_language;

#[cfg(test)]
#[path = "template_component_family_tests.rs"]
mod tests;
