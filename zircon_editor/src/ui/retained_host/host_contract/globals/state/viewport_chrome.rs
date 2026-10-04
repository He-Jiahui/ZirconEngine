use std::collections::BTreeMap;
use std::sync::Arc;

use super::HostContractState;
use crate::ui::retained_host::host_contract::data::{
    FloatingWindowData, HostWindowPresentationData, PaneData, SceneViewportChromeData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::ModelRc;

const STATUS_GRID_CONTROL_ID: &str = "WorkbenchStatusGrid";
const STATUS_SNAP_CONTROL_ID: &str = "WorkbenchStatusSnap";

impl HostContractState {
    pub(crate) fn patch_scene_viewport_chrome(
        &mut self,
        viewport: SceneViewportChromeData,
        status_grid_text: &str,
        status_snap_text: &str,
    ) -> bool {
        if !scene_viewport_chrome_needs_patch(
            &self.host_presentation,
            &viewport,
            status_grid_text,
            status_snap_text,
        ) {
            return false;
        }
        let presentation = Arc::make_mut(&mut self.host_presentation);
        let scene = &mut presentation.host_scene_data;
        let mut changed = false;
        changed |= patch_scene_pane(&mut scene.document_dock.pane, &viewport);
        for leaf in &mut scene.document_leaves {
            changed |= patch_scene_pane(&mut leaf.pane, &viewport);
        }
        changed |= patch_scene_pane(&mut scene.left_dock.pane, &viewport);
        changed |= patch_scene_pane(&mut scene.right_dock.pane, &viewport);
        changed |= patch_scene_pane(&mut scene.bottom_dock.pane, &viewport);
        changed |= patch_floating_windows(&mut scene.floating_layer.floating_windows, &viewport);
        changed |= patch_floating_windows(
            &mut presentation.native_floating_surface_data.floating_windows,
            &viewport,
        );
        changed |= patch_status_nodes(
            &mut presentation.workbench_window_nodes,
            status_grid_text,
            status_snap_text,
        );

        if changed {
            self.advance_structure_generation();
            self.advance_geometry_generation();
        }
        changed
    }

    pub(crate) fn patch_native_scene_viewport_chrome(
        &mut self,
        row: usize,
        window_id: &str,
        viewport: SceneViewportChromeData,
    ) -> bool {
        let windows = &self
            .host_presentation
            .native_floating_surface_data
            .floating_windows;
        let Some(window) = windows.get(row) else {
            return false;
        };
        if window.window_id.as_str() != window_id
            || !scene_pane_needs_patch(&window.active_pane, &viewport)
        {
            return false;
        }

        let presentation = Arc::make_mut(&mut self.host_presentation);
        let windows = &mut presentation.native_floating_surface_data.floating_windows;
        let mut next = windows
            .get(row)
            .expect("validated native presenter row should remain present")
            .clone();
        if !patch_scene_pane(&mut next.active_pane, &viewport) {
            return false;
        }
        *windows = windows.with_row_patches(BTreeMap::from([(row, next)]));
        self.advance_structure_generation();
        self.advance_geometry_generation();
        true
    }
}

fn scene_viewport_chrome_needs_patch(
    presentation: &HostWindowPresentationData,
    viewport: &SceneViewportChromeData,
    status_grid_text: &str,
    status_snap_text: &str,
) -> bool {
    let scene = &presentation.host_scene_data;
    scene_pane_needs_patch(&scene.document_dock.pane, viewport)
        || scene
            .document_leaves
            .iter()
            .any(|leaf| scene_pane_needs_patch(&leaf.pane, viewport))
        || scene_pane_needs_patch(&scene.left_dock.pane, viewport)
        || scene_pane_needs_patch(&scene.right_dock.pane, viewport)
        || scene_pane_needs_patch(&scene.bottom_dock.pane, viewport)
        || floating_windows_need_patch(&scene.floating_layer.floating_windows, viewport)
        || floating_windows_need_patch(
            &presentation.native_floating_surface_data.floating_windows,
            viewport,
        )
        || status_nodes_need_patch(
            &presentation.workbench_window_nodes,
            status_grid_text,
            status_snap_text,
        )
}

fn scene_pane_needs_patch(pane: &PaneData, viewport: &SceneViewportChromeData) -> bool {
    pane.kind.as_str() == "Scene" && !same_viewport_chrome(&pane.viewport, viewport)
}

fn floating_windows_need_patch(
    windows: &ModelRc<FloatingWindowData>,
    viewport: &SceneViewportChromeData,
) -> bool {
    windows
        .iter()
        .any(|window| scene_pane_needs_patch(&window.active_pane, viewport))
}

fn status_nodes_need_patch(
    nodes: &ModelRc<TemplatePaneNodeData>,
    grid_text: &str,
    snap_text: &str,
) -> bool {
    nodes.iter().any(|node| match node.control_id.as_str() {
        STATUS_GRID_CONTROL_ID => node.text.as_str() != grid_text,
        STATUS_SNAP_CONTROL_ID => node.text.as_str() != snap_text,
        _ => false,
    })
}

fn patch_scene_pane(pane: &mut PaneData, viewport: &SceneViewportChromeData) -> bool {
    if pane.kind.as_str() != "Scene" {
        return false;
    }
    let mut next = viewport.clone();
    next.toolbar_surface_frame = pane.viewport.toolbar_surface_frame.clone();
    next.toolbar_template_nodes = pane.viewport.toolbar_template_nodes.clone();
    next.toolbar_surface_key = pane.viewport.toolbar_surface_key.clone();
    next.toolbar_enter_play_enabled = pane.viewport.toolbar_enter_play_enabled;
    next.toolbar_exit_play_enabled = pane.viewport.toolbar_exit_play_enabled;
    next.toolbar_is_playing = pane.viewport.toolbar_is_playing;
    if same_viewport_chrome(&pane.viewport, &next) {
        return false;
    }
    pane.viewport = next;
    true
}

fn patch_floating_windows(
    windows: &mut ModelRc<FloatingWindowData>,
    viewport: &SceneViewportChromeData,
) -> bool {
    let mut patches = BTreeMap::new();
    for (row, window) in windows.iter().enumerate() {
        let mut next = window.clone();
        if patch_scene_pane(&mut next.active_pane, viewport) {
            patches.insert(row, next);
        }
    }
    if patches.is_empty() {
        return false;
    }
    *windows = windows.with_row_patches(patches);
    true
}

fn patch_status_nodes(
    nodes: &mut ModelRc<TemplatePaneNodeData>,
    grid_text: &str,
    snap_text: &str,
) -> bool {
    let mut patches = BTreeMap::new();
    for (row, node) in nodes.iter().enumerate() {
        let next_text = match node.control_id.as_str() {
            STATUS_GRID_CONTROL_ID => grid_text,
            STATUS_SNAP_CONTROL_ID => snap_text,
            _ => continue,
        };
        if node.text.as_str() == next_text {
            continue;
        }
        let mut next = node.clone();
        next.text = next_text.to_string();
        patches.insert(row, next);
    }
    if patches.is_empty() {
        return false;
    }
    *nodes = nodes.with_row_patches(patches);
    true
}

fn same_viewport_chrome(left: &SceneViewportChromeData, right: &SceneViewportChromeData) -> bool {
    left.mode == right.mode
        && left.transform_space == right.transform_space
        && left.pivot_mode == right.pivot_mode
        && left.projection_mode == right.projection_mode
        && left.view_orientation == right.view_orientation
        && left.display_mode == right.display_mode
        && left.grid_mode == right.grid_mode
        && left.gizmos_enabled == right.gizmos_enabled
        && left.preview_lighting == right.preview_lighting
        && left.preview_skybox == right.preview_skybox
        && left.translate_snap == right.translate_snap
        && left.rotate_snap_deg == right.rotate_snap_deg
        && left.scale_snap == right.scale_snap
        && left.translate_snap_label == right.translate_snap_label
        && left.rotate_snap_label == right.rotate_snap_label
        && left.scale_snap_label == right.scale_snap_label
}

#[cfg(test)]
#[path = "tests/viewport_chrome.rs"]
mod tests;
