mod componentized;
mod dock_overflow;
mod modal;
mod page_overflow;
mod root_template;

pub(in crate::ui::retained_host::host_contract) use self::componentized::draw_componentized_workbench_window;
#[cfg(test)]
pub(crate) use self::componentized::paint_componentized_extension_workspace_for_test;
