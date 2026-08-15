use super::pane_data_conversion;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views as view_data;
use crate::ui::layouts::windows::workbench_host_window::{
    self as host_window, build_host_scene_data_with_cache, build_native_floating_surface_data,
    frame_rect, HostChromeProjectionCache, ShellPresentation,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{self as host_contract, HostWindowPresentationData, UiHostWindow};
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostProjection};
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::floating_pane_geometry::floating_pane_content_size;
use super::root_template_overlay::to_host_contract_root_template_overlay_nodes_at_scale;
use super::shell_content_presentation::patch_shell_content_presentation;
use super::template_node_conversion::to_host_contract_template_nodes;
use super::workbench_window_projection::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;

#[path = "apply_presentation/pane_conversion.rs"]
mod pane_conversion;
#[path = "apply_presentation/scene_conversion.rs"]
mod scene_conversion;

use pane_conversion::to_host_contract_pane;
#[cfg(test)]
pub(in crate::ui::retained_host::ui) use scene_conversion::to_host_contract_host_scene_data;
pub(super) use scene_conversion::{
    to_host_contract_bottom_dock, to_host_contract_host_shell, to_host_contract_host_window_layout,
    to_host_contract_side_dock,
};
use scene_conversion::{
    to_host_contract_chrome_tab, to_host_contract_host_scene_data_with_runtime,
    to_host_contract_native_floating_surface_data_with_runtime,
};

pub(crate) fn apply_presentation(
    ui: &UiHostWindow,
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
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    module_plugins: &host_window::ModulePluginsPaneViewData,
    build_export: &host_window::BuildExportPaneViewData,
    root_template_projection: Option<&RetainedUiHostProjection>,
    workbench_window_projection: Option<&RetainedUiHostProjection>,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) {
    let template_v2_data = std::collections::BTreeMap::new();
    let mut chrome_projection_cache = HostChromeProjectionCache::default();
    apply_presentation_with_template_v2_data(
        ui,
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        module_plugins,
        build_export,
        &template_v2_data,
        root_template_projection,
        workbench_window_projection,
        componentized_workbench_layout_frames,
        floating_window_projection_bundle,
        component_showcase_runtime,
        1.0,
        "",
        &mut chrome_projection_cache,
        false,
    );
}

pub(crate) fn apply_presentation_with_template_v2_data(
    ui: &UiHostWindow,
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
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    module_plugins: &host_window::ModulePluginsPaneViewData,
    build_export: &host_window::BuildExportPaneViewData,
    template_v2_data: &std::collections::BTreeMap<
        String,
        crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot,
    >,
    root_template_projection: Option<&RetainedUiHostProjection>,
    workbench_window_projection: Option<&RetainedUiHostProjection>,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    template_scale_factor: f32,
    hierarchy_filter_query: &str,
    chrome_projection_cache: &mut HostChromeProjectionCache,
    shell_content_only: bool,
) {
    let presentation = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_shell_presentation_from_state"
        );
        ShellPresentation::from_state_with_template_v2_data_and_cache(
            model,
            chrome,
            geometry,
            preset_names,
            active_preset_name,
            ui_asset_panes,
            animation_panes,
            runtime_diagnostics,
            module_plugins,
            build_export,
            template_v2_data,
            floating_window_projection_bundle,
            chrome_projection_cache,
        )
    };
    let pane_surface_host = ui.global::<host_contract::PaneSurfaceHostContext>();

    let host_layout = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_host_window_layout");
        host_window_layout(componentized_workbench_layout_frames)
    };
    if shell_content_only
        && patch_shell_content_presentation(
            ui,
            &presentation,
            &host_layout,
            chrome,
            component_showcase_runtime,
            hierarchy_filter_query,
        )
    {
        return;
    }
    let host_scene_data = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_build_host_scene_data");
        build_host_scene_data_with_cache(
            &model.menu_bar,
            &presentation.host_surface_data,
            &presentation.host_shell,
            &host_layout,
            &presentation.status_primary,
            chrome.inspector.is_some(),
            &chrome.project_overview,
            chrome,
            chrome_projection_cache,
        )
    };
    let host_welcome_pane = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_build_welcome_pane");
        let welcome_pane = project_welcome_pane(&presentation.welcome.pane, &host_scene_data);
        to_host_contract_welcome_pane(&welcome_pane, &presentation.welcome.recent_projects)
    };
    let native_floating_surface_data = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_build_native_floating_surface_data"
        );
        build_native_floating_surface_data(
            &presentation.host_surface_data,
            &presentation.host_shell,
            &chrome.project_overview,
            chrome,
        )
    };
    let current_generation = ui.get_host_presentation_generation();
    let current_structure = current_generation.structure();
    let host_scene_data = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_convert_host_scene_data");
        to_host_contract_host_scene_data_with_runtime(
            &host_scene_data,
            component_showcase_runtime,
            Some(&presentation.welcome),
            hierarchy_filter_query,
        )
    };
    let native_floating_surface_data = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_convert_native_floating_surface_data"
        );
        to_host_contract_native_floating_surface_data_with_runtime(
            &native_floating_surface_data,
            component_showcase_runtime,
            Some(&presentation.welcome),
            hierarchy_filter_query,
        )
    };
    let host_shell = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_convert_host_shell");
        to_host_contract_host_shell(&presentation.host_shell)
    };
    let host_layout = {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_convert_host_layout");
        to_host_contract_host_window_layout(&host_layout)
    };
    let workbench_window_nodes =
        to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            workbench_window_projection,
            Some(&current_structure.workbench_window_nodes),
            componentized_workbench_layout_frames.mount_frame,
            template_scale_factor,
        );
    let host_presentation = HostWindowPresentationData {
        host_scene_data,
        native_floating_surface_data,
        host_shell,
        host_layout,
        close_prompt: current_structure.close_prompt.clone(),
        root_template_nodes: to_host_contract_root_template_overlay_nodes_at_scale(
            root_template_projection,
            template_scale_factor,
        ),
        workbench_window_nodes,
        ..HostWindowPresentationData::default()
    };
    drop(current_generation);
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_set_host_presentation");
        ui.set_host_presentation(host_presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "apply_set_tail_globals");
        pane_surface_host.set_welcome_pane(host_welcome_pane);
    }
}

pub(super) fn host_window_layout(
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> host_window::HostWindowLayoutData {
    let center_band_frame = componentized_workbench_layout_frames
        .center_band_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let left_region_frame = componentized_workbench_layout_frames
        .left_region_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let document_region_frame = componentized_workbench_layout_frames
        .document_region_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let right_region_frame = componentized_workbench_layout_frames
        .right_region_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let bottom_region_frame = componentized_workbench_layout_frames
        .bottom_region_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let viewport_content_frame = componentized_workbench_layout_frames
        .viewport_content_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let status_bar_frame = componentized_workbench_layout_frames
        .status_bar_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let left_splitter_frame = componentized_workbench_layout_frames
        .left_resize_splitter_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let right_splitter_frame = componentized_workbench_layout_frames
        .right_resize_splitter_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let bottom_splitter_frame = componentized_workbench_layout_frames
        .bottom_resize_splitter_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();

    host_window::HostWindowLayoutData {
        center_band_frame: frame_rect(center_band_frame),
        status_bar_frame: frame_rect(status_bar_frame),
        left_region_frame: frame_rect(left_region_frame),
        document_region_frame: frame_rect(document_region_frame),
        right_region_frame: frame_rect(right_region_frame),
        bottom_region_frame: frame_rect(bottom_region_frame),
        left_splitter_frame: frame_rect(left_splitter_frame),
        right_splitter_frame: frame_rect(right_splitter_frame),
        bottom_splitter_frame: frame_rect(bottom_splitter_frame),
        viewport_content_frame: frame_rect(viewport_content_frame),
    }
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(T) -> U,
{
    model_rc(
        (0..model.row_count())
            .filter_map(|row| model.row_data(row))
            .map(&mut map)
            .collect(),
    )
}

fn to_host_contract_frame_rect(frame: &host_window::FrameRect) -> host_contract::FrameRect {
    host_contract::FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_host_contract_tab_data(tab: host_window::TabData) -> host_contract::TabData {
    host_contract::TabData {
        id: tab.id,
        slot: tab.slot,
        title: tab.title,
        icon_key: tab.icon_key,
        active: tab.active,
        closeable: tab.closeable,
    }
}

fn to_host_contract_tabs(tabs: &ModelRc<host_window::TabData>) -> ModelRc<host_contract::TabData> {
    map_model_rc(tabs, to_host_contract_tab_data)
}

fn to_host_contract_floating_window_data(
    window: host_window::FloatingWindowData,
    header_height_px: f32,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
) -> host_contract::FloatingWindowData {
    let content_size = floating_pane_content_size(
        window.frame.width,
        window.frame.height,
        window.header_frame.height,
        header_height_px,
    );
    let pane_size = host_window::PaneContentSize::new(content_size.width, content_size.height);
    host_contract::FloatingWindowData {
        window_id: window.window_id,
        title: window.title,
        frame: to_host_contract_frame_rect(&window.frame),
        header_nodes: to_host_contract_template_nodes(&window.header_nodes),
        header_frame: to_host_contract_frame_rect(&window.header_frame),
        tab_frames: map_model_rc(&window.tab_frames, to_host_contract_chrome_tab),
        target_group: window.target_group,
        left_edge_target_group: window.left_edge_target_group,
        right_edge_target_group: window.right_edge_target_group,
        top_edge_target_group: window.top_edge_target_group,
        bottom_edge_target_group: window.bottom_edge_target_group,
        focus_target_id: window.focus_target_id,
        tabs: to_host_contract_tabs(&window.tabs),
        active_pane: to_host_contract_pane(
            window.active_pane,
            pane_size,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
        ),
    }
}

fn to_host_contract_floating_windows(
    windows: &ModelRc<host_window::FloatingWindowData>,
    header_height_px: f32,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
) -> ModelRc<host_contract::FloatingWindowData> {
    map_model_rc(windows, |window| {
        to_host_contract_floating_window_data(
            window,
            header_height_px,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
        )
    })
}

fn to_host_contract_new_project_form(
    form: &view_data::NewProjectFormData,
) -> host_contract::NewProjectFormData {
    host_contract::NewProjectFormData {
        project_name: form.project_name.clone(),
        location: form.location.clone(),
        project_path_preview: form.project_path_preview.clone(),
        template_label: form.template_label.clone(),
        validation_message: form.validation_message.clone(),
        can_create: form.can_create,
        can_open_existing: form.can_open_existing,
        browse_supported: form.browse_supported,
    }
}

fn to_host_contract_welcome_pane(
    pane: &view_data::WelcomePaneData,
    recent_projects: &ModelRc<view_data::RecentProjectData>,
) -> host_contract::WelcomePaneData {
    let nodes = welcome_nodes_with_native_dispatch(
        to_host_contract_template_nodes(&pane.nodes),
        &pane.form,
    );
    host_contract::WelcomePaneData {
        nodes,
        title: pane.title.clone(),
        subtitle: pane.subtitle.clone(),
        status_message: pane.status_message.clone(),
        form: to_host_contract_new_project_form(&pane.form),
        recent_projects: to_host_contract_recent_projects(recent_projects),
    }
}

fn welcome_nodes_with_native_dispatch(
    nodes: ModelRc<host_contract::TemplatePaneNodeData>,
    form: &view_data::NewProjectFormData,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    model_rc(
        (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .map(|mut node| {
                match node.control_id.as_str() {
                    "WelcomeProjectNameField" => {
                        node.component_role = "input-field".into();
                        node.dispatch_kind = "welcome_text".into();
                        node.action_id = "welcome.project.name.edit".into();
                        node.edit_action_id = "welcome.project.name.edit".into();
                        node.value_text = form.project_name.clone();
                        if node.text.is_empty() {
                            node.text = form.project_name.clone();
                        }
                    }
                    "WelcomeLocationField" => {
                        node.component_role = "input-field".into();
                        node.dispatch_kind = "welcome_text".into();
                        node.action_id = "welcome.project.location.edit".into();
                        node.edit_action_id = "welcome.project.location.edit".into();
                        node.value_text = form.location.clone();
                        if node.text.is_empty() {
                            node.text = form.location.clone();
                        }
                    }
                    "WelcomeCreateProjectButton" => {
                        node.dispatch_kind = "welcome".into();
                        node.action_id = "welcome.project.create".into();
                        node.disabled = !form.can_create;
                    }
                    "WelcomeOpenExistingButton" => {
                        node.dispatch_kind = "welcome".into();
                        node.action_id = "welcome.project.open_existing".into();
                        node.disabled = !form.can_open_existing;
                    }
                    _ => {}
                }
                node
            })
            .collect(),
    )
}

fn project_welcome_pane(
    pane: &view_data::WelcomePaneData,
    scene: &host_window::HostWindowSceneData,
) -> view_data::WelcomePaneData {
    let mut pane = pane.clone();
    if let Some(size) = resolve_visible_welcome_pane_size(scene) {
        pane.nodes = view_data::welcome_pane_nodes(size);
    }
    pane
}

fn project_welcome_pane_for_size(
    pane: &view_data::WelcomePaneData,
    size: host_window::PaneContentSize,
) -> view_data::WelcomePaneData {
    let mut pane = pane.clone();
    pane.nodes =
        view_data::welcome_pane_nodes(UiSize::new(size.width.max(1.0), size.height.max(1.0)));
    pane
}

fn resolve_visible_welcome_pane_size(scene: &host_window::HostWindowSceneData) -> Option<UiSize> {
    if scene.document_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.document_dock.region_frame.width.max(0.0),
            dock_content_height(
                scene.document_dock.region_frame.height,
                scene.document_dock.header_height_px,
            ),
        ));
    }

    if scene.left_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.left_dock.panel_width_px.max(0.0),
            dock_content_height(
                scene.left_dock.region_frame.height,
                scene.left_dock.panel_header_height_px,
            ),
        ));
    }

    if scene.right_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.right_dock.panel_width_px.max(0.0),
            dock_content_height(
                scene.right_dock.region_frame.height,
                scene.right_dock.panel_header_height_px,
            ),
        ));
    }

    if scene.bottom_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.bottom_dock.region_frame.width.max(0.0),
            dock_content_height(
                scene.bottom_dock.region_frame.height,
                scene.bottom_dock.header_height_px,
            ),
        ));
    }

    scene
        .floating_layer
        .floating_windows
        .iter()
        .find_map(|window| {
            (window.active_pane.kind.as_str() == "Welcome").then(|| {
                UiSize::new(
                    window.frame.width.max(0.0),
                    (window.frame.height - scene.floating_layer.header_height_px).max(0.0),
                )
            })
        })
}

fn dock_content_height(region_height: f32, header_height: f32) -> f32 {
    (region_height - header_height - 1.0).max(0.0)
}

fn to_host_contract_recent_project(
    data: view_data::RecentProjectData,
) -> host_contract::RecentProjectData {
    host_contract::RecentProjectData {
        display_name: data.display_name,
        path: data.path,
        last_opened_label: data.last_opened_label,
        status_label: data.status_label,
        invalid: data.invalid,
    }
}

fn to_host_contract_recent_projects(
    data: &ModelRc<view_data::RecentProjectData>,
) -> ModelRc<host_contract::RecentProjectData> {
    map_model_rc(data, to_host_contract_recent_project)
}

pub(crate) fn to_host_contract_scene_viewport_chrome(
    data: &view_data::SceneViewportChromeData,
) -> host_contract::SceneViewportChromeData {
    host_contract::SceneViewportChromeData {
        mode: data.mode.clone(),
        transform_space: data.transform_space.clone(),
        projection_mode: data.projection_mode.clone(),
        view_orientation: data.view_orientation.clone(),
        display_mode: data.display_mode.clone(),
        grid_mode: data.grid_mode.clone(),
        gizmos_enabled: data.gizmos_enabled,
        preview_lighting: data.preview_lighting,
        preview_skybox: data.preview_skybox,
        translate_snap: data.translate_snap,
        rotate_snap_deg: data.rotate_snap_deg,
        scale_snap: data.scale_snap,
        translate_snap_label: data.translate_snap_label.clone(),
        rotate_snap_label: data.rotate_snap_label.clone(),
        scale_snap_label: data.scale_snap_label.clone(),
        toolbar_surface_frame: None,
    }
}

fn to_host_contract_animation_editor_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::AnimationEditorPaneData {
    runtime.map_or_else(
        || pane_data_conversion::to_host_contract_animation_editor_pane_from_host_pane(data, pane_size),
        |runtime| {
            pane_data_conversion::to_host_contract_animation_editor_pane_from_host_pane_with_runtime(
                data, pane_size, runtime,
            )
        },
    )
}

fn to_host_contract_assets_activity_pane(
    data: host_window::AssetsActivityPaneViewData,
) -> host_contract::AssetsActivityPaneData {
    pane_data_conversion::to_host_contract_assets_activity_pane(data)
}

fn to_host_contract_hierarchy_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
    hierarchy_filter_query: &str,
) -> host_contract::HierarchyPaneData {
    runtime.map_or_else(
        || {
            pane_data_conversion::to_host_contract_hierarchy_pane_from_host_pane_with_query(
                data,
                pane_size,
                hierarchy_filter_query,
            )
        },
        |runtime| {
            pane_data_conversion::to_host_contract_hierarchy_pane_from_host_pane_with_runtime(
                data,
                pane_size,
                runtime,
                hierarchy_filter_query,
            )
        },
    )
}

fn to_host_contract_inspector_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::InspectorPaneData {
    runtime.map_or_else(
        || pane_data_conversion::to_host_contract_inspector_pane_from_host_pane(data, pane_size),
        |runtime| {
            pane_data_conversion::to_host_contract_inspector_pane_from_host_pane_with_runtime(
                data, pane_size, runtime,
            )
        },
    )
}

fn to_host_contract_console_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::ConsolePaneData {
    runtime.map_or_else(
        || pane_data_conversion::to_host_contract_console_pane_from_host_pane(data, pane_size),
        |runtime| {
            pane_data_conversion::to_host_contract_console_pane_from_host_pane_with_runtime(
                data, pane_size, runtime,
            )
        },
    )
}

fn to_host_contract_project_overview_pane(
    data: host_window::ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    pane_data_conversion::to_host_contract_project_overview_pane(data)
}

fn to_host_contract_module_plugins_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::ModulePluginsPaneData {
    pane_data_conversion::to_host_contract_module_plugins_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_build_export_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::BuildExportPaneData {
    pane_data_conversion::to_host_contract_build_export_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_generated_bottom_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::GeneratedBottomPaneData {
    pane_data_conversion::to_host_contract_generated_bottom_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_runtime_diagnostics_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::RuntimeDiagnosticsPaneData {
    pane_data_conversion::to_host_contract_runtime_diagnostics_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_performance_timeline_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::PerformanceTimelinePaneData {
    pane_data_conversion::to_host_contract_performance_timeline_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_ui_asset_pane(
    data: crate::ui::asset_editor::UiAssetEditorPanePresentation,
    instance_id: &str,
) -> host_contract::UiAssetEditorPaneData {
    pane_data_conversion::to_host_contract_ui_asset_pane(data, instance_id)
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn visible_welcome_size_borrows_floating_window_rows() {
        let source = include_str!("apply_presentation.rs");
        let function = source
            .split("fn resolve_visible_welcome_pane_size")
            .nth(1)
            .and_then(|body| body.split("fn dock_content_height").next())
            .expect("welcome size implementation");

        assert!(function.contains(".floating_windows"));
        assert!(function.contains(".iter()"));
        assert!(!function.contains("row_data"));
    }

    #[test]
    fn apply_presentation_does_not_build_discarded_pane_globals() {
        let source = include_str!("apply_presentation.rs");
        let function = source
            .split("pub(crate) fn apply_presentation")
            .nth(1)
            .and_then(|body| body.split("fn host_window_layout").next())
            .expect("apply presentation implementation");

        assert!(!function.contains("apply_pane_surface_globals"));
        assert!(!function.contains("set_activity_asset_"));
        assert!(!function.contains("set_browser_asset_"));
        assert!(!function.contains("set_recent_projects"));
        assert!(!function.contains("set_project_overview"));
    }
}
