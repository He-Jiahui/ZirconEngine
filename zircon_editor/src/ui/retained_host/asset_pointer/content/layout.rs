use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::workbench::asset_content_layout::AssetContentSurfaceProfile;
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceItemGeneration};

#[derive(Clone, Debug)]
pub(crate) struct AssetContentListPointerLayout {
    pub pane_size: UiSize,
    pub surface_profile: AssetContentSurfaceProfile,
    pub view_mode: AssetViewMode,
    pub folder_ids: Vec<String>,
    pub items: AssetWorkspaceItemGeneration,
}

impl PartialEq for AssetContentListPointerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.pane_size == other.pane_size
            && self.surface_profile == other.surface_profile
            && self.view_mode == other.view_mode
            && self.folder_ids == other.folder_ids
            && self.items.shares_item_identity_with(&other.items)
    }
}

impl Default for AssetContentListPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            surface_profile: AssetContentSurfaceProfile::Browser,
            view_mode: AssetViewMode::List,
            folder_ids: Vec::new(),
            items: AssetWorkspaceItemGeneration::default(),
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
            folder_ids: match surface_profile {
                AssetContentSurfaceProfile::Activity => snapshot
                    .visible_folders
                    .iter()
                    .map(|folder| folder.folder_id.clone())
                    .collect(),
                AssetContentSurfaceProfile::Browser => Vec::new(),
            },
            items: snapshot.visible_assets.clone(),
        }
    }

    pub(crate) fn item_count(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn item_uuid(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|item| item.uuid.as_str())
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        pane_size: UiSize,
        surface_profile: AssetContentSurfaceProfile,
        view_mode: AssetViewMode,
        folder_ids: Vec<String>,
        item_ids: Vec<String>,
    ) -> Self {
        use zircon_runtime_interface::resource::ResourceKind;

        let items = item_ids
            .into_iter()
            .map(|uuid| crate::ui::workbench::snapshot::AssetItemSnapshot {
                locator: format!("res://{uuid}"),
                display_name: uuid.clone(),
                file_name: uuid.clone(),
                extension: String::new(),
                kind: ResourceKind::Mesh,
                asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::default(),
                preview_artifact_path: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                selected: false,
                resource_state: None,
                resource_revision: None,
                uuid,
            })
            .collect();
        Self {
            pane_size,
            surface_profile,
            view_mode,
            folder_ids,
            items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::snapshot::{AssetFolderSnapshot, AssetWorkspaceSnapshot};

    #[test]
    fn browser_content_layout_keeps_source_tree_folders_out_of_asset_rows() {
        let snapshot = AssetWorkspaceSnapshot {
            visible_folders: vec![AssetFolderSnapshot {
                folder_id: "res://materials".to_string(),
                ..AssetFolderSnapshot::default()
            }],
            ..AssetWorkspaceSnapshot::default()
        };

        let browser = AssetContentListPointerLayout::from_snapshot(
            &snapshot,
            UiSize::new(420.0, 220.0),
            AssetContentSurfaceProfile::Browser,
        );
        let activity = AssetContentListPointerLayout::from_snapshot(
            &snapshot,
            UiSize::new(420.0, 220.0),
            AssetContentSurfaceProfile::Activity,
        );

        assert!(browser.folder_ids.is_empty());
        assert_eq!(activity.folder_ids, vec!["res://materials"]);
    }
}
