use super::super::super::super::data::HostWindowPresentationData;
use super::super::super::super::data::{FrameRect, HostWindowLayoutData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};
use super::super::super::root_frames::zero_origin;
use super::super::{dock_layer, resize};
use super::modal;
use super::page_overflow::draw_host_page_overflow_menu;
use super::root_template::{draw_root_template_overlay, frame_bounds};

pub(in crate::ui::retained_host::host_contract) fn draws_componentized_workbench_window(
    presentation: &HostWindowPresentationData,
) -> bool {
    has_template_nodes(&presentation.workbench_window_nodes)
}

pub(in crate::ui::retained_host::host_contract) fn draw_componentized_workbench_window(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let frame_bounds = frame_bounds(frame);
    draw_componentized_workbench_chrome(frame, presentation, &frame_bounds);
    // The componentized template owns the chrome shell, while existing host scene data
    // remains the source of pane, viewport, splitter, and floating surface content.
    dock_layer::draw_dock_layers(frame, presentation);
    resize::draw_resize_layer(frame, presentation);
    dock_layer::draw_floating_layer(frame, presentation);
    draw_host_page_overflow_menu(frame, presentation);
    modal::draw_menu_and_prompt_layers(frame, presentation);
    draw_root_template_overlay(frame, presentation);
}

fn draw_componentized_workbench_chrome(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    frame_bounds: &FrameRect,
) {
    let Some((top_chrome, status_bar)) =
        componentized_chrome_clips(&presentation.host_layout, frame_bounds)
    else {
        draw_template_nodes(
            frame,
            &presentation.workbench_window_nodes,
            &zero_origin(),
            frame_bounds,
            Some(&presentation.text_input_focus),
        );
        return;
    };

    draw_componentized_workbench_chrome_clip(frame, presentation, &top_chrome);
    draw_componentized_workbench_chrome_clip(frame, presentation, &status_bar);
}

fn draw_componentized_workbench_chrome_clip(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    clip: &FrameRect,
) {
    if !visible_rect(clip) {
        return;
    }

    draw_template_nodes(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        clip,
        Some(&presentation.text_input_focus),
    );
}

fn componentized_chrome_clips(
    layout: &HostWindowLayoutData,
    frame_bounds: &FrameRect,
) -> Option<(FrameRect, FrameRect)> {
    if !visible_rect(&layout.center_band_frame) || !visible_rect(&layout.status_bar_frame) {
        return None;
    }

    let top_height = layout.center_band_frame.y.clamp(0.0, frame_bounds.height);
    Some((
        FrameRect {
            x: frame_bounds.x,
            y: frame_bounds.y,
            width: frame_bounds.width,
            height: top_height,
        },
        layout.status_bar_frame.clone(),
    ))
}

fn visible_rect(rect: &FrameRect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}
