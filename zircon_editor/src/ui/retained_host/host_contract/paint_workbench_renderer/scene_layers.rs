mod chrome;
mod dock_layer;
mod overlay;
mod resize;

use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::root_frames::RootFrames;

pub(in crate::ui::retained_host::host_contract) use self::overlay::{
    draw_componentized_workbench_window, draws_componentized_workbench_window,
};

pub(in crate::ui::retained_host::host_contract) fn draw_host_scene(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    chrome::draw_top_chrome_layers(frame, root, presentation);
    dock_layer::draw_dock_layers(frame, presentation);
    resize::draw_resize_layer(frame, presentation);
    dock_layer::draw_floating_layer(frame, presentation);
    overlay::draw_menu_and_prompt_layers(frame, presentation);
    chrome::draw_status_bar_template_nodes(frame, root, presentation);
    overlay::draw_profiled_root_template_overlay(frame, presentation);
}
