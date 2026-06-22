mod selection;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{
    FrameRect, HostTextInputFocusData, PaneData, TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};

use selection::select_pane_template_nodes;

pub(super) fn draw_pane_template_nodes(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    select_pane_template_nodes(pane)
        .map(|nodes| draw_if_present(frame, nodes, body, clip, text_input_focus))
        .unwrap_or(false)
}

fn draw_if_present(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    has_template_nodes(nodes) && draw_template_nodes(frame, nodes, origin, clip, text_input_focus)
}
