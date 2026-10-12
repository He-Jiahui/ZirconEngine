use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract) fn effective_clip(
    frame: &HostRgbaFrame,
    clip: Option<&FrameRect>,
) -> Option<Option<FrameRect>> {
    match (frame.paint_clip(), clip) {
        (Some(active), Some(clip)) => intersect(active, clip).map(Some),
        (Some(active), None) => Some(Some(active.clone())),
        (None, Some(clip)) => Some(Some(clip.clone())),
        (None, None) => Some(None),
    }
}
