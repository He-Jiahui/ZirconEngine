use super::super::super::data::{FrameRect, HostPageChromeData, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::menus;
use super::super::root_frames::{zero_origin, RootFrames};

pub(super) fn draw_top_chrome_layers(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    if frame
        .paint_clip()
        .is_some_and(|damage| intersect(&root.top_bar, damage).is_none())
    {
        return;
    }
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
        if let Some(clip) = page_chrome_paint_clip(&root.top_bar, &scene.page_chrome) {
            draw_template_nodes(
                frame,
                &scene.page_chrome.template_nodes,
                &zero_origin(),
                &clip,
                None,
            );
        }
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_menu_bar_labels");
        menus::draw_menu_bar_labels(frame, presentation);
    }
}

fn page_chrome_paint_clip(top_bar: &FrameRect, page: &HostPageChromeData) -> Option<FrameRect> {
    // The page root includes the menu spacer, but its background owns only the
    // published page band used by native page input.
    if page.tab_row_frame.width > 0.0 && page.tab_row_frame.height > 0.0 {
        intersect(top_bar, &page.tab_row_frame)
    } else if page
        .template_nodes
        .iter()
        .any(|node| node.control_id.as_str() == "WorkbenchPageBar")
    {
        None
    } else {
        Some(top_bar.clone())
    }
}

pub(super) fn draw_status_bar_template_nodes(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    if frame
        .paint_clip()
        .is_some_and(|damage| intersect(&root.status_bar, damage).is_none())
    {
        return;
    }
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

#[cfg(test)]
#[path = "tests/chrome_page_paint_clip_tests.rs"]
mod page_paint_clip_tests;
