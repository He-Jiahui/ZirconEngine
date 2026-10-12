mod chrome;
mod dock_layer;
mod overlay;
mod resize;

pub(in crate::ui::retained_host::host_contract) use self::overlay::draw_componentized_workbench_window;
#[cfg(test)]
pub(crate) use self::overlay::paint_componentized_extension_workspace_for_test;
