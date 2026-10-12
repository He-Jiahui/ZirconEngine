use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_debug_reflector_overlay::draw_debug_reflector_overlay_iter;
use super::super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn draw_pane_debug_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    if pane.kind.as_str() != "RuntimeDiagnostics" {
        return false;
    }
    let content_present = pane.runtime_diagnostics.overlay_primitives.row_count() > 0;
    let primitives = pane.runtime_diagnostics.overlay_primitives.iter();
    draw_debug_reflector_overlay_iter(frame, primitives, body, clip);
    content_present
}
