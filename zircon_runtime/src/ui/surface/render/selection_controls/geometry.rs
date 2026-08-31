use zircon_runtime_interface::ui::layout::UiFrame;

use super::{state::SelectionRenderState, style::SelectionVisual};

pub(super) fn leading_mark_rect(frame: UiFrame, visual: &SelectionVisual) -> UiFrame {
    UiFrame::new(
        frame.x + visual.mark_inset_x,
        frame.y + (frame.height - visual.mark_size).max(0.0) * 0.5,
        visual.mark_size,
        visual.mark_size,
    )
}

pub(super) fn label_rect_after_mark(
    frame: UiFrame,
    mark: UiFrame,
    visual: &SelectionVisual,
) -> UiFrame {
    let x = mark.x + mark.width + visual.label_gap;
    UiFrame::new(
        x,
        frame.y + visual.label_inset_y,
        (frame.x + frame.width - x - visual.mark_inset_x).max(visual.min_frame_extent),
        (frame.height - visual.label_inset_y * 2.0).max(visual.label_line_height),
    )
}

pub(super) fn toggle_label_rect(
    frame: UiFrame,
    track: UiFrame,
    visual: &SelectionVisual,
) -> UiFrame {
    UiFrame::new(
        frame.x + visual.mark_inset_x,
        frame.y + visual.label_inset_y,
        (track.x - frame.x - visual.mark_inset_x - visual.label_gap).max(visual.min_frame_extent),
        (frame.height - visual.label_inset_y * 2.0).max(visual.label_line_height),
    )
}

pub(super) fn toggle_track_rect(frame: UiFrame, visual: &SelectionVisual) -> UiFrame {
    let width = visual
        .toggle_track_width
        .min((frame.width - visual.mark_inset_x * 2.0).max(visual.min_frame_extent));
    let height = visual
        .toggle_track_height
        .min(frame.height.max(visual.min_frame_extent));
    UiFrame::new(
        (frame.x + frame.width - visual.toggle_right_inset - width).max(frame.x),
        frame.y + (frame.height - height).max(0.0) * 0.5,
        width,
        height,
    )
}

pub(super) fn toggle_thumb_rect(
    state: &SelectionRenderState,
    track: UiFrame,
    visual: &SelectionVisual,
) -> UiFrame {
    let size = visual
        .toggle_thumb_size
        .min(track.width)
        .min(track.height)
        .max(visual.min_frame_extent);
    let available = (track.width - size - visual.toggle_thumb_inset * 2.0).max(0.0);
    UiFrame::new(
        track.x + visual.toggle_thumb_inset + if state.active() { available } else { 0.0 },
        track.y + (track.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}

pub(super) fn centered_square(frame: UiFrame, size: f32) -> UiFrame {
    let size = size.min(frame.width).min(frame.height).max(f32::EPSILON);
    UiFrame::new(
        frame.x + (frame.width - size).max(0.0) * 0.5,
        frame.y + (frame.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}
