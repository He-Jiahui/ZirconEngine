use super::super::super::data::HostWindowPresentationData;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::menus;
use super::super::root_frames::{zero_origin, RootFrames};

pub(super) fn draw_top_chrome_layers(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_menu_template_nodes");
        draw_template_nodes(
            frame,
            &scene.menu_chrome.template_nodes,
            &zero_origin(),
            &root.top_bar,
            None,
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_page_template_nodes");
        draw_template_nodes(
            frame,
            &scene.page_chrome.template_nodes,
            &zero_origin(),
            &root.top_bar,
            None,
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_menu_bar_labels");
        menus::draw_menu_bar_labels(frame, presentation);
    }
}

pub(super) fn draw_status_bar_template_nodes(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    zircon_runtime::profile_scope!(
        "editor",
        "host_painter",
        "painter_status_bar_template_nodes"
    );
    draw_template_nodes(
        frame,
        &scene.status_bar.template_nodes,
        &scene.status_bar.status_bar_frame,
        &root.status_bar,
        None,
    );
}
