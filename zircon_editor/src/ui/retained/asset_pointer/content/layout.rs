use std::sync::Arc;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::workbench::asset_content_layout::AssetContentSurfaceProfile;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetViewMode, AssetWorkspaceItemGeneration,
};

/// Pointer routing borrows the same immutable folders as retained paint publication.
#[derive(Clone, Debug, Default)]
pub(crate) struct AssetContentFolderIds(Arc<[AssetFolderSnapshot]>);

impl AssetContentFolderIds {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::ops::Index<usize> for AssetContentFolderIds {
    type Output = String;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index].folder_id
    }
}

impl PartialEq for AssetContentFolderIds {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || (self.0.is_empty() && other.0.is_empty())
    }
}

#[cfg(test)]
impl PartialEq<Vec<String>> for AssetContentFolderIds {
    fn eq(&self, other: &Vec<String>) -> bool {
        self.0.len() == other.len()
            && self
                .0
                .iter()
                .zip(other)
                .all(|(folder, id)| &folder.folder_id == id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AssetContentListPointerLayout {
    pub pane_size: UiSize,
    pub surface_profile: AssetContentSurfaceProfile,
    pub view_mode: AssetViewMode,
    pub folder_ids: AssetContentFolderIds,
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
            folder_ids: AssetContentFolderIds::default(),
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
                AssetContentSurfaceProfile::Activity => {
                    AssetContentFolderIds(Arc::clone(&snapshot.visible_folders))
                }
                AssetContentSurfaceProfile::Browser => AssetContentFolderIds::default(),
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
            folder_ids: AssetContentFolderIds(
                folder_ids
                    .into_iter()
                    .map(|folder_id| AssetFolderSnapshot {
                        folder_id,
                        ..Default::default()
                    })
                    .collect(),
            ),
            items,
        }
    }
}

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
