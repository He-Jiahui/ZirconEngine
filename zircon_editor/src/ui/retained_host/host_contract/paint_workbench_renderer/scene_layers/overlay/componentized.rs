use super::super::super::super::data::HostWindowPresentationData;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};
use super::super::super::root_frames::zero_origin;
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
    draw_template_nodes(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        &frame_bounds,
        Some(&presentation.text_input_focus),
    );
    draw_root_template_overlay(frame, presentation);
}
