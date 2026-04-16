use super::ShellFrame;

pub(super) fn default_floating_window_frame(
    index: usize,
    document_frame: ShellFrame,
    center_band_frame: ShellFrame,
) -> ShellFrame {
    let width = (document_frame.width * 0.38).clamp(320.0, 520.0);
    let height = (center_band_frame.height * 0.44).clamp(220.0, 340.0);
    let x = (document_frame.x + document_frame.width - width - 18.0 - index as f32 * 26.0)
        .max(document_frame.x + 12.0);
    let y = center_band_frame.y + 46.0 + index as f32 * 22.0;
    ShellFrame::new(x, y, width, height)
}

pub(super) fn clamp_floating_window_frame(frame: ShellFrame, available: ShellFrame) -> ShellFrame {
    if available.width <= 0.0 || available.height <= 0.0 {
        return ShellFrame::default();
    }

    let width = frame.width.max(220.0).min(available.width);
    let height = frame.height.max(160.0).min(available.height);
    let max_x = (available.right() - width).max(available.x);
    let max_y = (available.bottom() - height).max(available.y);

    ShellFrame::new(
        frame.x.max(available.x).min(max_x),
        frame.y.max(available.y).min(max_y),
        width,
        height,
    )
}
