use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetHostEvent {
    OpenAsset {
        asset_path: String,
    },
    SelectFolder {
        folder_id: String,
    },
    SelectItem {
        asset_uuid: String,
    },
    ActivateReference {
        asset_uuid: String,
    },
    SetSearchQuery {
        query: String,
    },
    SetKindFilter {
        kind: Option<String>,
    },
    SetViewMode {
        surface: crate::core::editor_event::EditorAssetSurface,
        view_mode: crate::core::editor_event::EditorAssetViewMode,
    },
    SetUtilityTab {
        surface: crate::core::editor_event::EditorAssetSurface,
        tab: crate::core::editor_event::EditorAssetUtilityTab,
    },
    OpenAssetBrowser,
    LocateSelectedAsset,
    ImportModel,
}
