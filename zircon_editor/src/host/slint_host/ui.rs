use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use slint::{Image, ModelRc, SharedString, VecModel};
use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};
use zircon_manager::{AssetRecordKind, ResourceStateRecord};

use crate::layout::ActivityDrawerSlot;
use crate::snapshot::{
    AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot,
    ProjectOverviewSnapshot, ViewContentKind, ViewTabSnapshot,
};
use crate::workbench::model::{
    DocumentTabModel, HostPageTabModel, PaneActionModel, PaneEmptyStateModel, WorkbenchViewModel,
};
use crate::workbench::startup::RecentProjectValidation;
use crate::{ShellFrame, ShellRegionId, WorkbenchShellGeometry};

use super::{
    AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData, BreadcrumbData,
    FrameRect, NewProjectFormData, PaneData, ProjectOverviewData, RecentProjectData,
    SceneNodeData, TabData, WelcomePaneData, WorkbenchShell,
};

thread_local! {
    static PREVIEW_IMAGE_CACHE: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

pub(crate) fn apply_presentation(
    ui: &WorkbenchShell,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) {
    let presentation =
        ShellPresentation::from_state(&model, chrome, preset_names, active_preset_name);

    ui.set_host_tabs(presentation.host_tabs);
    ui.set_breadcrumbs(presentation.breadcrumbs);
    ui.set_left_tabs(presentation.left_tabs);
    ui.set_right_tabs(presentation.right_tabs);
    ui.set_bottom_tabs(presentation.bottom_tabs);
    ui.set_document_tabs(presentation.document_tabs);
    ui.set_left_pane(presentation.left_pane);
    ui.set_right_pane(presentation.right_pane);
    ui.set_bottom_pane(presentation.bottom_pane);
    ui.set_document_pane(presentation.document_pane);
    ui.set_welcome_pane(presentation.welcome.pane);
    ui.set_recent_projects(presentation.welcome.recent_projects);
    ui.set_hierarchy_nodes(presentation.hierarchy_nodes);
    ui.set_project_overview(presentation.project_overview);
    ui.set_activity_asset_tree_folders(presentation.activity.tree_folders);
    ui.set_activity_asset_content_folders(presentation.activity.content_folders);
    ui.set_activity_asset_content_items(presentation.activity.content_items);
    ui.set_activity_asset_selection(presentation.activity.selection);
    ui.set_activity_asset_references(presentation.activity.references);
    ui.set_activity_asset_used_by(presentation.activity.used_by);
    ui.set_activity_asset_search_query(presentation.activity.search_query);
    ui.set_activity_asset_kind_filter(presentation.activity.kind_filter);
    ui.set_activity_asset_view_mode(presentation.activity.view_mode);
    ui.set_activity_asset_utility_tab(presentation.activity.utility_tab);
    ui.set_browser_asset_tree_folders(presentation.browser.tree_folders);
    ui.set_browser_asset_content_folders(presentation.browser.content_folders);
    ui.set_browser_asset_content_items(presentation.browser.content_items);
    ui.set_browser_asset_selection(presentation.browser.selection);
    ui.set_browser_asset_references(presentation.browser.references);
    ui.set_browser_asset_used_by(presentation.browser.used_by);
    ui.set_browser_asset_search_query(presentation.browser.search_query);
    ui.set_browser_asset_kind_filter(presentation.browser.kind_filter);
    ui.set_browser_asset_view_mode(presentation.browser.view_mode);
    ui.set_browser_asset_utility_tab(presentation.browser.utility_tab);
    ui.set_project_path(presentation.project_path);
    ui.set_status_primary(presentation.status_primary);
    ui.set_status_secondary(presentation.status_secondary);
    ui.set_viewport_label(presentation.viewport_label);
    ui.set_drawers_visible(presentation.drawers_visible);
    ui.set_left_expanded(presentation.left_expanded);
    ui.set_right_expanded(presentation.right_expanded);
    ui.set_bottom_expanded(presentation.bottom_expanded);
    ui.set_left_drawer_extent(presentation.left_drawer_extent);
    ui.set_right_drawer_extent(presentation.right_drawer_extent);
    ui.set_bottom_drawer_extent(presentation.bottom_drawer_extent);
    ui.set_save_project_enabled(presentation.save_project_enabled);
    ui.set_undo_enabled(presentation.undo_enabled);
    ui.set_redo_enabled(presentation.redo_enabled);
    ui.set_delete_enabled(presentation.delete_enabled);
    ui.set_inspector_name(presentation.inspector_name);
    ui.set_inspector_parent(presentation.inspector_parent);
    ui.set_inspector_x(presentation.inspector_x);
    ui.set_inspector_y(presentation.inspector_y);
    ui.set_inspector_z(presentation.inspector_z);
    ui.set_mesh_import_path(presentation.mesh_import_path);
    ui.set_preset_names(presentation.preset_names);
    ui.set_active_preset_name(presentation.active_preset_name);
    ui.set_shell_min_width_px(geometry.window_min_width);
    ui.set_shell_min_height_px(geometry.window_min_height);
    ui.set_center_band_frame(frame_rect(geometry.center_band_frame));
    ui.set_status_bar_frame(frame_rect(geometry.status_bar_frame));
    ui.set_left_region_frame(frame_rect(geometry.region_frame(ShellRegionId::Left)));
    ui.set_document_region_frame(frame_rect(geometry.region_frame(ShellRegionId::Document)));
    ui.set_right_region_frame(frame_rect(geometry.region_frame(ShellRegionId::Right)));
    ui.set_bottom_region_frame(frame_rect(geometry.region_frame(ShellRegionId::Bottom)));
    ui.set_left_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Left)));
    ui.set_right_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Right)));
    ui.set_bottom_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Bottom)));
    ui.set_viewport_content_frame(frame_rect(geometry.viewport_content_frame));
}

fn frame_rect(frame: ShellFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

struct ShellPresentation {
    host_tabs: ModelRc<TabData>,
    breadcrumbs: ModelRc<BreadcrumbData>,
    left_tabs: ModelRc<TabData>,
    right_tabs: ModelRc<TabData>,
    bottom_tabs: ModelRc<TabData>,
    document_tabs: ModelRc<TabData>,
    left_pane: PaneData,
    right_pane: PaneData,
    bottom_pane: PaneData,
    document_pane: PaneData,
    welcome: WelcomePresentation,
    hierarchy_nodes: ModelRc<SceneNodeData>,
    project_overview: ProjectOverviewData,
    activity: AssetSurfacePresentation,
    browser: AssetSurfacePresentation,
    project_path: SharedString,
    status_primary: SharedString,
    status_secondary: SharedString,
    viewport_label: SharedString,
    drawers_visible: bool,
    left_expanded: bool,
    right_expanded: bool,
    bottom_expanded: bool,
    left_drawer_extent: f32,
    right_drawer_extent: f32,
    bottom_drawer_extent: f32,
    save_project_enabled: bool,
    undo_enabled: bool,
    redo_enabled: bool,
    delete_enabled: bool,
    inspector_name: SharedString,
    inspector_parent: SharedString,
    inspector_x: SharedString,
    inspector_y: SharedString,
    inspector_z: SharedString,
    mesh_import_path: SharedString,
    preset_names: ModelRc<SharedString>,
    active_preset_name: SharedString,
}

struct AssetSurfacePresentation {
    tree_folders: ModelRc<AssetFolderData>,
    content_folders: ModelRc<AssetFolderData>,
    content_items: ModelRc<AssetItemData>,
    selection: AssetSelectionData,
    references: ModelRc<AssetReferenceData>,
    used_by: ModelRc<AssetReferenceData>,
    search_query: SharedString,
    kind_filter: SharedString,
    view_mode: SharedString,
    utility_tab: SharedString,
}

struct WelcomePresentation {
    pane: WelcomePaneData,
    recent_projects: ModelRc<RecentProjectData>,
}

impl ShellPresentation {
    fn from_state(
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        preset_names: &[String],
        active_preset_name: Option<&str>,
    ) -> Self {
        let left_tabs = collect_tabs(
            model,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        );
        let right_tabs = collect_tabs(
            model,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
        );
        let bottom_tabs = collect_tabs(
            model,
            &[
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerSlot::BottomRight,
            ],
        );

        let left_expanded = side_expanded(
            model,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        );
        let right_expanded = side_expanded(
            model,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
        );
        let bottom_expanded = side_expanded(
            model,
            &[
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerSlot::BottomRight,
            ],
        );
        let left_drawer_extent = drawer_extent(
            chrome,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            COLLAPSED_SIDE_EXTENT,
        );
        let right_drawer_extent = drawer_extent(
            chrome,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
            COLLAPSED_SIDE_EXTENT,
        );
        let bottom_drawer_extent = drawer_extent(
            chrome,
            &[
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerSlot::BottomRight,
            ],
            COLLAPSED_BOTTOM_EXTENT,
        );
        let activity = asset_surface_presentation(&chrome.asset_activity);
        let browser = asset_surface_presentation(&chrome.asset_browser);
        let welcome = welcome_presentation(&chrome.welcome);

        Self {
            host_tabs: model_rc(
                model
                    .host_strip
                    .pages
                    .iter()
                    .map(|page| host_tab_data(page, &model.host_strip.active_page))
                    .collect(),
            ),
            breadcrumbs: model_rc(
                model
                    .host_strip
                    .breadcrumbs
                    .iter()
                    .map(|crumb| BreadcrumbData {
                        label: crumb.label.clone().into(),
                    })
                    .collect(),
            ),
            left_tabs: model_rc(left_tabs),
            right_tabs: model_rc(right_tabs),
            bottom_tabs: model_rc(bottom_tabs),
            document_tabs: model_rc(model.document_tabs.iter().map(document_tab_data).collect()),
            left_pane: side_pane(
                model,
                chrome,
                &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            ),
            right_pane: side_pane(
                model,
                chrome,
                &[
                    ActivityDrawerSlot::RightTop,
                    ActivityDrawerSlot::RightBottom,
                ],
            ),
            bottom_pane: side_pane(
                model,
                chrome,
                &[
                    ActivityDrawerSlot::BottomLeft,
                    ActivityDrawerSlot::BottomRight,
                ],
            ),
            document_pane: document_pane(model, chrome),
            welcome,
            hierarchy_nodes: model_rc(
                chrome
                    .scene_entries
                    .iter()
                    .map(|entry| SceneNodeData {
                        id: entry.id.to_string().into(),
                        name: entry.name.clone().into(),
                        depth: entry.depth as i32,
                        selected: entry.selected,
                    })
                    .collect(),
            ),
            project_overview: project_overview_data(&chrome.project_overview),
            activity,
            browser,
            project_path: chrome.project_path.clone().into(),
            status_primary: chrome.status_line.clone().into(),
            status_secondary: model
                .status_bar
                .secondary_text
                .clone()
                .unwrap_or_default()
                .into(),
            viewport_label: model.status_bar.viewport_label.clone().into(),
            drawers_visible: model.drawer_ring.visible,
            left_expanded,
            right_expanded,
            bottom_expanded,
            left_drawer_extent,
            right_drawer_extent,
            bottom_drawer_extent,
            save_project_enabled: chrome.project_open,
            undo_enabled: chrome.can_undo,
            redo_enabled: chrome.can_redo,
            delete_enabled: chrome.inspector.is_some(),
            inspector_name: chrome
                .inspector
                .as_ref()
                .map(|inspector| inspector.name.clone())
                .unwrap_or_default()
                .into(),
            inspector_parent: chrome
                .inspector
                .as_ref()
                .map(|inspector| inspector.parent.clone())
                .unwrap_or_default()
                .into(),
            inspector_x: chrome
                .inspector
                .as_ref()
                .map(|inspector| inspector.translation[0].clone())
                .unwrap_or_default()
                .into(),
            inspector_y: chrome
                .inspector
                .as_ref()
                .map(|inspector| inspector.translation[1].clone())
                .unwrap_or_default()
                .into(),
            inspector_z: chrome
                .inspector
                .as_ref()
                .map(|inspector| inspector.translation[2].clone())
                .unwrap_or_default()
                .into(),
            mesh_import_path: chrome.mesh_import_path.clone().into(),
            preset_names: model_rc(
                preset_names
                    .iter()
                    .cloned()
                    .map(SharedString::from)
                    .collect(),
            ),
            active_preset_name: active_preset_name.unwrap_or_default().into(),
        }
    }
}

const COLLAPSED_SIDE_EXTENT: f32 = 56.0;
const COLLAPSED_BOTTOM_EXTENT: f32 = 48.0;

fn project_overview_data(snapshot: &ProjectOverviewSnapshot) -> ProjectOverviewData {
    ProjectOverviewData {
        project_name: snapshot.project_name.clone().into(),
        project_root: snapshot.project_root.clone().into(),
        assets_root: snapshot.assets_root.clone().into(),
        library_root: snapshot.library_root.clone().into(),
        default_scene_uri: snapshot.default_scene_uri.clone().into(),
        catalog_revision: snapshot.catalog_revision.to_string().into(),
        folder_count: snapshot.folder_count.to_string().into(),
        asset_count: snapshot.asset_count.to_string().into(),
    }
}

fn welcome_presentation(snapshot: &crate::WelcomePaneSnapshot) -> WelcomePresentation {
    WelcomePresentation {
        pane: WelcomePaneData {
            title: snapshot.title.clone().into(),
            subtitle: snapshot.subtitle.clone().into(),
            status_message: snapshot.status_message.clone().into(),
            form: NewProjectFormData {
                project_name: snapshot.form.project_name.clone().into(),
                location: snapshot.form.location.clone().into(),
                project_path_preview: snapshot.form.project_path_preview.clone().into(),
                template_label: snapshot.form.template_label.clone().into(),
                validation_message: snapshot.form.validation_message.clone().into(),
                can_create: snapshot.form.can_create,
                can_open_existing: snapshot.form.can_open_existing,
                browse_supported: snapshot.browse_supported,
            },
        },
        recent_projects: model_rc(
            snapshot
                .recent_projects
                .iter()
                .map(|recent| RecentProjectData {
                    display_name: recent.display_name.clone().into(),
                    path: recent.path.clone().into(),
                    last_opened_label: recent.last_opened_label.clone().into(),
                    status_label: recent_validation_label(recent.validation).into(),
                    invalid: recent.validation != RecentProjectValidation::Valid,
                })
                .collect(),
        ),
    }
}

fn asset_surface_presentation(snapshot: &AssetWorkspaceSnapshot) -> AssetSurfacePresentation {
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

fn asset_kind_label(kind: AssetRecordKind) -> &'static str {
    match kind {
        AssetRecordKind::Texture => "Texture",
        AssetRecordKind::Shader => "Shader",
        AssetRecordKind::Material => "Material",
        AssetRecordKind::Scene => "Scene",
        AssetRecordKind::Model => "Model",
    }
}

fn resource_state_label(state: Option<ResourceStateRecord>) -> &'static str {
    match state {
        Some(ResourceStateRecord::Pending) => "Pending",
        Some(ResourceStateRecord::Ready) => "Ready",
        Some(ResourceStateRecord::Error) => "Error",
        Some(ResourceStateRecord::Reloading) => "Reloading",
        None => "",
    }
}

fn item_has_error(state: Option<ResourceStateRecord>, diagnostics: &[String]) -> bool {
    matches!(state, Some(ResourceStateRecord::Error)) || !diagnostics.is_empty()
}

fn diagnostics_text(diagnostics: &[String]) -> String {
    diagnostics.join(" | ")
}

fn recent_validation_label(validation: RecentProjectValidation) -> &'static str {
    match validation {
        RecentProjectValidation::Valid => "",
        RecentProjectValidation::Missing => "Missing",
        RecentProjectValidation::InvalidManifest => "Manifest Error",
        RecentProjectValidation::InvalidProject => "Invalid Project",
    }
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

fn host_tab_data(page: &HostPageTabModel, active_page: &crate::MainPageId) -> TabData {
    TabData {
        id: page.id.0.clone().into(),
        slot: SharedString::default(),
        title: page.title.clone().into(),
        icon_key: host_tab_icon(page).into(),
        active: &page.id == active_page,
        closeable: page.closeable,
    }
}

fn host_tab_icon(page: &HostPageTabModel) -> &'static str {
    if page.title == "Workbench" {
        "scene"
    } else if page.title.contains("Prefab") {
        "prefab"
    } else {
        "asset-browser"
    }
}

fn document_tab_data(tab: &DocumentTabModel) -> TabData {
    TabData {
        id: tab.instance_id.0.clone().into(),
        slot: SharedString::default(),
        title: tab.title.clone().into(),
        icon_key: tab.icon_key.clone().into(),
        active: tab.active,
        closeable: tab.closeable,
    }
}

fn collect_tabs(model: &WorkbenchViewModel, slots: &[ActivityDrawerSlot]) -> Vec<TabData> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack.tabs.iter().map(move |tab| TabData {
                id: tab.instance_id.0.clone().into(),
                slot: drawer_slot_key(stack.slot).into(),
                title: tab.title.clone().into(),
                icon_key: tab.icon_key.clone().into(),
                active: tab.active,
                closeable: tab.closeable,
            })
        })
        .collect()
}

fn side_expanded(model: &WorkbenchViewModel, slots: &[ActivityDrawerSlot]) -> bool {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .any(|stack| {
            stack.visible
                && !stack.tabs.is_empty()
                && stack.mode != crate::ActivityDrawerMode::Collapsed
        })
}

fn drawer_extent(
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
    collapsed_extent: f32,
) -> f32 {
    slots
        .iter()
        .filter_map(|slot| chrome.workbench.drawers.get(slot))
        .filter(|drawer| drawer.visible)
        .map(|drawer| match drawer.mode {
            crate::ActivityDrawerMode::Collapsed => collapsed_extent,
            crate::ActivityDrawerMode::Pinned | crate::ActivityDrawerMode::AutoHide => {
                drawer.extent.max(collapsed_extent)
            }
        })
        .fold(0.0_f32, f32::max)
}

fn side_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
) -> PaneData {
    let stack = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.mode != crate::ActivityDrawerMode::Collapsed
                && stack.active_tab.is_some()
                && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.active_tab.is_some() && !stack.tabs.is_empty())
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| !stack.tabs.is_empty())
        });

    let Some(stack) = stack else {
        return blank_pane();
    };
    let tab = stack
        .tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| stack.tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        drawer_slot_key(stack.slot),
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
    )
}

fn document_pane(model: &WorkbenchViewModel, chrome: &EditorChromeSnapshot) -> PaneData {
    let tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        "",
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
    )
}

fn pane_from_tab(
    instance_id: &str,
    slot: &str,
    title: &str,
    icon_key: &str,
    kind: ViewContentKind,
    empty_state: Option<&PaneEmptyStateModel>,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
) -> PaneData {
    let (subtitle, info, show_toolbar) = pane_metadata(kind, snapshot, chrome);
    let (
        empty_title,
        empty_body,
        primary_action_label,
        primary_action_id,
        secondary_action_label,
        secondary_action_id,
        secondary_hint,
    ) = empty_state
        .map(|state| {
            (
                state.title.clone(),
                state.body.clone(),
                state
                    .primary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .primary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state.secondary_hint.clone().unwrap_or_default(),
            )
        })
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        });

    PaneData {
        id: instance_id.into(),
        slot: slot.into(),
        kind: pane_kind_key(kind).into(),
        title: title.into(),
        icon_key: icon_key.into(),
        subtitle: subtitle.into(),
        info: info.into(),
        show_empty: empty_state.is_some(),
        empty_title: empty_title.into(),
        empty_body: empty_body.into(),
        primary_action_label: primary_action_label.into(),
        primary_action_id: primary_action_id.into(),
        secondary_action_label: secondary_action_label.into(),
        secondary_action_id: secondary_action_id.into(),
        secondary_hint: secondary_hint.into(),
        show_toolbar,
    }
}

fn pane_metadata(
    kind: ViewContentKind,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
) -> (String, String, bool) {
    match kind {
        ViewContentKind::Welcome => (
            chrome.welcome.subtitle.clone(),
            chrome.welcome.status_message.clone(),
            false,
        ),
        ViewContentKind::Project => (
            if chrome.project_overview.project_name.is_empty() {
                chrome.project_path.clone()
            } else {
                chrome.project_overview.project_name.clone()
            },
            format!(
                "{} folders • {} assets",
                chrome.project_overview.folder_count, chrome.project_overview.asset_count
            ),
            false,
        ),
        ViewContentKind::Assets => (
            chrome
                .asset_activity
                .selected_folder_id
                .clone()
                .unwrap_or_else(|| "res://".to_string()),
            format!(
                "{} folders • {} assets",
                chrome.asset_activity.visible_folders.len(),
                chrome.asset_activity.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::Hierarchy => (
            format!("{} nodes", chrome.scene_entries.len()),
            "Hierarchy selection drives Scene and Inspector".to_string(),
            false,
        ),
        ViewContentKind::Inspector => (
            "Selection Inspector".to_string(),
            chrome
                .inspector
                .as_ref()
                .map(|inspector| format!("Node {}", inspector.id))
                .unwrap_or_default(),
            false,
        ),
        ViewContentKind::Scene | ViewContentKind::Game => (
            format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            String::new(),
            true,
        ),
        ViewContentKind::Console => ("Task Output".to_string(), chrome.status_line.clone(), false),
        ViewContentKind::PrefabEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Prefab Workspace".to_string()),
            "Prefab editor host slot is ready. Asset-specific tooling is still placeholder.".into(),
            false,
        ),
        ViewContentKind::AssetBrowser => (
            chrome.asset_browser.project_name.clone(),
            format!(
                "{} folders • {} assets",
                chrome.asset_browser.visible_folders.len(),
                chrome.asset_browser.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::Placeholder => (
            "Missing View".to_string(),
            "This pane was restored from layout state but the descriptor is unavailable.".into(),
            false,
        ),
    }
}

fn payload_path(snapshot: Option<&ViewTabSnapshot>) -> Option<String> {
    snapshot
        .and_then(|view| view.serializable_payload.get("path"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn action_id_from_model(action: &PaneActionModel) -> String {
    match action.binding.as_ref().map(EditorUiBinding::payload) {
        Some(EditorUiBindingPayload::MenuAction { action_id }) => action_id.clone(),
        _ => match action.label.as_str() {
            "Open Scene" => "OpenScene".to_string(),
            "Create Scene" => "CreateScene".to_string(),
            _ => String::new(),
        },
    }
}

fn pane_kind_key(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "Welcome",
        ViewContentKind::Project => "Project",
        ViewContentKind::Hierarchy => "Hierarchy",
        ViewContentKind::Inspector => "Inspector",
        ViewContentKind::Scene => "Scene",
        ViewContentKind::Game => "Game",
        ViewContentKind::Assets => "Assets",
        ViewContentKind::Console => "Console",
        ViewContentKind::PrefabEditor => "PrefabEditor",
        ViewContentKind::AssetBrowser => "AssetBrowser",
        ViewContentKind::Placeholder => "Placeholder",
    }
}

fn drawer_slot_key(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::BottomLeft => "bottom_left",
        ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

fn find_tab_snapshot<'a>(
    chrome: &'a EditorChromeSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    for drawer in chrome.workbench.drawers.values() {
        if let Some(tab) = drawer
            .tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id)
        {
            return Some(tab);
        }
    }

    for page in &chrome.workbench.main_pages {
        match page {
            MainPageSnapshot::Workbench { workspace, .. } => {
                if let Some(tab) = find_in_workspace(workspace, instance_id) {
                    return Some(tab);
                }
            }
            MainPageSnapshot::Exclusive { view, .. } if view.instance_id.0 == instance_id => {
                return Some(view)
            }
            MainPageSnapshot::Exclusive { .. } => {}
        }
    }

    for window in &chrome.workbench.floating_windows {
        if let Some(tab) = find_in_workspace(&window.workspace, instance_id) {
            return Some(tab);
        }
    }

    None
}

fn find_in_workspace<'a>(
    workspace: &'a crate::DocumentWorkspaceSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    match workspace {
        crate::DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            find_in_workspace(first, instance_id).or_else(|| find_in_workspace(second, instance_id))
        }
        crate::DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id),
    }
}

fn blank_pane() -> PaneData {
    PaneData {
        id: SharedString::default(),
        slot: SharedString::default(),
        kind: "Placeholder".into(),
        title: SharedString::default(),
        icon_key: SharedString::default(),
        subtitle: SharedString::default(),
        info: SharedString::default(),
        show_empty: false,
        empty_title: SharedString::default(),
        empty_body: SharedString::default(),
        primary_action_label: SharedString::default(),
        primary_action_id: SharedString::default(),
        secondary_action_label: SharedString::default(),
        secondary_action_id: SharedString::default(),
        secondary_hint: SharedString::default(),
        show_toolbar: false,
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
