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
        surface: crate::EditorAssetSurface,
        view_mode: crate::EditorAssetViewMode,
    },
    SetUtilityTab {
        surface: crate::EditorAssetSurface,
        tab: crate::EditorAssetUtilityTab,
    },
    OpenAssetBrowser,
    LocateSelectedAsset,
    ImportModel,
}
