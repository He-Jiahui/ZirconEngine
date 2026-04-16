use super::asset_surface_presentation::{asset_surface_presentation, AssetSurfacePresentation};
use super::floating_windows::collect_floating_windows;
use super::model_rc::model_rc;
use super::pane_projection::{document_pane, side_pane};
use super::project_overview::project_overview_data;
use super::welcome_presentation::{welcome_presentation, WelcomePresentation};
use super::workbench_tabs::{
    collect_tabs, document_tab_data, drawer_extent, host_tab_data, side_expanded,
};
use super::*;

pub(super) struct ShellPresentation {
    pub host_tabs: ModelRc<TabData>,
    pub breadcrumbs: ModelRc<BreadcrumbData>,
    pub left_tabs: ModelRc<TabData>,
    pub right_tabs: ModelRc<TabData>,
    pub bottom_tabs: ModelRc<TabData>,
    pub document_tabs: ModelRc<TabData>,
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub left_pane: PaneData,
    pub right_pane: PaneData,
    pub bottom_pane: PaneData,
    pub document_pane: PaneData,
    pub welcome: WelcomePresentation,
    pub hierarchy_nodes: ModelRc<SceneNodeData>,
    pub project_overview: ProjectOverviewData,
    pub activity: AssetSurfacePresentation,
    pub browser: AssetSurfacePresentation,
    pub project_path: SharedString,
    pub status_primary: SharedString,
    pub status_secondary: SharedString,
    pub viewport_label: SharedString,
    pub drawers_visible: bool,
    pub left_expanded: bool,
    pub right_expanded: bool,
    pub bottom_expanded: bool,
    pub left_drawer_extent: f32,
    pub right_drawer_extent: f32,
    pub bottom_drawer_extent: f32,
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub delete_enabled: bool,
    pub inspector_name: SharedString,
    pub inspector_parent: SharedString,
    pub inspector_x: SharedString,
    pub inspector_y: SharedString,
    pub inspector_z: SharedString,
    pub mesh_import_path: SharedString,
    pub preset_names: ModelRc<SharedString>,
    pub active_preset_name: SharedString,
}

impl ShellPresentation {
    pub(super) fn from_state(
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        preset_names: &[String],
        active_preset_name: Option<&str>,
        ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
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
            floating_windows: model_rc(collect_floating_windows(
                model,
                chrome,
                geometry,
                ui_asset_panes,
            )),
            left_pane: side_pane(
                model,
                chrome,
                &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                ui_asset_panes,
            ),
            right_pane: side_pane(
                model,
                chrome,
                &[
                    ActivityDrawerSlot::RightTop,
                    ActivityDrawerSlot::RightBottom,
                ],
                ui_asset_panes,
            ),
            bottom_pane: side_pane(
                model,
                chrome,
                &[
                    ActivityDrawerSlot::BottomLeft,
                    ActivityDrawerSlot::BottomRight,
                ],
                ui_asset_panes,
            ),
            document_pane: document_pane(model, chrome, ui_asset_panes),
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
