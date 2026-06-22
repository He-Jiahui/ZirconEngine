use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::super::{first_non_empty, MUTED_TEXT};

pub(super) fn draw_pane_fallback(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) {
    let label = first_non_empty(&[
        pane.title.as_str(),
        pane.kind.as_str(),
        pane.subtitle.as_str(),
        pane.info.as_str(),
    ]);
    draw_text_bars_clipped(
        frame,
        body.x + 12.0,
        body.y + 16.0,
        label,
        Some(clip),
        MUTED_TEXT,
    );
}
