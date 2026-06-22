use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::clip::effective_clip;

pub(super) fn resolve_text_pixel_clip(
    frame: &HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
) -> Option<(PixelRect, Option<FrameRect>)> {
    let effective_clip = effective_clip(frame, clip)?;
    let pixel_clip =
        PixelRect::from_frame(rect, effective_clip.as_ref(), frame.width(), frame.height())?;
    Some((pixel_clip, effective_clip))
}
