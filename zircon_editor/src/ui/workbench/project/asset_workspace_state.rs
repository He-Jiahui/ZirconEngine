use std::collections::HashMap;
use std::sync::Arc;

use crate::core::asset::AssetTypeId;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetDetailsGeneration,
    EditorAssetFolderRecord,
};
use zircon_runtime::core::framework::asset::{ResourceManagementGeneration, ResourceManagementRow};
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetSelectionSnapshot,
    AssetSubassetSnapshot, AssetSurfaceMode, AssetTypeProjectionSnapshot, AssetUtilityTab,
    AssetViewMode, AssetWorkspaceSnapshot, ProjectOverviewSnapshot,
};
use zircon_runtime::asset::project::AssetSourceUnit;
use zircon_runtime::asset::AssetUri;

#[derive(Clone, Debug)]
pub(crate) struct AssetWorkspaceState {
    catalog: Option<Arc<EditorAssetCatalogGeneration>>,
    selected_folder_id: String,
    selected_asset_uuid: Option<String>,
    selected_details: Option<Arc<EditorAssetDetailsGeneration>>,
    resources: Arc<ResourceManagementGeneration>,
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
            resources: Arc::new(ResourceManagementGeneration::default()),
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
    pub fn sync_catalog(&mut self, catalog: Arc<EditorAssetCatalogGeneration>) {
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

    pub fn sync_selected_details(&mut self, details: Option<Arc<EditorAssetDetailsGeneration>>) {
        self.selected_details = details;
    }

    pub fn sync_resources(&mut self, resources: Arc<ResourceManagementGeneration>) -> bool {
        if Arc::ptr_eq(&self.resources, &resources) {
            return false;
        }
        self.resources = resources;
        true
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

    pub(crate) fn asset_type_id_for_locator(&self, locator: &AssetUri) -> Option<AssetTypeId> {
        let locator = locator.to_string();
        self.catalog
            .as_ref()?
            .asset_by_locator(&locator)
            .map(|asset| AssetTypeId::from_resource_kind(asset.kind))
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

        let normalized_search_query = self.search_query.to_ascii_lowercase();
        let folder_tree = build_folder_tree(catalog.folders.as_ref(), &self.selected_folder_id);
        let visible_folders = catalog
            .folders
            .iter()
            .filter(|folder| {
                folder.parent_folder_id.as_deref() == Some(self.selected_folder_id.as_str())
            })
            .filter(|folder| folder_matches_search(folder, &normalized_search_query))
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
            .filter(|asset| {
                asset_matches_filters(asset, &normalized_search_query, self.kind_filter)
            })
            .map(|asset| self.asset_item_snapshot(asset))
            .collect::<Vec<_>>();

        AssetWorkspaceSnapshot {
            project_name: catalog.project_name.to_string(),
            project_root: catalog.project_root.to_string(),
            assets_root: catalog.assets_root.to_string(),
            cache_root: catalog.cache_root.to_string(),
            default_scene_uri: catalog.default_scene_uri.to_string(),
            catalog_revision: catalog.catalog_revision,
            surface_mode,
            view_mode: self.view_mode(surface_mode),
            utility_tab: self.utility_tab(surface_mode),
            search_query: self.search_query.clone(),
            kind_filter: self.kind_filter,
            folder_tree,
            visible_folders,
            visible_assets,
            creation_menu: Default::default(),
            selected_folder_id: Some(self.selected_folder_id.clone()),
            selected_asset_uuid: self.selected_asset_uuid.clone(),
            selection: self.selection_snapshot(),
        }
    }

    pub(crate) fn build_surface_snapshots(
        &self,
    ) -> (AssetWorkspaceSnapshot, AssetWorkspaceSnapshot) {
        let activity = self.build_snapshot(AssetSurfaceMode::Activity);
        let mut explorer = activity.clone();
        explorer.surface_mode = AssetSurfaceMode::Explorer;
        explorer.view_mode = self.view_mode(AssetSurfaceMode::Explorer);
        explorer.utility_tab = self.utility_tab(AssetSurfaceMode::Explorer);
        (activity, explorer)
    }

    pub fn project_overview(&self) -> ProjectOverviewSnapshot {
        let Some(catalog) = self.catalog.as_ref() else {
            return ProjectOverviewSnapshot::default();
        };

        ProjectOverviewSnapshot {
            project_name: catalog.project_name.to_string(),
            project_root: catalog.project_root.to_string(),
            assets_root: catalog.assets_root.to_string(),
            cache_root: catalog.cache_root.to_string(),
            default_scene_uri: catalog.default_scene_uri.to_string(),
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
        self.catalog
            .as_ref()
            .is_some_and(|catalog| catalog.folder(folder_id).is_some())
    }

    fn asset_record(&self, asset_uuid: &str) -> Option<&EditorAssetCatalogRecord> {
        self.catalog.as_ref()?.asset(asset_uuid)
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
        let resource = self.resources.row_by_locator(&asset.locator);

        AssetSelectionSnapshot {
            uuid: Some(asset.uuid.clone()),
            display_name: asset.display_name.clone(),
            locator: asset.locator.clone(),
            kind: Some(asset.kind),
            asset_type: asset_type_projection(asset.kind),
            preview_artifact_path: asset.preview_artifact_path.clone(),
            meta_path: asset.meta_path.clone(),
            toolkit_view_id: String::new(),
            toolkit_open_operation: String::new(),
            context_commands: Vec::new(),
            package_id: details
                .and_then(|details| details.package_id.as_deref())
                .map(str::to_string),
            asset_unit: details
                .map(|details| asset_unit_label(details.unit).to_string())
                .unwrap_or_default(),
            included_files: details
                .map(|details| details.included_files.to_vec())
                .unwrap_or_default(),
            subassets: details
                .map(|details| {
                    details
                        .subassets
                        .iter()
                        .map(|subasset| AssetSubassetSnapshot {
                            uuid: subasset.uuid.clone(),
                            locator: subasset.locator.clone(),
                            kind: subasset.kind,
                            asset_type: asset_type_projection(subasset.kind),
                            artifact_locator: subasset.artifact_locator.clone(),
                            dependency_locators: subasset.dependency_locators.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            diagnostics: asset.diagnostics.clone(),
            resource_state: resource_state(resource.as_deref()),
            resource_revision: resource.as_ref().map(|resource| resource.revision),
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
        let resource = self.resources.row_by_locator(&asset.locator);
        AssetItemSnapshot {
            uuid: asset.uuid.clone(),
            locator: asset.locator.clone(),
            display_name: asset.display_name.clone(),
            file_name: asset.file_name.clone(),
            extension: asset.extension.clone(),
            kind: asset.kind,
            asset_type: asset_type_projection(asset.kind),
            preview_artifact_path: asset.preview_artifact_path.clone(),
            dirty: asset.dirty,
            diagnostics: asset.diagnostics.clone(),
            selected: self.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str()),
            resource_state: resource_state(resource.as_deref()),
            resource_revision: resource.as_ref().map(|resource| resource.revision),
        }
    }
}

fn asset_type_projection(kind: ResourceKind) -> AssetTypeProjectionSnapshot {
    AssetTypeProjectionSnapshot::from_resource_kind(kind)
}

fn asset_unit_label(unit: AssetSourceUnit) -> &'static str {
    match unit {
        AssetSourceUnit::Single => "single",
        AssetSourceUnit::Compound => "compound",
    }
}

fn resource_state(resource: Option<&ResourceManagementRow>) -> Option<ResourceState> {
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
    if let Some(root_folders) = folders_by_parent.get(&None) {
        append_folder_branch(
            &mut tree,
            root_folders,
            &folders_by_parent,
            selected_folder_id,
            0,
        );
    }
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
    if let Some(package_path) = locator.strip_prefix("package://") {
        return package_path
            .rsplit_once('/')
            .map(|(parent, _)| format!("package://{parent}"))
            .unwrap_or_else(|| locator.to_string());
    }

    let locator_path = locator.strip_prefix("res://").unwrap_or(locator);
    locator_path
        .rsplit_once('/')
        .map(|(parent, _)| format!("res://{parent}"))
        .unwrap_or_else(|| "res://".to_string())
}

fn folder_matches_search(folder: &EditorAssetFolderRecord, normalized_search_query: &str) -> bool {
    if normalized_search_query.is_empty() {
        return true;
    }
    folder
        .display_name
        .to_ascii_lowercase()
        .contains(normalized_search_query)
}

fn asset_matches_filters(
    asset: &EditorAssetCatalogRecord,
    normalized_search_query: &str,
    kind_filter: Option<ResourceKind>,
) -> bool {
    let search_matches = if normalized_search_query.is_empty() {
        true
    } else {
        asset
            .display_name
            .to_ascii_lowercase()
            .contains(normalized_search_query)
            || asset
                .file_name
                .to_ascii_lowercase()
                .contains(normalized_search_query)
            || asset
                .locator
                .to_ascii_lowercase()
                .contains(normalized_search_query)
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
        asset_type: reference.kind.map(asset_type_projection),
        known_project_asset: reference.known_project_asset,
    }
}

#[cfg(test)]
mod performance_tests {
    use std::sync::Arc;

    use zircon_runtime::core::framework::asset::ResourceManagementGeneration;

    use super::{parent_folder_id_for_locator, AssetWorkspaceState};

    #[test]
    fn stable_resource_generation_skips_asset_projection_invalidation() {
        let mut workspace = AssetWorkspaceState::default();
        let generation = Arc::new(ResourceManagementGeneration::default());

        assert!(workspace.sync_resources(generation.clone()));
        assert!(!workspace.sync_resources(generation));
    }

    #[test]
    fn asset_snapshot_normalizes_search_once_and_streams_parent_paths() {
        let source = include_str!("asset_workspace_state.rs");
        let test_module = source
            .rfind("#[cfg(test)]")
            .expect("performance test module");
        let implementation = &source[..test_module];
        assert_eq!(
            implementation
                .matches("self.search_query.to_ascii_lowercase()")
                .count(),
            1
        );
        assert!(!implementation.contains("split('/').collect"));

        assert_eq!(parent_folder_id_for_locator("res://mesh.glb"), "res://");
        assert_eq!(
            parent_folder_id_for_locator("res://models/props/mesh.glb"),
            "res://models/props"
        );
        assert_eq!(
            parent_folder_id_for_locator("package://tools/mesh.glb"),
            "package://tools"
        );
        assert_eq!(
            parent_folder_id_for_locator("package://tools/models/mesh.glb"),
            "package://tools/models"
        );
    }

    #[test]
    fn dual_asset_surfaces_share_one_projection_build() {
        let source = include_str!("../snapshot/data/editor_state_snapshot_build.rs");
        assert!(source.contains("build_surface_snapshots()"));
        assert!(!source.contains(".build_snapshot(AssetSurfaceMode::"));
    }
}
