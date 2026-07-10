use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::workbench::asset_content_layout::AssetContentSurfaceProfile;
use crate::ui::workbench::snapshot::AssetViewMode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetContentListPointerLayout {
    pub pane_size: UiSize,
    pub surface_profile: AssetContentSurfaceProfile,
    pub view_mode: AssetViewMode,
    pub folder_ids: Vec<String>,
    pub item_ids: Vec<String>,
}

impl Default for AssetContentListPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            surface_profile: AssetContentSurfaceProfile::Browser,
            view_mode: AssetViewMode::List,
            folder_ids: Vec::new(),
            item_ids: Vec::new(),
        }
    }
}

impl AssetContentListPointerLayout {
    pub(crate) fn from_snapshot(
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        pane_size: UiSize,
        surface_profile: AssetContentSurfaceProfile,
    ) -> Self {
        Self {
            pane_size,
            surface_profile,
            view_mode: snapshot.view_mode,
            folder_ids: snapshot
                .visible_folders
                .iter()
                .map(|folder| folder.folder_id.clone())
                .collect(),
            item_ids: snapshot
                .visible_assets
                .iter()
                .map(|item| item.uuid.clone())
                .collect(),
        }
    }
}
