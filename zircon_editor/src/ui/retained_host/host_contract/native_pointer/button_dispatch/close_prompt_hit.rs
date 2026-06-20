use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::primitives::SharedString;

use super::super::routing::contains;

pub(in crate::ui::retained_host::host_contract) fn close_prompt_action_at(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<SharedString> {
    let prompt = &presentation.close_prompt;
    if !prompt.visible {
        return None;
    }
    if prompt.can_save && contains(&prompt.save_button_frame, x, y) {
        return Some("save".into());
    }
    if contains(&prompt.discard_button_frame, x, y) {
        return Some("discard".into());
    }
    if contains(&prompt.cancel_button_frame, x, y) {
        return Some("cancel".into());
    }
    None
}
