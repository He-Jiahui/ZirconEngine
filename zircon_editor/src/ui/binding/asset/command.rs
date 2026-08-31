use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssetCommand {
    OpenAsset {
        asset_locator: String,
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
        kind: String,
    },
    SetViewMode {
        surface: String,
        view_mode: String,
    },
    SetUtilityTab {
        surface: String,
        tab: String,
    },
    RelocateAsset {
        asset_uuid: String,
        target_locator: String,
    },
    DeleteAsset {
        asset_uuid: String,
    },
    OpenAssetBrowser,
    LocateSelectedAsset,
    ImportModel,
}
