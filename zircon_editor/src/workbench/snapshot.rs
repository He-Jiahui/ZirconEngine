//! UI-facing snapshots for editor data and workbench layout binding.

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;
use zircon_manager::{AssetRecordKind, ResourceStateRecord};
use zircon_graphics::GizmoAxis;
use zircon_math::UVec2;
use zircon_scene::NodeId;

#[allow(unused_imports)]
pub use super::startup::{
    NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot,
};
use super::startup::EditorSessionMode;
use crate::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout, MainPageId,
    SplitAxis, WorkbenchLayout,
};
use crate::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind,
};

#[derive(Clone, Debug)]
pub struct SceneEntry {
    pub id: NodeId,
    pub name: String,
    pub depth: usize,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
}

#[derive(Clone, Debug, Default)]
pub struct ProjectOverviewSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub library_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub folder_count: usize,
    pub asset_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetSurfaceMode {
    #[default]
    Activity,
    Explorer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetViewMode {
    #[default]
    List,
    Thumbnail,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetUtilityTab {
    #[default]
    Preview,
    References,
    Metadata,
    Plugins,
}

#[derive(Clone, Debug, Default)]
pub struct AssetFolderSnapshot {
    pub folder_id: String,
    pub parent_folder_id: Option<String>,
    pub display_name: String,
    pub recursive_asset_count: usize,
    pub depth: usize,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct AssetItemSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub kind: AssetRecordKind,
    pub preview_artifact_path: String,
    pub dirty: bool,
    pub diagnostics: Vec<String>,
    pub selected: bool,
    pub resource_state: Option<ResourceStateRecord>,
    pub resource_revision: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct AssetReferenceSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<AssetRecordKind>,
    pub known_project_asset: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AssetSelectionSnapshot {
    pub uuid: Option<String>,
    pub display_name: String,
    pub locator: String,
    pub kind: Option<AssetRecordKind>,
    pub preview_artifact_path: String,
    pub meta_path: String,
    pub adapter_key: String,
    pub diagnostics: Vec<String>,
    pub resource_state: Option<ResourceStateRecord>,
    pub resource_revision: Option<u64>,
    pub references: Vec<AssetReferenceSnapshot>,
    pub used_by: Vec<AssetReferenceSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct AssetWorkspaceSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub library_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub surface_mode: AssetSurfaceMode,
    pub view_mode: AssetViewMode,
    pub utility_tab: AssetUtilityTab,
    pub search_query: String,
    pub kind_filter: Option<AssetRecordKind>,
    pub folder_tree: Vec<AssetFolderSnapshot>,
    pub visible_folders: Vec<AssetFolderSnapshot>,
    pub visible_assets: Vec<AssetItemSnapshot>,
    pub selected_folder_id: Option<String>,
    pub selected_asset_uuid: Option<String>,
    pub selection: AssetSelectionSnapshot,
}

#[derive(Clone, Debug)]
pub struct EditorDataSnapshot {
    pub scene_entries: Vec<SceneEntry>,
    pub inspector: Option<InspectorSnapshot>,
    pub status_line: String,
    pub hovered_axis: Option<GizmoAxis>,
    pub viewport_size: UVec2,
    pub mesh_import_path: String,
    pub project_overview: ProjectOverviewSnapshot,
    pub asset_activity: AssetWorkspaceSnapshot,
    pub asset_browser: AssetWorkspaceSnapshot,
    pub project_path: String,
    pub session_mode: EditorSessionMode,
    pub welcome: WelcomePaneSnapshot,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewContentKind {
    Welcome,
    Project,
    Hierarchy,
    Inspector,
    Scene,
    Game,
    Assets,
    Console,
    PrefabEditor,
    AssetBrowser,
    Placeholder,
}

#[derive(Clone, Debug)]
pub struct ViewTabSnapshot {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub icon_key: String,
    pub kind: ViewKind,
    pub host: ViewHost,
    pub serializable_payload: Value,
    pub dirty: bool,
    pub content_kind: ViewContentKind,
    pub placeholder: bool,
}

#[derive(Clone, Debug)]
pub enum DocumentWorkspaceSnapshot {
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentWorkspaceSnapshot>,
        second: Box<DocumentWorkspaceSnapshot>,
    },
    Tabs {
        tabs: Vec<ViewTabSnapshot>,
        active_tab: Option<ViewInstanceId>,
    },
}

#[derive(Clone, Debug)]
pub struct ActivityDrawerSnapshot {
    pub slot: ActivityDrawerSlot,
    pub tabs: Vec<ViewTabSnapshot>,
    pub active_tab: Option<ViewInstanceId>,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub enum MainPageSnapshot {
    Workbench {
        id: MainPageId,
        title: String,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}

#[derive(Clone, Debug)]
pub struct FloatingWindowSnapshot {
    pub window_id: MainPageId,
    pub title: String,
    pub workspace: DocumentWorkspaceSnapshot,
    pub focused_view: Option<ViewInstanceId>,
}

#[derive(Clone, Debug)]
pub struct WorkbenchSnapshot {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainPageSnapshot>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    pub floating_windows: Vec<FloatingWindowSnapshot>,
}

#[derive(Clone, Debug)]
pub struct EditorChromeSnapshot {
    pub workbench: WorkbenchSnapshot,
    pub scene_entries: Vec<SceneEntry>,
    pub inspector: Option<InspectorSnapshot>,
    pub status_line: String,
    pub hovered_axis: Option<GizmoAxis>,
    pub viewport_size: UVec2,
    pub mesh_import_path: String,
    pub project_overview: ProjectOverviewSnapshot,
    pub asset_activity: AssetWorkspaceSnapshot,
    pub asset_browser: AssetWorkspaceSnapshot,
    pub project_path: String,
    pub session_mode: EditorSessionMode,
    pub welcome: WelcomePaneSnapshot,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

impl EditorChromeSnapshot {
    pub fn build(
        data: EditorDataSnapshot,
        layout: &WorkbenchLayout,
        instances: Vec<ViewInstance>,
        descriptors: Vec<ViewDescriptor>,
    ) -> Self {
        let instances_by_id: HashMap<_, _> = instances
            .into_iter()
            .map(|instance| (instance.instance_id.clone(), instance))
            .collect();
        let descriptors_by_id: HashMap<_, _> = descriptors
            .into_iter()
            .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor))
            .collect();

        let drawers = layout
            .drawers
            .iter()
            .map(|(slot, drawer)| {
                (
                    *slot,
                    ActivityDrawerSnapshot {
                        slot: *slot,
                        tabs: drawer
                            .tab_stack
                            .tabs
                            .iter()
                            .map(|instance_id| {
                                resolve_view_tab(instance_id, &instances_by_id, &descriptors_by_id)
                            })
                            .collect(),
                        active_tab: drawer.tab_stack.active_tab.clone(),
                        active_view: drawer.active_view.clone(),
                        mode: drawer.mode,
                        extent: drawer.extent,
                        visible: drawer.visible,
                    },
                )
            })
            .collect();

        let main_pages = layout
            .main_pages
            .iter()
            .map(|page| match page {
                MainHostPageLayout::WorkbenchPage {
                    id,
                    title,
                    document_workspace,
                } => MainPageSnapshot::Workbench {
                    id: id.clone(),
                    title: title.clone(),
                    workspace: resolve_document_workspace(
                        document_workspace,
                        &instances_by_id,
                        &descriptors_by_id,
                    ),
                },
                MainHostPageLayout::ExclusiveActivityWindowPage {
                    id,
                    title,
                    window_instance,
                } => MainPageSnapshot::Exclusive {
                    id: id.clone(),
                    title: title.clone(),
                    view: resolve_view_tab(window_instance, &instances_by_id, &descriptors_by_id),
                },
            })
            .collect();

        let floating_windows = layout
            .floating_windows
            .iter()
            .map(|window| FloatingWindowSnapshot {
                window_id: window.window_id.clone(),
                title: window.title.clone(),
                workspace: resolve_document_workspace(
                    &window.workspace,
                    &instances_by_id,
                    &descriptors_by_id,
                ),
                focused_view: window.focused_view.clone(),
            })
            .collect();

        Self {
            workbench: WorkbenchSnapshot {
                active_main_page: layout.active_main_page.clone(),
                main_pages,
                drawers,
                floating_windows,
            },
            scene_entries: data.scene_entries,
            inspector: data.inspector,
            status_line: data.status_line,
            hovered_axis: data.hovered_axis,
            viewport_size: data.viewport_size,
            mesh_import_path: data.mesh_import_path,
            project_overview: data.project_overview,
            asset_activity: data.asset_activity,
            asset_browser: data.asset_browser,
            project_path: data.project_path,
            session_mode: data.session_mode,
            welcome: data.welcome,
            project_open: data.project_open,
            can_undo: data.can_undo,
            can_redo: data.can_redo,
        }
    }
}

fn resolve_document_workspace(
    node: &DocumentNode,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> DocumentWorkspaceSnapshot {
    match node {
        DocumentNode::SplitNode {
            axis,
            ratio,
            first,
            second,
        } => DocumentWorkspaceSnapshot::Split {
            axis: *axis,
            ratio: *ratio,
            first: Box::new(resolve_document_workspace(first, instances, descriptors)),
            second: Box::new(resolve_document_workspace(second, instances, descriptors)),
        },
        DocumentNode::Tabs(stack) => DocumentWorkspaceSnapshot::Tabs {
            tabs: stack
                .tabs
                .iter()
                .map(|instance_id| resolve_view_tab(instance_id, instances, descriptors))
                .collect(),
            active_tab: stack.active_tab.clone(),
        },
    }
}

fn resolve_view_tab(
    instance_id: &ViewInstanceId,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> ViewTabSnapshot {
    let Some(instance) = instances.get(instance_id) else {
        return placeholder_view(
            instance_id.clone(),
            ViewDescriptorId::new("missing.instance"),
            format!("Missing View {}", instance_id.0),
        );
    };

    let Some(descriptor) = descriptors.get(&instance.descriptor_id) else {
        return placeholder_view(
            instance.instance_id.clone(),
            instance.descriptor_id.clone(),
            format!("Missing Descriptor {}", instance.title),
        );
    };

    ViewTabSnapshot {
        instance_id: instance.instance_id.clone(),
        descriptor_id: descriptor.descriptor_id.clone(),
        title: instance.title.clone(),
        icon_key: descriptor.icon_key.clone(),
        kind: descriptor.kind,
        host: instance.host.clone(),
        serializable_payload: instance.serializable_payload.clone(),
        dirty: instance.dirty,
        content_kind: descriptor_content_kind(&descriptor.descriptor_id),
        placeholder: false,
    }
}

fn placeholder_view(
    instance_id: ViewInstanceId,
    descriptor_id: ViewDescriptorId,
    title: String,
) -> ViewTabSnapshot {
    ViewTabSnapshot {
        instance_id,
        descriptor_id,
        title,
        icon_key: "missing".to_string(),
        kind: ViewKind::ActivityView,
        host: ViewHost::Document(MainPageId::workbench(), vec![]),
        serializable_payload: Value::Null,
        dirty: false,
        content_kind: ViewContentKind::Placeholder,
        placeholder: true,
    }
}

fn descriptor_content_kind(descriptor_id: &ViewDescriptorId) -> ViewContentKind {
    match descriptor_id.0.as_str() {
        "editor.welcome" => ViewContentKind::Welcome,
        "editor.project" => ViewContentKind::Project,
        "editor.hierarchy" => ViewContentKind::Hierarchy,
        "editor.inspector" => ViewContentKind::Inspector,
        "editor.scene" => ViewContentKind::Scene,
        "editor.game" => ViewContentKind::Game,
        "editor.assets" => ViewContentKind::Assets,
        "editor.console" => ViewContentKind::Console,
        "editor.prefab" => ViewContentKind::PrefabEditor,
        "editor.asset_browser" => ViewContentKind::AssetBrowser,
        _ => ViewContentKind::Placeholder,
    }
}
