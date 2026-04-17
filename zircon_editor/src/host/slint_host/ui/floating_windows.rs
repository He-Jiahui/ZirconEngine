use super::apply_presentation::frame_rect;
use super::model_rc::model_rc;
use super::pane_projection::{blank_pane, find_tab_snapshot, pane_from_tab};
use super::workbench_tabs::document_tab_data;
use super::*;
use crate::host::slint_host::floating_window_projection::{
    FloatingWindowProjectionBundle, resolve_floating_window_outer_frame,
};

pub(super) fn collect_floating_windows(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<FloatingWindowData> {
    model
        .floating_windows
        .iter()
        .map(|window| {
            floating_window_data(
                window,
                chrome,
                geometry,
                ui_asset_panes,
                floating_window_projection_bundle,
            )
        })
        .collect()
}

fn floating_window_data(
    window: &crate::FloatingWindowModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> FloatingWindowData {
    let active_tab = window.focus_target_tab();
    let active_pane = active_tab
        .map(|tab| {
            pane_from_tab(
                &tab.instance_id.0,
                &window.window_id.0,
                &tab.title,
                &tab.icon_key,
                tab.content_kind,
                tab.empty_state.as_ref(),
                find_tab_snapshot(chrome, &tab.instance_id.0),
                chrome,
                ui_asset_panes.get(&tab.instance_id.0),
            )
        })
        .unwrap_or_else(blank_pane);
    let frame = floating_window_projection_bundle
        .outer_frame(&window.window_id)
        .unwrap_or_else(|| resolve_floating_window_outer_frame(geometry, &window.window_id));

    FloatingWindowData {
        window_id: window.window_id.0.clone().into(),
        title: window.title.clone().into(),
        frame: frame_rect(frame),
        target_group: floating_window_group_key(&window.window_id).into(),
        left_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::DockEdge::Left,
        )
        .into(),
        right_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::DockEdge::Right,
        )
        .into(),
        top_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::DockEdge::Top,
        )
        .into(),
        bottom_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::DockEdge::Bottom,
        )
        .into(),
        focus_target_id: window
            .focus_target_instance()
            .map(|instance_id| instance_id.0.clone())
            .unwrap_or_default()
            .into(),
        tabs: model_rc(window.tabs.iter().map(document_tab_data).collect()),
        active_pane,
    }
}
