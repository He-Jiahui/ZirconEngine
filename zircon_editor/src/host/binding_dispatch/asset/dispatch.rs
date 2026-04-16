use zircon_editor_ui::{AssetCommand, EditorUiBinding, EditorUiBindingPayload};

use super::super::error::EditorBindingDispatchError;
use super::asset_host_event::AssetHostEvent;
use super::surface::parse_asset_surface;
use super::utility_tab::parse_asset_utility_tab;
use super::view_mode::parse_asset_view_mode;

pub fn dispatch_asset_binding(
    binding: &EditorUiBinding,
) -> Result<AssetHostEvent, EditorBindingDispatchError> {
    let EditorUiBindingPayload::AssetCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    match command {
        AssetCommand::OpenAsset { asset_path } => Ok(AssetHostEvent::OpenAsset {
            asset_path: asset_path.clone(),
        }),
        AssetCommand::SelectFolder { folder_id } => Ok(AssetHostEvent::SelectFolder {
            folder_id: folder_id.clone(),
        }),
        AssetCommand::SelectItem { asset_uuid } => Ok(AssetHostEvent::SelectItem {
            asset_uuid: asset_uuid.clone(),
        }),
        AssetCommand::ActivateReference { asset_uuid } => Ok(AssetHostEvent::ActivateReference {
            asset_uuid: asset_uuid.clone(),
        }),
        AssetCommand::SetSearchQuery { query } => Ok(AssetHostEvent::SetSearchQuery {
            query: query.clone(),
        }),
        AssetCommand::SetKindFilter { kind } => Ok(AssetHostEvent::SetKindFilter {
            kind: (!kind.is_empty()).then_some(kind.clone()),
        }),
        AssetCommand::SetViewMode { surface, view_mode } => Ok(AssetHostEvent::SetViewMode {
            surface: parse_asset_surface(surface)?,
            view_mode: parse_asset_view_mode(view_mode)?,
        }),
        AssetCommand::SetUtilityTab { surface, tab } => Ok(AssetHostEvent::SetUtilityTab {
            surface: parse_asset_surface(surface)?,
            tab: parse_asset_utility_tab(tab)?,
        }),
        AssetCommand::OpenAssetBrowser => Ok(AssetHostEvent::OpenAssetBrowser),
        AssetCommand::LocateSelectedAsset => Ok(AssetHostEvent::LocateSelectedAsset),
        AssetCommand::ImportModel => Ok(AssetHostEvent::ImportModel),
    }
}
