#[path = "scene_layers/chrome.rs"]
mod chrome;
#[path = "scene_layers/dock_layer.rs"]
mod dock_layer;
#[path = "scene_layers/overlay.rs"]
mod overlay;
#[path = "scene_layers/resize.rs"]
mod resize;

pub(in crate::ui::retained_host::host_contract) use self::overlay::draw_componentized_workbench_window;
#[cfg(test)]
pub(crate) use self::overlay::paint_componentized_extension_workspace_for_test;
