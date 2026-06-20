use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::paint_close_prompt::draw_close_prompt;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::is_visible_frame;
use super::super::paint_primitives::draw_rect;
use super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};
use super::root_frames::{zero_origin, RootFrames};
use super::{docks, menus};

pub(in crate::ui::retained_host::host_contract) fn draw_host_scene(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let viewport_image = presentation.viewport_image.as_ref();
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

    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_left_dock");
        docks::draw_side_dock(
            frame,
            &scene.left_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock");
        docks::draw_document_dock(
            frame,
            &scene.document_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_right_dock");
        docks::draw_side_dock(
            frame,
            &scene.right_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock");
        docks::draw_bottom_dock(
            frame,
            &scene.bottom_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_resize_layer");
        draw_resize_layer(frame, presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_floating_layer");
        docks::draw_floating_layer(
            frame,
            presentation,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_open_menu_popup");
        menus::draw_open_menu_popup(frame, presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_close_prompt");
        draw_close_prompt(frame, presentation);
    }

    {
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
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_root_template_overlay");
        draw_root_template_overlay(frame, presentation);
    }
}

pub(in crate::ui::retained_host::host_contract) fn draws_componentized_workbench_window(
    presentation: &HostWindowPresentationData,
) -> bool {
    has_template_nodes(&presentation.workbench_window_nodes)
}

pub(in crate::ui::retained_host::host_contract) fn draw_componentized_workbench_window(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: frame.width() as f32,
        height: frame.height() as f32,
    };
    draw_template_nodes(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        &frame_bounds,
        Some(&presentation.text_input_focus),
    );
    draw_root_template_overlay(frame, presentation);
}

fn draw_root_template_overlay(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    if !has_template_nodes(&presentation.root_template_nodes) {
        return;
    }

    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: frame.width() as f32,
        height: frame.height() as f32,
    };
    draw_template_nodes(
        frame,
        &presentation.root_template_nodes,
        &zero_origin(),
        &frame_bounds,
        None,
    );
}

fn draw_resize_layer(frame: &mut HostRgbaFrame, presentation: &HostWindowPresentationData) {
    let resize = &presentation.host_scene_data.resize_layer;
    for splitter in [
        &resize.left_splitter_frame,
        &resize.right_splitter_frame,
        &resize.bottom_splitter_frame,
    ] {
        if is_visible_frame(splitter) {
            draw_rect(frame, splitter.clone(), [79, 92, 112, 255]);
        }
    }
}
