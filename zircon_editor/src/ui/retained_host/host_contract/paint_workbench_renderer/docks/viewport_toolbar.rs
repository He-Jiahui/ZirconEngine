use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::{MUTED_TEXT, SEPARATOR, TOOLBAR};

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_toolbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    toolbar: &FrameRect,
    clip: &FrameRect,
) {
    if !is_visible_frame(toolbar) {
        return;
    }
    draw_rect_clipped(frame, toolbar.clone(), Some(clip), TOOLBAR);
    draw_border_clipped(frame, toolbar.clone(), Some(clip), SEPARATOR);
    for (index, label) in [
        pane.viewport.tool.as_str(),
        pane.viewport.transform_space.as_str(),
        pane.viewport.display_mode.as_str(),
        pane.viewport.grid_mode.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
        draw_text_bars_clipped(
            frame,
            toolbar.x + 10.0 + index as f32 * 62.0,
            toolbar.y + 12.0,
            label,
            Some(clip),
            MUTED_TEXT,
        );
    }
}
