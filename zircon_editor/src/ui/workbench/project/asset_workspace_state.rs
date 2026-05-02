use std::collections::HashMap;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord,
};
use zircon_runtime_interface::resource::{ResourceKind, ResourceRecord, ResourceState};

use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetSelectionSnapshot,
    AssetSurfaceMode, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
    ProjectOverviewSnapshot,
};

#[derive(Clone, Debug)]
pub(crate) struct AssetWorkspaceState {
    catalog: Option<EditorAssetCatalogSnapshotRecord>,
    selected_folder_id: String,
    selected_asset_uuid: Option<String>,
    selected_details: Option<EditorAssetDetailsRecord>,
    resources_by_locator: HashMap<String, ResourceRecord>,
    search_query: String,
    kind_filter: Option<ResourceKind>,
    activity_view_mode: AssetViewMode,
    browser_view_mode: AssetViewMode,
    activity_utility_tab: AssetUtilityTab,
    browser_utility_tab: AssetUtilityTab,
}

impl Default for AssetWorkspaceState {
    fn default() -> Self {
        Self {
            catalog: None,
            selected_folder_id: "res://".to_string(),
            selected_asset_uuid: None,
            selected_details: None,
            resources_by_locator: HashMap::new(),
            search_query: String::new(),
            kind_filter: None,
            activity_view_mode: AssetViewMode::List,
            browser_view_mode: AssetViewMode::Thumbnail,
            activity_utility_tab: AssetUtilityTab::Preview,
            browser_utility_tab: AssetUtilityTab::Preview,
        }
    }
}

impl AssetWorkspaceState {
    pub fn sync_catalog(&mut self, catalog: EditorAssetCatalogSnapshotRecord) {
        self.catalog = Some(catalog);

        if !self.folder_exists(&self.selected_folder_id) {
            self.selected_folder_id = "res://".to_string();
        }
        if self
            .selected_asset_uuid
            .as_ref()
            .is_some_and(|uuid| self.asset_record(uuid).is_none())
        {
            self.selected_asset_uuid = None;
            self.selected_details = None;
        }
    }

    pub fn sync_selected_details(&mut self, details: Option<EditorAssetDetailsRecord>) {
        self.selected_details = details;
    }

    pub fn sync_resources(&mut self, resources: Vec<ResourceRecord>) {
        self.resources_by_locator = resources
            .into_iter()
            .map(|resource| (resource.primary_locator.to_string(), resource))
            .collect();
    }

    pub fn select_folder(&mut self, folder_id: impl Into<String>) {
        let folder_id = folder_id.into();
        if self.folder_exists(&folder_id) {
            self.selected_folder_id = folder_id;
            if self
                .selected_asset_uuid
                .as_ref()
                .is_some_and(|uuid| !self.asset_belongs_to_folder(uuid, &self.selected_folder_id))
            {
                self.selected_asset_uuid = None;
                self.selected_details = None;
            }
        }
    }

    pub fn select_asset(&mut self, asset_uuid: Option<String>) {
        self.selected_asset_uuid = asset_uuid.filter(|uuid| self.asset_record(uuid).is_some());
        if self.selected_details.as_ref().is_some_and(|details| {
            Some(details.asset.uuid.as_str()) != self.selected_asset_uuid.as_deref()
        }) {
            self.selected_details = None;
        }
    }

    pub fn navigate_to_asset(&mut self, asset_uuid: &str) {
        if let Some(record) = self.asset_record(asset_uuid) {
            self.selected_folder_id = parent_folder_id_for_locator(&record.locator);
            self.selected_asset_uuid = Some(asset_uuid.to_string());
        }
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    pub fn set_kind_filter(&mut self, kind_filter: Option<ResourceKind>) {
        self.kind_filter = kind_filter;
    }

    pub fn set_activity_view_mode(&mut self, view_mode: AssetViewMode) {
        self.activity_view_mode = view_mode;
    }

    pub fn set_browser_view_mode(&mut self, view_mode: AssetViewMode) {
        self.browser_view_mode = view_mode;
    }

    pub fn set_activity_utility_tab(&mut self, utility_tab: AssetUtilityTab) {
        self.activity_utility_tab = utility_tab;
    }

    pub fn set_browser_utility_tab(&mut self, utility_tab: AssetUtilityTab) {
        self.browser_utility_tab = utility_tab;
    }

    #[cfg(test)]
    pub fn selected_folder_id(&self) -> &str {
        &self.selected_folder_id
    }

    #[cfg(test)]
    pub fn selected_asset_uuid(&self) -> Option<&str> {
        self.selected_asset_uuid.as_deref()
    }

    pub fn build_snapshot(&self, surface_mode: AssetSurfaceMode) -> AssetWorkspaceSnapshot {
        let Some(catalog) = self.catalog.as_ref() else {
            return AssetWorkspaceSnapshot {
                surface_mode,
                view_mode: self.view_mode(surface_mode),
                utility_tab: self.utility_tab(surface_mode),
                search_query: self.search_query.clone(),
                kind_filter: self.kind_filter,
                selected_folder_id: Some(self.selected_folder_id.clone()),
                selected_asset_uuid: self.selected_asset_uuid.clone(),
                ..AssetWorkspaceSnapshot::default()
            };
        };

        let folder_tree = build_folder_tree(&catalog.folders, &self.selected_folder_id);
        let visible_folders = catalog
            .folders
            .iter()
            .filter(|folder| {
                folder.parent_folder_id.as_deref() == Some(self.selected_folder_id.as_str())
            })
            .filter(|folder| folder_matches_search(folder, &self.search_query))
            .map(|folder| AssetFolderSnapshot {
                folder_id: folder.folder_id.clone(),
                parent_folder_id: folder.parent_folder_id.clone(),
                display_name: folder.display_name.clone(),
                recursive_asset_count: folder.recursive_asset_count,
                depth: 0,
                selected: folder.folder_id == self.selected_folder_id,
            })
            .collect::<Vec<_>>();
        let visible_assets = catalog
            .assets
            .iter()
            .filter(|asset| asset_belongs_to_folder(asset, &self.selected_folder_id))
            .filter(|asset| asset_matches_filters(asset, &self.search_query, self.kind_filter))
            .map(|asset| self.asset_item_snapshot(asset))
            .collect::<Vec<_>>();

        AssetWorkspaceSnapshot {
            project_name: catalog.project_name.clone(),
            project_root: catalog.project_root.clone(),
            assets_root: catalog.assets_root.clone(),
            library_root: catalog.library_root.clone(),
            default_scene_uri: catalog.default_scene_uri.clone(),
            catalog_revision: catalog.catalog_revision,
            surface_mode,
            view_mode: self.view_mode(surface_mode),
            utility_tab: self.utility_tab(surface_mode),
            search_query: self.search_query.clone(),
            kind_filter: self.kind_filter,
            folder_tree,
            visible_folders,
            visible_assets,
            selected_folder_id: Some(self.selected_folder_id.clone()),
            selected_asset_uuid: self.selected_asset_uuid.clone(),
            selection: self.selection_snapshot(),
        }
    }

    pub fn project_overview(&self) -> ProjectOverviewSnapshot {
        let Some(catalog) = self.catalog.as_ref() else {
            return ProjectOverviewSnapshot::default();
        };

        ProjectOverviewSnapshot {
            project_name: catalog.project_name.clone(),
            project_root: catalog.project_root.clone(),
            assets_root: catalog.assets_root.clone(),
            library_root: catalog.library_root.clone(),
            default_scene_uri: catalog.default_scene_uri.clone(),
            catalog_revision: catalog.catalog_revision,
            folder_count: catalog.folders.len(),
            asset_count: catalog.assets.len(),
        }
    }

    fn view_mode(&self, surface_mode: AssetSurfaceMode) -> AssetViewMode {
        match surface_mode {
            AssetSurfaceMode::Activity => self.activity_view_mode,
            AssetSurfaceMode::Explorer => self.browser_view_mode,
        }
    }

    fn utility_tab(&self, surface_mode: AssetSurfaceMode) -> AssetUtilityTab {
        match surface_mode {
            AssetSurfaceMode::Activity => self.activity_utility_tab,
            AssetSurfaceMode::Explorer => self.browser_utility_tab,
        }
    }

    fn folder_exists(&self, folder_id: &str) -> bool {
        self.catalog.as_ref().is_some_and(|catalog| {
            catalog
                .folders
                .iter()
                .any(|folder| folder.folder_id == folder_id)
        })
    }

    fn asset_record(&self, asset_uuid: &str) -> Option<&EditorAssetCatalogRecord> {
        self.catalog
            .as_ref()?
            .assets
            .iter()
            .find(|asset| asset.uuid == asset_uuid)
    }

    fn asset_belongs_to_folder(&self, asset_uuid: &str, folder_id: &str) -> bool {
        self.asset_record(asset_uuid)
            .is_some_and(|asset| asset_belongs_to_folder(asset, folder_id))
    }

    fn selection_snapshot(&self) -> AssetSelectionSnapshot {
        let Some(selected_uuid) = self.selected_asset_uuid.as_ref() else {
            return AssetSelectionSnapshot::default();
        };
        let Some(asset) = self.asset_record(selected_uuid) else {
            return AssetSelectionSnapshot::default();
        };
        let details = self
            .selected_details
            .as_ref()
            .filter(|details| details.asset.uuid == *selected_uuid);
        let resource = self.resources_by_locator.get(&asset.locator);

        AssetSelectionSnapshot {
            uuid: Some(asset.uuid.clone()),
            display_name: asset.display_name.clone(),
            locator: asset.locator.clone(),
            kind: Some(asset.kind),
            preview_artifact_path: asset.preview_artifact_path.clone(),
            meta_path: asset.meta_path.clone(),
            adapter_key: details
                .and_then(|details| details.editor_adapter.clone())
                .unwrap_or_default(),
            diagnostics: asset.diagnostics.clone(),
            resource_state: resource_state(resource),
            resource_revision: resource.map(|resource| resource.revision),
            references: details
                .map(|details| {
                    details
                        .direct_references
                        .iter()
                        .map(reference_snapshot)
                        .collect()
                })
                .unwrap_or_default(),
            used_by: details
                .map(|details| {
                    details
                        .referenced_by
                        .iter()
                        .map(reference_snapshot)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn asset_item_snapshot(&self, asset: &EditorAssetCatalogRecord) -> AssetItemSnapshot {
        let resource = self.resources_by_locator.get(&asset.locator);
        AssetItemSnapshot {
            uuid: asset.uuid.clone(),
            locator: asset.locator.clone(),
            display_name: asset.display_name.clone(),
            file_name: asset.file_name.clone(),
            extension: asset.extension.clone(),
            kind: asset.kind,
            preview_artifact_path: asset.preview_artifact_path.clone(),
            dirty: asset.dirty,
            diagnostics: asset.diagnostics.clone(),
            selected: self.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str()),
            resource_state: resource_state(resource),
            resource_revision: resource.map(|resource| resource.revision),
        }
    }
}

fn resource_state(resource: Option<&ResourceRecord>) -> Option<ResourceState> {
    resource.map(|resource| resource.state)
}

fn build_folder_tree(
    folders: &[EditorAssetFolderRecord],
    selected_folder_id: &str,
) -> Vec<AssetFolderSnapshot> {
    let mut folders_by_parent = HashMap::<Option<&str>, Vec<&EditorAssetFolderRecord>>::new();
    for folder in folders {
        folders_by_parent
            .entry(folder.parent_folder_id.as_deref())
            .or_default()
            .push(folder);
    }
    for children in folders_by_parent.values_mut() {
        children.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    }

    let mut tree = Vec::new();
    append_folder_branch(
        &mut tree,
        folders_by_parent
            .get(&None)
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
        &folders_by_parent,
        selected_folder_id,
        0,
    );
    tree
}

fn append_folder_branch(
    out: &mut Vec<AssetFolderSnapshot>,
    branch: &[&EditorAssetFolderRecord],
    folders_by_parent: &HashMap<Option<&str>, Vec<&EditorAssetFolderRecord>>,
    selected_folder_id: &str,
    depth: usize,
) {
    for folder in branch {
        out.push(AssetFolderSnapshot {
            folder_id: folder.folder_id.clone(),
            parent_folder_id: folder.parent_folder_id.clone(),
            display_name: folder.display_name.clone(),
            recursive_asset_count: folder.recursive_asset_count,
            depth,
            selected: folder.folder_id == selected_folder_id,
        });
        if let Some(children) = folders_by_parent.get(&Some(folder.folder_id.as_str())) {
            append_folder_branch(
                out,
                children,
                folders_by_parent,
                selected_folder_id,
                depth + 1,
            );
        }
    }
}

fn asset_belongs_to_folder(asset: &EditorAssetCatalogRecord, folder_id: &str) -> bool {
    parent_folder_id_for_locator(&asset.locator) == folder_id
}

fn parent_folder_id_for_locator(locator: &str) -> String {
    let locator_path = locator.strip_prefix("res://").unwrap_or(locator);
    let mut segments = locator_path.split('/').collect::<Vec<_>>();
    if segments.len() <= 1 {
        return "res://".to_string();
    }
    segments.pop();
    format!("res://{}", segments.join("/"))
}

fn folder_matches_search(folder: &EditorAssetFolderRecord, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }
    folder
        .display_name
        .to_ascii_lowercase()
        .contains(&search_query.to_ascii_lowercase())
}

fn asset_matches_filters(
    asset: &EditorAssetCatalogRecord,
    search_query: &str,
    kind_filter: Option<ResourceKind>,
) -> bool {
    let search_matches = if search_query.is_empty() {
        true
    } else {
        let needle = search_query.to_ascii_lowercase();
        asset.display_name.to_ascii_lowercase().contains(&needle)
            || asset.file_name.to_ascii_lowercase().contains(&needle)
            || asset.locator.to_ascii_lowercase().contains(&needle)
    };
    let kind_matches = kind_filter.is_none_or(|kind| asset.kind == kind);
    search_matches && kind_matches
}

fn reference_snapshot(
    reference: &crate::ui::host::editor_asset_manager::EditorAssetReferenceRecord,
) -> AssetReferenceSnapshot {
    AssetReferenceSnapshot {
        uuid: reference.uuid.clone(),
        locator: reference.locator.clone(),
        display_name: reference.display_name.clone(),
        kind: reference.kind,
        known_project_asset: reference.known_project_asset,
    }
}
