use super::super::super::super::data::FrameRect;
use super::super::super::{UiProfileFrame, UiProfileNamedFrame};
use super::visibility::{is_visible_frame, is_visible_profile_frame};

pub(in crate::ui::retained_host::host_contract) fn push_named_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: FrameRect,
    clip: Option<FrameRect>,
) {
    if !is_visible_frame(&frame) {
        return;
    }
    push_named_profile_frame(out, id, kind, surface, frame.into(), clip.map(Into::into));
}

pub(in crate::ui::retained_host::host_contract) fn push_named_profile_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: UiProfileFrame,
    clip: Option<UiProfileFrame>,
) {
    if !is_visible_profile_frame(&frame) {
        return;
    }
    out.push(UiProfileNamedFrame {
        id: id.to_string(),
        kind: kind.to_string(),
        surface: surface.to_string(),
        frame,
        clip,
    });
}
