use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{intersect, is_visible_frame};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn effective_template_clip(
    frame: &HostRgbaFrame,
    clip: &FrameRect,
) -> Option<FrameRect> {
    match frame.paint_clip() {
        Some(active_clip) => intersect(active_clip, clip),
        None if is_visible_frame(clip) => Some(clip.clone()),
        None => None,
    }
}
