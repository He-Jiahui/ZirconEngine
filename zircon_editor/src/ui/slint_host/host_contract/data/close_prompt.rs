use slint::SharedString;

use super::FrameRect;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostClosePromptData {
    pub visible: bool,
    pub target_window_id: SharedString,
    pub title: SharedString,
    pub message: SharedString,
    pub details: SharedString,
    pub can_save: bool,
    pub overlay_frame: FrameRect,
    pub dialog_frame: FrameRect,
    pub save_button_frame: FrameRect,
    pub discard_button_frame: FrameRect,
    pub cancel_button_frame: FrameRect,
}
