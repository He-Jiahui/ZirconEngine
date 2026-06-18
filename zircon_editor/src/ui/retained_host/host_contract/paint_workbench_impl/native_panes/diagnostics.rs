use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_debug_reflector_overlay::draw_debug_reflector_overlay;
use super::super::super::paint_frame::HostRgbaFrame;

pub(super) fn draw_pane_debug_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    if pane.kind.as_str() != "RuntimeDiagnostics" {
        return false;
    }
    let primitives = (0..pane.runtime_diagnostics.overlay_primitives.row_count())
        .filter_map(|row| pane.runtime_diagnostics.overlay_primitives.row_data(row))
        .collect::<Vec<_>>();
    draw_debug_reflector_overlay(frame, &primitives, body, clip)
}
