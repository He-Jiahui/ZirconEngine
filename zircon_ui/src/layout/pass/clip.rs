use crate::UiFrame;

pub(crate) fn resolve_clip_frame(
    inherited_clip: Option<UiFrame>,
    frame: UiFrame,
    clip_to_bounds: bool,
) -> Option<UiFrame> {
    let clipped = inherited_clip.and_then(|clip| clip.intersection(frame));
    if clip_to_bounds {
        clipped.or(Some(frame))
    } else {
        clipped
    }
}
