use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::{union_visible_frame, visible_frame};

pub(in crate::ui::retained_host::host_contract) fn close_prompt_action_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let prompt = &presentation.close_prompt;
    if !prompt.visible {
        return None;
    }
    union_visible_frame(
        visible_frame(&prompt.overlay_frame).then_some(prompt.overlay_frame.clone()),
        prompt.dialog_frame.clone(),
    )
}
