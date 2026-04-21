use super::pane_projection::side_pane;
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    asset_surface_presentation, project_overview_data, welcome_presentation,
    AssetSurfacePresentation, WelcomePresentation,
};
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::widgets::common::{collect_tabs, document_tab_data, host_tab_data, side_expanded};

pub(crate) struct ShellPresentation {
    pub host_surface_data: HostWindowSurfaceData,
    pub welcome: WelcomePresentation,
    pub hierarchy_nodes: ModelRc<SceneNodeData>,
    pub project_overview: ProjectOverviewData,
    pub activity: AssetSurfacePresentation,
    pub browser: AssetSurfacePresentation,
    pub host_shell: HostWindowShellData,
    pub status_primary: SharedString,
    pub delete_enabled: bool,
    pub inspector_name: SharedString,
    pub inspector_parent: SharedString,
    pub inspector_x: SharedString,
    pub inspector_y: SharedString,
    pub inspector_z: SharedString,
    pub mesh_import_path: SharedString,
}

impl ShellPresentation {
    pub(crate) fn from_state(
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        preset_names: &[String],
        active_preset_name: Option<&str>,
        ui_asset_panes: &std::collections::BTreeMap<
            String,
            crate::ui::asset_editor::UiAssetEditorPanePresentation,
        >,
        animation_panes: &std::collections::BTreeMap<
            String,
            crate::ui::animation_editor::AnimationEditorPanePresentation,
        >,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
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
        let activity = asset_surface_presentation(&chrome.asset_activity);
        let browser = asset_surface_presentation(&chrome.asset_browser);
        let welcome = welcome_presentation(&chrome.welcome);
        let preset_names = model_rc(
            preset_names
                .iter()
                .cloned()
                .map(SharedString::from)
                .collect(),
        );

        Self {
            host_surface_data: HostWindowSurfaceData {
                host_tabs: model_rc(
                    model
                        .host_strip
                        .pages
                        .iter()
                        .map(|page| host_tab_data(page, &model.host_strip.active_page))
                        .collect(),
                ),
                left_tabs: model_rc(left_tabs),
                right_tabs: model_rc(right_tabs),
                bottom_tabs: model_rc(bottom_tabs),
                document_tabs: model_rc(
                    model.document_tabs.iter().map(document_tab_data).collect(),
                ),
                floating_windows: model_rc(collect_floating_windows(
                    model,
                    chrome,
                    geometry,
                    ui_asset_panes,
                    animation_panes,
                    floating_window_projection_bundle,
                )),
                left_pane: side_pane(
                    model,
                    chrome,
                    &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                    ui_asset_panes,
                    animation_panes,
                ),
                right_pane: side_pane(
                    model,
                    chrome,
                    &[
                        ActivityDrawerSlot::RightTop,
                        ActivityDrawerSlot::RightBottom,
                    ],
                    ui_asset_panes,
                    animation_panes,
                ),
                bottom_pane: side_pane(
                    model,
                    chrome,
                    &[
                        ActivityDrawerSlot::BottomLeft,
                        ActivityDrawerSlot::BottomRight,
                    ],
                    ui_asset_panes,
                    animation_panes,
                ),
                document_pane: document_pane(model, chrome, ui_asset_panes, animation_panes),
            },
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
            host_shell: HostWindowShellData {
                project_path: chrome.project_path.clone().into(),
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
                save_project_enabled: chrome.project_open,
                undo_enabled: chrome.can_undo,
                redo_enabled: chrome.can_redo,
                preset_names,
                active_preset_name: active_preset_name.unwrap_or_default().into(),
                shell_min_width_px: geometry.window_min_width,
                shell_min_height_px: geometry.window_min_height,
                native_floating_window_mode: false,
                native_floating_window_id: "".into(),
                native_window_title: "Zircon Editor".into(),
                native_window_bounds: FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
            },
            status_primary: chrome.status_line.clone().into(),
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
        }
    }
}
