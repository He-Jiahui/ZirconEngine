use super::model_rc::model_rc;
use super::*;

thread_local! {
    static PREVIEW_IMAGE_CACHE: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

pub(super) struct AssetSurfacePresentation {
    pub tree_folders: ModelRc<AssetFolderData>,
    pub content_folders: ModelRc<AssetFolderData>,
    pub content_items: ModelRc<AssetItemData>,
    pub selection: AssetSelectionData,
    pub references: ModelRc<AssetReferenceData>,
    pub used_by: ModelRc<AssetReferenceData>,
    pub search_query: SharedString,
    pub kind_filter: SharedString,
    pub view_mode: SharedString,
    pub utility_tab: SharedString,
}

pub(super) fn asset_surface_presentation(
    snapshot: &AssetWorkspaceSnapshot,
) -> AssetSurfacePresentation {
    AssetSurfacePresentation {
        tree_folders: model_rc(
            snapshot
                .folder_tree
                .iter()
                .map(|folder| AssetFolderData {
                    id: folder.folder_id.clone().into(),
                    name: folder.display_name.clone().into(),
                    count: folder.recursive_asset_count as i32,
                    depth: folder.depth as i32,
                    selected: folder.selected,
                })
                .collect(),
        ),
        content_folders: model_rc(
            snapshot
                .visible_folders
                .iter()
                .map(|folder| AssetFolderData {
                    id: folder.folder_id.clone().into(),
                    name: folder.display_name.clone().into(),
                    count: folder.recursive_asset_count as i32,
                    depth: folder.depth as i32,
                    selected: folder.selected,
                })
                .collect(),
        ),
        content_items: model_rc(
            snapshot
                .visible_assets
                .iter()
                .map(|item| AssetItemData {
                    uuid: item.uuid.clone().into(),
                    locator: item.locator.clone().into(),
                    name: item.display_name.clone().into(),
                    file_name: item.file_name.clone().into(),
                    kind: asset_kind_label(item.kind).into(),
                    extension: item.extension.clone().into(),
                    dirty: item.dirty,
                    has_error: item_has_error(item.resource_state, &item.diagnostics),
                    has_preview: !item.preview_artifact_path.is_empty(),
                    state: resource_state_label(item.resource_state).into(),
                    revision: item
                        .resource_revision
                        .map(|revision| revision.to_string())
                        .unwrap_or_default()
                        .into(),
                    selected: item.selected,
                    preview: preview_image(
                        &item.preview_artifact_path,
                        &format!(
                            "{}:{}:{}",
                            item.preview_artifact_path,
                            item.resource_revision.unwrap_or_default(),
                            item.dirty
                        ),
                    ),
                })
                .collect(),
        ),
        selection: AssetSelectionData {
            uuid: snapshot.selection.uuid.clone().unwrap_or_default().into(),
            name: snapshot.selection.display_name.clone().into(),
            locator: snapshot.selection.locator.clone().into(),
            kind: snapshot
                .selection
                .kind
                .map(asset_kind_label)
                .unwrap_or_default()
                .into(),
            meta_path: snapshot.selection.meta_path.clone().into(),
            adapter_key: snapshot.selection.adapter_key.clone().into(),
            state: resource_state_label(snapshot.selection.resource_state).into(),
            revision: snapshot
                .selection
                .resource_revision
                .map(|revision| revision.to_string())
                .unwrap_or_default()
                .into(),
            diagnostics: diagnostics_text(&snapshot.selection.diagnostics).into(),
            has_preview: !snapshot.selection.preview_artifact_path.is_empty(),
            preview: preview_image(
                &snapshot.selection.preview_artifact_path,
                &format!(
                    "{}:{}:{}",
                    snapshot.selection.preview_artifact_path,
                    snapshot.selection.resource_revision.unwrap_or_default(),
                    snapshot.selection.locator
                ),
            ),
        },
        references: model_rc(
            snapshot
                .selection
                .references
                .iter()
                .map(reference_data)
                .collect(),
        ),
        used_by: model_rc(
            snapshot
                .selection
                .used_by
                .iter()
                .map(reference_data)
                .collect(),
        ),
        search_query: snapshot.search_query.clone().into(),
        kind_filter: snapshot
            .kind_filter
            .map(asset_kind_label)
            .unwrap_or_default()
            .into(),
        view_mode: asset_view_mode_key(snapshot.view_mode).into(),
        utility_tab: asset_utility_tab_key(snapshot.utility_tab).into(),
    }
}

fn reference_data(reference: &crate::snapshot::AssetReferenceSnapshot) -> AssetReferenceData {
    AssetReferenceData {
        uuid: reference.uuid.clone().into(),
        locator: reference.locator.clone().into(),
        name: reference.display_name.clone().into(),
        kind: reference
            .kind
            .map(asset_kind_label)
            .unwrap_or_default()
            .into(),
        known_project_asset: reference.known_project_asset,
    }
}

fn asset_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "Texture",
        ResourceKind::Shader => "Shader",
        ResourceKind::Material => "Material",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model => "Model",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
    }
}

fn resource_state_label(state: Option<ResourceState>) -> &'static str {
    match state {
        Some(ResourceState::Pending) => "Pending",
        Some(ResourceState::Ready) => "Ready",
        Some(ResourceState::Error) => "Error",
        Some(ResourceState::Reloading) => "Reloading",
        None => "",
    }
}

fn item_has_error(state: Option<ResourceState>, diagnostics: &[String]) -> bool {
    matches!(state, Some(ResourceState::Error)) || !diagnostics.is_empty()
}

fn diagnostics_text(diagnostics: &[String]) -> String {
    diagnostics.join(" | ")
}

fn asset_view_mode_key(mode: AssetViewMode) -> &'static str {
    match mode {
        AssetViewMode::List => "list",
        AssetViewMode::Thumbnail => "thumbnail",
    }
}

fn asset_utility_tab_key(tab: AssetUtilityTab) -> &'static str {
    match tab {
        AssetUtilityTab::Preview => "preview",
        AssetUtilityTab::References => "references",
        AssetUtilityTab::Metadata => "metadata",
        AssetUtilityTab::Plugins => "plugins",
    }
}

fn preview_image(path: &str, cache_key: &str) -> Image {
    if path.is_empty() || !Path::new(path).exists() {
        return Image::default();
    }

    PREVIEW_IMAGE_CACHE.with(|cache| {
        if let Some(image) = cache.borrow().get(cache_key).cloned() {
            return image;
        }

        let loaded = Image::load_from_path(Path::new(path)).unwrap_or_default();
        cache
            .borrow_mut()
            .insert(cache_key.to_string(), loaded.clone());
        loaded
    })
}
