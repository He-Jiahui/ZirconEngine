use std::collections::BTreeMap;

use crate::layout::{MainPageId, WorkbenchLayout};

use super::super::floating_window::{clamp_floating_window_frame, default_floating_window_frame};
use super::super::ShellFrame;

const EPSILON: f32 = 0.001;

pub(super) fn build_floating_window_frames(
    layout: &WorkbenchLayout,
    document_frame: ShellFrame,
    center_band_frame: ShellFrame,
) -> BTreeMap<MainPageId, ShellFrame> {
    layout
        .floating_windows
        .iter()
        .enumerate()
        .map(|(index, window)| {
            let requested_frame = if window.frame.width > EPSILON && window.frame.height > EPSILON {
                window.frame
            } else {
                default_floating_window_frame(index, document_frame, center_band_frame)
            };
            (
                window.window_id.clone(),
                clamp_floating_window_frame(requested_frame, center_band_frame),
            )
        })
        .collect()
}
