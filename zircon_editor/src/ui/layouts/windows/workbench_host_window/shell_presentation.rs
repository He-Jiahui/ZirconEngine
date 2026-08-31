use std::sync::Arc;

use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{project_overview_data, welcome_presentation, WelcomePresentation};
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::widgets::common::side_expanded;
use crate::ui::workbench::startup::display_project_title;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

use crate::ui::retained_host::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;

#[derive(Clone)]
pub(crate) struct ShellPresentation {
    pub host_surface_data: HostWindowSurfaceData,
    pub retained_scene_data: Option<Arc<HostWindowSceneData>>,
    pub welcome: WelcomePresentation,
    pub project_overview: ProjectOverviewData,
    pub host_shell: HostWindowShellData,
    pub status_primary: SharedString,
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
        runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
        module_plugins: &ModulePluginsPaneViewData,
        build_export: &BuildExportPaneViewData,
        template_v2_data: &std::collections::BTreeMap<
            String,
            crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot,
        >,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
        chrome_projection_cache: &mut HostChromeProjectionCache,
    ) -> Self {
        let host_tabs = chrome_projection_cache.host_tabs(model);
        let left_tabs = chrome_projection_cache.left_tabs(model);
        let right_tabs = chrome_projection_cache.right_tabs(model);
        let bottom_tabs = chrome_projection_cache.bottom_tabs(model);
        let document_tabs = chrome_projection_cache.document_tabs(model);

        let welcome = welcome_presentation(&chrome.welcome);
        let host_shell = build_host_window_shell_data(
            model,
            chrome,
            geometry,
            preset_names,
            active_preset_name,
            chrome_projection_cache,
        );

        Self {
            host_surface_data: HostWindowSurfaceData {
                host_tabs,
                left_tabs,
                right_tabs,
                bottom_tabs,
                document_tabs,
                floating_windows: model_rc(collect_floating_windows_with_template_v2_data(
                    model,
                    chrome,
                    geometry,
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    module_plugins,
                    build_export,
                    template_v2_data,
                    floating_window_projection_bundle,
                )),
                left_pane: side_pane_with_template_v2_data(
                    model,
                    chrome,
                    &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    module_plugins,
                    build_export,
                    template_v2_data,
                ),
                right_pane: side_pane_with_template_v2_data(
                    model,
                    chrome,
                    &[
                        ActivityDrawerSlot::RightTop,
                        ActivityDrawerSlot::RightBottom,
                    ],
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    module_plugins,
                    build_export,
                    template_v2_data,
                ),
                bottom_pane: side_pane_with_template_v2_data(
                    model,
                    chrome,
                    &[ActivityDrawerSlot::Bottom],
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    module_plugins,
                    build_export,
                    template_v2_data,
                ),
                document_pane: document_pane_with_template_v2_data(
                    model,
                    chrome,
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    module_plugins,
                    build_export,
                    template_v2_data,
                ),
            },
            retained_scene_data: None,
            welcome,
            project_overview: project_overview_data(&chrome.project_overview),
            host_shell,
            status_primary: chrome.status_line.clone().into(),
            mesh_import_path: chrome.mesh_import_path.clone().into(),
        }
    }
}

pub(crate) fn build_host_window_shell_data(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    chrome_projection_cache: &mut HostChromeProjectionCache,
) -> HostWindowShellData {
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
    let bottom_expanded = side_expanded(model, &[ActivityDrawerSlot::Bottom]);
    let status_secondary = model.status_bar.secondary_text.clone().unwrap_or_default();
    let preset_names = chrome_projection_cache.preset_names(preset_names);
    let design_stack =
        crate::ui::workbench::preset::EditorUiDesignStack::material_fyrox_jetbrains_unreal();

    HostWindowShellData {
        project_path: display_project_title(&chrome.project_path).into(),
        status_secondary: status_secondary.into(),
        debug_refresh_rate: STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.into(),
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
        skin_id: design_stack.skin_id.into(),
        panel_preset_id: design_stack.panel_preset_id.into(),
        shell_preset_id: design_stack.shell_preset_id.into(),
        window_model_preset_id: design_stack.window_model_preset_id.into(),
        shell_min_width_px: geometry.window_min_width,
        shell_min_height_px: geometry.window_min_height,
        native_floating_window_mode: false,
        native_floating_window_id: "".into(),
        native_surface_tree_id: "".into(),
        native_window_title: "Zircon Editor".into(),
        native_window_bounds: FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    }
}
