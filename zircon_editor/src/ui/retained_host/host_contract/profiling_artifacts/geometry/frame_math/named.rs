use super::super::super::super::data::FrameRect;
use super::super::super::{UiProfileFrame, UiProfileNamedFrame};
use super::visibility::{is_visible_frame, is_visible_profile_frame};

pub(in crate::ui::retained_host::host_contract) fn push_named_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: impl Into<String>,
    kind: impl Into<String>,
    surface: impl Into<String>,
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
    id: impl Into<String>,
    kind: impl Into<String>,
    surface: impl Into<String>,
    frame: UiProfileFrame,
    clip: Option<UiProfileFrame>,
) {
    if !is_visible_profile_frame(&frame) {
        return;
    }
    out.push(UiProfileNamedFrame {
        id: id.into(),
        kind: kind.into(),
        surface: surface.into(),
        frame,
        clip,
    });
}

#[cfg(test)]
#[path = "tests/named_optimization_batch_de_editor342_tests.rs"]
mod optimization_batch_de_editor342_tests;
