use crate::ui::retained_host::primitives::SharedString;

use super::super::FrameRect;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostTextInputFocusData {
    pub control_id: SharedString,
    pub dispatch_kind: SharedString,
    pub action_id: SharedString,
    pub edit_action_id: SharedString,
    pub commit_action_id: SharedString,
    pub value_text: SharedString,
    pub edit_frame: FrameRect,
}

impl HostTextInputFocusData {
    pub(crate) fn is_active(&self) -> bool {
        !self.control_id.is_empty()
    }

    pub(crate) fn captures_keyboard_chord(&self) -> bool {
        self.is_active() && self.dispatch_kind.as_str() == "chord_capture"
    }

    pub(crate) fn accepts_text_input(&self) -> bool {
        self.is_active() && !self.captures_keyboard_chord()
    }

    pub(crate) fn edit_target_id(&self) -> SharedString {
        if !self.edit_action_id.is_empty() {
            self.edit_action_id.clone()
        } else if !self.action_id.is_empty() {
            self.action_id.clone()
        } else {
            self.control_id.clone()
        }
    }

    pub(crate) fn commit_target_id(&self) -> SharedString {
        if !self.commit_action_id.is_empty() {
            self.commit_action_id.clone()
        } else {
            self.edit_target_id()
        }
    }
}
