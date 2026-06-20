use super::super::data::{FrameRect, HostClosePromptData};

pub(in crate::ui::retained_host::host_contract) fn prompt_details_frame(
    prompt: &HostClosePromptData,
) -> FrameRect {
    FrameRect {
        x: prompt.dialog_frame.x + 18.0,
        y: prompt.dialog_frame.y + 76.0,
        width: (prompt.dialog_frame.width - 36.0).max(0.0),
        height: 42.0,
    }
}
