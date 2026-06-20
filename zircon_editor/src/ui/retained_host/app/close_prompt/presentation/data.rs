use super::super::model::{can_save_dirty_view, PendingClosePrompt};
use super::layout::close_prompt_layout;
use super::text::{dirty_details, prompt_message, prompt_title, target_window_id};
use crate::ui::retained_host::{FrameRect, HostClosePromptData, UiHostWindow};

pub(super) fn host_prompt_data(
    ui: &UiHostWindow,
    prompt: &PendingClosePrompt,
) -> HostClosePromptData {
    let layout = close_prompt_layout(ui);
    HostClosePromptData {
        visible: true,
        target_window_id: target_window_id(&prompt.target).into(),
        title: prompt_title(&prompt.target).into(),
        message: prompt_message(prompt.dirty_views.len()).into(),
        details: dirty_details(&prompt.dirty_views).into(),
        can_save: prompt.dirty_views.iter().all(can_save_dirty_view),
        overlay_frame: FrameRect {
            x: 0.0,
            y: 0.0,
            width: layout.width,
            height: layout.height,
        },
        dialog_frame: layout.dialog,
        save_button_frame: layout.save,
        discard_button_frame: layout.discard,
        cancel_button_frame: layout.cancel,
    }
}
