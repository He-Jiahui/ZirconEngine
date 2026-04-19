use zircon_runtime::ui::{binding::UiBindingCall, binding::UiBindingValue};

use super::DockCommand;
use crate::ui::binding::core::{
    required_f32_argument, required_string_argument, EditorUiBindingError,
};

impl DockCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::FocusView { instance_id } => UiBindingCall::new("DockCommand.FocusView")
                .with_argument(UiBindingValue::string(instance_id)),
            Self::CloseView { instance_id } => UiBindingCall::new("DockCommand.CloseView")
                .with_argument(UiBindingValue::string(instance_id)),
            Self::AttachViewToDrawer { instance_id, slot } => {
                UiBindingCall::new("DockCommand.AttachViewToDrawer")
                    .with_argument(UiBindingValue::string(instance_id))
                    .with_argument(UiBindingValue::string(slot))
            }
            Self::AttachViewToDocument {
                instance_id,
                page_id,
            } => UiBindingCall::new("DockCommand.AttachViewToDocument")
                .with_argument(UiBindingValue::string(instance_id))
                .with_argument(UiBindingValue::string(page_id)),
            Self::DetachViewToWindow {
                instance_id,
                window_id,
            } => UiBindingCall::new("DockCommand.DetachViewToWindow")
                .with_argument(UiBindingValue::string(instance_id))
                .with_argument(UiBindingValue::string(window_id)),
            Self::ActivateDrawerTab { slot, instance_id } => {
                UiBindingCall::new("DockCommand.ActivateDrawerTab")
                    .with_argument(UiBindingValue::string(slot))
                    .with_argument(UiBindingValue::string(instance_id))
            }
            Self::ActivateMainPage { page_id } => {
                UiBindingCall::new("DockCommand.ActivateMainPage")
                    .with_argument(UiBindingValue::string(page_id))
            }
            Self::SetDrawerMode { slot, mode } => UiBindingCall::new("DockCommand.SetDrawerMode")
                .with_argument(UiBindingValue::string(slot))
                .with_argument(UiBindingValue::string(mode)),
            Self::SetDrawerExtent { slot, extent } => {
                UiBindingCall::new("DockCommand.SetDrawerExtent")
                    .with_argument(UiBindingValue::string(slot))
                    .with_argument(UiBindingValue::Float(*extent as f64))
            }
            Self::SavePreset { name } => UiBindingCall::new("DockCommand.SavePreset")
                .with_argument(UiBindingValue::string(name)),
            Self::LoadPreset { name } => UiBindingCall::new("DockCommand.LoadPreset")
                .with_argument(UiBindingValue::string(name)),
            Self::ResetToDefault => UiBindingCall::new("DockCommand.ResetToDefault"),
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "DockCommand.FocusView" => Self::FocusView {
                instance_id: required_string_argument(&call, 0, "DockCommand.FocusView")?,
            },
            "DockCommand.CloseView" => Self::CloseView {
                instance_id: required_string_argument(&call, 0, "DockCommand.CloseView")?,
            },
            "DockCommand.AttachViewToDrawer" => Self::AttachViewToDrawer {
                instance_id: required_string_argument(&call, 0, "DockCommand.AttachViewToDrawer")?,
                slot: required_string_argument(&call, 1, "DockCommand.AttachViewToDrawer")?,
            },
            "DockCommand.AttachViewToDocument" => Self::AttachViewToDocument {
                instance_id: required_string_argument(
                    &call,
                    0,
                    "DockCommand.AttachViewToDocument",
                )?,
                page_id: required_string_argument(&call, 1, "DockCommand.AttachViewToDocument")?,
            },
            "DockCommand.DetachViewToWindow" => Self::DetachViewToWindow {
                instance_id: required_string_argument(&call, 0, "DockCommand.DetachViewToWindow")?,
                window_id: required_string_argument(&call, 1, "DockCommand.DetachViewToWindow")?,
            },
            "DockCommand.ActivateDrawerTab" => Self::ActivateDrawerTab {
                slot: required_string_argument(&call, 0, "DockCommand.ActivateDrawerTab")?,
                instance_id: required_string_argument(&call, 1, "DockCommand.ActivateDrawerTab")?,
            },
            "DockCommand.ActivateMainPage" => Self::ActivateMainPage {
                page_id: required_string_argument(&call, 0, "DockCommand.ActivateMainPage")?,
            },
            "DockCommand.SetDrawerMode" => Self::SetDrawerMode {
                slot: required_string_argument(&call, 0, "DockCommand.SetDrawerMode")?,
                mode: required_string_argument(&call, 1, "DockCommand.SetDrawerMode")?,
            },
            "DockCommand.SetDrawerExtent" => Self::SetDrawerExtent {
                slot: required_string_argument(&call, 0, "DockCommand.SetDrawerExtent")?,
                extent: required_f32_argument(&call, 1, "DockCommand.SetDrawerExtent")?,
            },
            "DockCommand.SavePreset" => Self::SavePreset {
                name: required_string_argument(&call, 0, "DockCommand.SavePreset")?,
            },
            "DockCommand.LoadPreset" => Self::LoadPreset {
                name: required_string_argument(&call, 0, "DockCommand.LoadPreset")?,
            },
            "DockCommand.ResetToDefault" => Self::ResetToDefault,
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
