#[path = "component_family/classify.rs"]
mod classify;
#[path = "component_family/family.rs"]
mod family;
#[path = "component_family/layout.rs"]
mod layout;
#[path = "component_family/roles.rs"]
mod roles;
#[path = "component_family/visual_language.rs"]
mod visual_language;
#[path = "component_family/workbench.rs"]
mod workbench;

pub(in crate::ui::retained_host::host_contract) use classify::{
    is_any_component_family, is_component_family, template_component_family,
};
pub(in crate::ui::retained_host::host_contract) use family::TemplateComponentFamily;
pub(in crate::ui::retained_host::host_contract) use visual_language::uses_workbench_visual_language;

#[cfg(test)]
#[path = "tests/component_family.rs"]
mod tests;
