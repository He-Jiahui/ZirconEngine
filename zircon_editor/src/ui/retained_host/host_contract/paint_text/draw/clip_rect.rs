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
    let extent = if super::super::font::active_authored_font_request().is_some() {
        clip.unwrap_or(rect)
    } else { rect };
    let pixel_clip =
        PixelRect::from_frame(extent, effective_clip.as_ref(), frame.width(), frame.height())?;
    Some((pixel_clip, effective_clip))
}
