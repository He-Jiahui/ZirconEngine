use std::collections::BTreeMap;

use crate::core::commands::EditorCommandRegistry;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::{
    build_host_scene_data_with_cache, BuildExportPaneViewData, FrameRect, HostWindowLayoutData,
    ModulePluginsPaneViewData, ShellPresentation,
};
use super::{
    current_menu_label_slot_metrics, HostChromeProjectionCache, MenuChromeProjectionInput,
    MenuLabelSlotMetrics,
};

#[test]
fn independent_shell_rebuilds_retain_stable_chrome_models_and_track_slot_metrics() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let commands = EditorCommandRegistry::default_workbench();
    let first_model = WorkbenchViewModel::build(&commands, &chrome);
    let second_model = WorkbenchViewModel::build(&commands, &chrome);
    let geometry = WorkbenchShellGeometry::default();
    let presets = vec!["rider".to_string(), "compact".to_string()];
    let ui_asset_panes = BTreeMap::<String, UiAssetEditorPanePresentation>::new();
    let animation_panes = BTreeMap::<String, AnimationEditorPanePresentation>::new();
    let template_v2_data = BTreeMap::new();
    let floating_windows = FloatingWindowProjectionBundle::default();
    let mut cache = HostChromeProjectionCache::default();

    let first = ShellPresentation::from_state(
        &first_model,
        &chrome,
        &geometry,
        &presets,
        Some("rider"),
        &ui_asset_panes,
        &animation_panes,
        None,
        &ModulePluginsPaneViewData::default(),
        &BuildExportPaneViewData::default(),
        &template_v2_data,
        &floating_windows,
        &mut cache,
    );
    let first_scene = build_host_scene_data_with_cache(
        &first_model.menu_bar,
        &first.host_surface_data,
        &first.host_shell,
        &host_layout_fixture(),
        &first.status_primary,
        chrome.inspector.is_some(),
        &chrome.project_overview,
        &chrome,
        &mut cache,
    );

    let second = ShellPresentation::from_state(
        &second_model,
        &chrome,
        &geometry,
        &presets,
        Some("rider"),
        &ui_asset_panes,
        &animation_panes,
        None,
        &ModulePluginsPaneViewData::default(),
        &BuildExportPaneViewData::default(),
        &template_v2_data,
        &floating_windows,
        &mut cache,
    );
    let second_scene = build_host_scene_data_with_cache(
        &second_model.menu_bar,
        &second.host_surface_data,
        &second.host_shell,
        &host_layout_fixture(),
        &second.status_primary,
        chrome.inspector.is_some(),
        &chrome.project_overview,
        &chrome,
        &mut cache,
    );

    assert!(first
        .host_surface_data
        .host_tabs
        .shares_values_with(&second.host_surface_data.host_tabs));
    assert!(first
        .host_surface_data
        .left_tabs
        .shares_values_with(&second.host_surface_data.left_tabs));
    assert!(first
        .host_surface_data
        .document_tabs
        .shares_values_with(&second.host_surface_data.document_tabs));
    assert!(first_scene
        .page_chrome
        .template_nodes
        .shares_values_with(&second_scene.page_chrome.template_nodes));
    assert!(first_scene
        .left_dock
        .header_nodes
        .shares_values_with(&second_scene.left_dock.header_nodes));
    assert!(first_scene
        .left_dock
        .rail_nodes
        .shares_values_with(&second_scene.left_dock.rail_nodes));
    assert!(first_scene
        .document_dock
        .header_nodes
        .shares_values_with(&second_scene.document_dock.header_nodes));
    assert!(first_scene
        .menu_chrome
        .menus
        .shares_values_with(&second_scene.menu_chrome.menus));
    assert!(first_scene
        .menu_chrome
        .template_nodes
        .shares_values_with(&second_scene.menu_chrome.template_nodes));

    let slot_metrics = current_menu_label_slot_metrics();
    let cache_input = MenuChromeProjectionInput::new(
        &first.host_shell,
        chrome.inspector.is_some(),
        &first_scene.metrics,
        &first_scene.menu_chrome.resolved_preset_name,
        1280.0,
        slot_metrics,
    );
    let changed_slot_metrics = MenuLabelSlotMetrics {
        font_size: slot_metrics.font_size * 1.25,
        ..slot_metrics
    };
    assert!(cache_input.matches(
        &first.host_shell,
        chrome.inspector.is_some(),
        &first_scene.metrics,
        &first_scene.menu_chrome.resolved_preset_name,
        1280.0,
        slot_metrics,
    ));
    assert!(!cache_input.matches(
        &first.host_shell,
        chrome.inspector.is_some(),
        &first_scene.metrics,
        &first_scene.menu_chrome.resolved_preset_name,
        1280.0,
        changed_slot_metrics,
    ));
}

fn host_layout_fixture() -> HostWindowLayoutData {
    HostWindowLayoutData {
        authoritative: true,
        center_band_frame: host_layout_frame_fixture(0.0, 64.0, 1280.0, 616.0),
        status_bar_frame: host_layout_frame_fixture(0.0, 698.0, 1280.0, 22.0),
        left_region_frame: host_layout_frame_fixture(0.0, 64.0, 280.0, 516.0),
        document_region_frame: host_layout_frame_fixture(280.0, 64.0, 720.0, 516.0),
        right_region_frame: host_layout_frame_fixture(1000.0, 64.0, 280.0, 516.0),
        bottom_region_frame: host_layout_frame_fixture(0.0, 580.0, 1280.0, 118.0),
        left_splitter_frame: host_layout_frame_fixture(276.0, 64.0, 4.0, 516.0),
        right_splitter_frame: host_layout_frame_fixture(1000.0, 64.0, 4.0, 516.0),
        bottom_splitter_frame: host_layout_frame_fixture(0.0, 576.0, 1280.0, 4.0),
        viewport_content_frame: host_layout_frame_fixture(280.0, 96.0, 720.0, 484.0),
    }
}

fn host_layout_frame_fixture(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
