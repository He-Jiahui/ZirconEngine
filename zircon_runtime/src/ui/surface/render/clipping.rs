use zircon_runtime_interface::ui::layout::UiFrame;

/// Preserve a known empty clip so child render commands cannot become unclipped.
pub(super) fn intersect_clip_frame(clip_frame: Option<UiFrame>, frame: UiFrame) -> Option<UiFrame> {
    clip_frame.map(|clip| {
        let left = clip.x.max(frame.x);
        let top = clip.y.max(frame.y);
        let right = clip.right().min(frame.right());
        let bottom = clip.bottom().min(frame.bottom());
        UiFrame::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
    })
}
