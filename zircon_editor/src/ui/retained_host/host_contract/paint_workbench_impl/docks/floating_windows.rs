use super::super::super::data::{
    FloatingWindowData, FrameRect, HostPaneInteractionStateData, HostTextInputFocusData,
    HostViewportImageData, HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, translated};
use super::super::super::paint_primitives::{draw_border, draw_rect};
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::{ACCENT, FLOATING_PANEL, FLOATING_SHADOW, TOP_BAR};
use super::pane;

pub(in crate::ui::retained_host::host_contract) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    for row in 0..windows.row_count() {
        let Some(window) = windows.row_data(row) else {
            continue;
        };
        draw_floating_window(
            frame,
            &window,
            interaction,
            viewport_image,
            text_input_focus,
        );
    }
}

fn draw_floating_window(
    frame: &mut HostRgbaFrame,
    window: &FloatingWindowData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&window.frame) {
        return;
    }
    let shadow = FrameRect {
        x: window.frame.x + 4.0,
        y: window.frame.y + 5.0,
        width: window.frame.width,
        height: window.frame.height,
    };
    draw_rect(frame, shadow, FLOATING_SHADOW);
    draw_rect(frame, window.frame.clone(), FLOATING_PANEL);
    draw_border(frame, window.frame.clone(), ACCENT);

    let header = translated(&window.header_frame, window.frame.x, window.frame.y);
    if is_visible_frame(&header) {
        draw_rect(frame, header.clone(), TOP_BAR);
        draw_template_nodes(frame, &window.header_nodes, &window.frame, &header, None);
    }

    let body = FrameRect {
        x: window.frame.x + 1.0,
        y: header
            .y
            .max(window.frame.y)
            .saturating_add_f32(header.height.max(0.0) + 1.0),
        width: (window.frame.width - 2.0).max(0.0),
        height: (window.frame.height - header.height.max(0.0) - 2.0).max(0.0),
    };
    pane::draw_pane(
        frame,
        &window.active_pane,
        &body,
        interaction,
        viewport_image,
        text_input_focus,
    );
}

trait SaturatingAddF32 {
    fn saturating_add_f32(self, value: f32) -> f32;
}

impl SaturatingAddF32 for f32 {
    fn saturating_add_f32(self, value: f32) -> f32 {
        let result = self + value;
        if result.is_finite() {
            result
        } else {
            self
        }
    }
}
