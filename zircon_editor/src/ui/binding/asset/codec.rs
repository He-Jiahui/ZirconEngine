use zircon_runtime_interface::ui::{binding::UiBindingCall, binding::UiBindingValue};

use super::AssetCommand;
use crate::ui::binding::core::{required_string_argument, EditorUiBindingError};

impl AssetCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::OpenAsset { asset_path } => UiBindingCall::new("AssetCommand.OpenAsset")
                .with_argument(UiBindingValue::string(asset_path)),
            Self::SelectFolder { folder_id } => UiBindingCall::new("AssetCommand.SelectFolder")
                .with_argument(UiBindingValue::string(folder_id)),
            Self::SelectItem { asset_uuid } => UiBindingCall::new("AssetCommand.SelectItem")
                .with_argument(UiBindingValue::string(asset_uuid)),
            Self::ActivateReference { asset_uuid } => {
                UiBindingCall::new("AssetCommand.ActivateReference")
                    .with_argument(UiBindingValue::string(asset_uuid))
            }
            Self::SetSearchQuery { query } => UiBindingCall::new("AssetCommand.SetSearchQuery")
                .with_argument(UiBindingValue::string(query)),
            Self::SetKindFilter { kind } => UiBindingCall::new("AssetCommand.SetKindFilter")
                .with_argument(UiBindingValue::string(kind)),
            Self::SetViewMode { surface, view_mode } => {
                UiBindingCall::new("AssetCommand.SetViewMode")
                    .with_argument(UiBindingValue::string(surface))
                    .with_argument(UiBindingValue::string(view_mode))
            }
            Self::SetUtilityTab { surface, tab } => {
                UiBindingCall::new("AssetCommand.SetUtilityTab")
                    .with_argument(UiBindingValue::string(surface))
                    .with_argument(UiBindingValue::string(tab))
            }
            Self::OpenAssetBrowser => UiBindingCall::new("AssetCommand.OpenAssetBrowser"),
            Self::LocateSelectedAsset => UiBindingCall::new("AssetCommand.LocateSelectedAsset"),
            Self::ImportModel => UiBindingCall::new("AssetCommand.ImportModel"),
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "AssetCommand.OpenAsset" => Self::OpenAsset {
                asset_path: required_string_argument(&call, 0, "AssetCommand.OpenAsset")?,
            },
            "AssetCommand.SelectFolder" => Self::SelectFolder {
                folder_id: required_string_argument(&call, 0, "AssetCommand.SelectFolder")?,
            },
            "AssetCommand.SelectItem" => Self::SelectItem {
                asset_uuid: required_string_argument(&call, 0, "AssetCommand.SelectItem")?,
            },
            "AssetCommand.ActivateReference" => Self::ActivateReference {
                asset_uuid: required_string_argument(&call, 0, "AssetCommand.ActivateReference")?,
            },
            "AssetCommand.SetSearchQuery" => Self::SetSearchQuery {
                query: required_string_argument(&call, 0, "AssetCommand.SetSearchQuery")?,
            },
            "AssetCommand.SetKindFilter" => Self::SetKindFilter {
                kind: required_string_argument(&call, 0, "AssetCommand.SetKindFilter")?,
            },
            "AssetCommand.SetViewMode" => Self::SetViewMode {
                surface: required_string_argument(&call, 0, "AssetCommand.SetViewMode")?,
                view_mode: required_string_argument(&call, 1, "AssetCommand.SetViewMode")?,
            },
            "AssetCommand.SetUtilityTab" => Self::SetUtilityTab {
                surface: required_string_argument(&call, 0, "AssetCommand.SetUtilityTab")?,
                tab: required_string_argument(&call, 1, "AssetCommand.SetUtilityTab")?,
            },
            "AssetCommand.OpenAssetBrowser" => Self::OpenAssetBrowser,
            "AssetCommand.LocateSelectedAsset" => Self::LocateSelectedAsset,
            "AssetCommand.ImportModel" => Self::ImportModel,
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
