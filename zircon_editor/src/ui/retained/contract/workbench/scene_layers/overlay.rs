#[path = "../overlay/componentized.rs"]
mod componentized;
#[path = "../overlay/dock_overflow.rs"]
mod dock_overflow;
#[path = "overlay/modal.rs"]
mod modal;
#[path = "../overlay/page_overflow.rs"]
mod page_overflow;
#[path = "../overlay/root_template.rs"]
mod root_template;

pub(in crate::ui::retained_host::host_contract) use self::componentized::draw_componentized_workbench_window;
#[cfg(test)]
pub(crate) use self::componentized::paint_componentized_extension_workspace_for_test;
