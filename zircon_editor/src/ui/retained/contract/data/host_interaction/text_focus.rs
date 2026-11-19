use crate::ui::retained_host::primitives::SharedString;

use super::super::FrameRect;

#[path = "../text_focus/editing.rs"]
mod editing;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostTextInputFocusData {
    pub control_id: SharedString,
    pub dispatch_kind: SharedString,
    pub action_id: SharedString,
    pub edit_action_id: SharedString,
    pub commit_action_id: SharedString,
    pub value_text: SharedString,
    /// A local text-edit operation, not a comparison with the rounded presentation value.
    pub details_axis_edited: bool,
    pub caret_scalar_offset: Option<usize>,
    pub selection_anchor_scalar_offset: Option<usize>,
    pub edit_frame: FrameRect,
    pub palette_search_target:
        Option<crate::ui::retained_host::host_contract::data::UiAssetPaletteInputTarget>,
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

    pub(in crate::ui::retained_host::host_contract) fn is_details_transform_axis(&self) -> bool {
        if self.dispatch_kind.as_str() != "workbench" {
            return false;
        }
        // Require the actual field identity and both normalized binding-path tokens.
        let axis = match self.control_id.as_str() {
            "WorkbenchTransformPositionX" => "position_x",
            "WorkbenchTransformPositionY" => "position_y",
            "WorkbenchTransformPositionZ" => "position_z",
            "WorkbenchTransformRotationX" => "rotation_x",
            "WorkbenchTransformRotationY" => "rotation_y",
            "WorkbenchTransformRotationZ" => "rotation_z",
            "WorkbenchTransformScaleX" => "scale_x",
            "WorkbenchTransformScaleY" => "scale_y",
            "WorkbenchTransformScaleZ" => "scale_z",
            _ => return false,
        };
        let edit = self
            .edit_action_id
            .as_str()
            .strip_prefix("inspector.transform.")
            .and_then(|action| action.strip_suffix(".edit"));
        let commit = self
            .commit_action_id
            .as_str()
            .strip_prefix("inspector.transform.")
            .and_then(|action| action.strip_suffix(".commit"));
        edit == Some(axis) && commit == Some(axis)
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
