use std::collections::BTreeMap;
use std::sync::Arc;

use super::HostContractState;
use crate::ui::retained_host::host_contract::data::{
    FloatingWindowData, PaneData, SceneViewportChromeData, TemplatePaneNodeData,
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
        let presentation = Arc::make_mut(&mut self.host_presentation);
        let scene = &mut presentation.host_scene_data;
        let mut changed = false;
        changed |= patch_scene_pane(&mut scene.document_dock.pane, &viewport);
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
            self.presentation_structure_generation =
                self.presentation_structure_generation.saturating_add(1);
        }
        changed
    }
}

fn patch_scene_pane(pane: &mut PaneData, viewport: &SceneViewportChromeData) -> bool {
    if pane.kind.as_str() != "Scene" {
        return false;
    }
    let mut next = viewport.clone();
    next.toolbar_surface_frame = pane.viewport.toolbar_surface_frame.clone();
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
mod tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn scene_chrome_patch_preserves_geometry_and_unrelated_rows() {
        let mut presentation = crate::ui::retained_host::HostWindowPresentationData::default();
        presentation.host_scene_data.document_dock.pane.kind = "Scene".to_string();
        let toolbar_frame =
            Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
        presentation
            .host_scene_data
            .document_dock
            .pane
            .viewport
            .toolbar_surface_frame = Some(Arc::clone(&toolbar_frame));
        presentation.workbench_window_nodes = model_rc(vec![
            TemplatePaneNodeData {
                control_id: STATUS_GRID_CONTROL_ID.to_string(),
                text: "Grid: Off".to_string(),
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                control_id: "Unrelated".to_string(),
                text: "stable".to_string(),
                ..TemplatePaneNodeData::default()
            },
        ]);
        let mut state = HostContractState::new(
            crate::ui::retained_host::primitives::PhysicalSize::new(1280, 720),
        );
        state.replace_host_presentation(presentation);
        let original_nodes = state.host_presentation.workbench_window_nodes.clone();
        let viewport = SceneViewportChromeData {
            grid_mode: "VisibleAndSnap".to_string(),
            ..SceneViewportChromeData::default()
        };

        assert!(state.patch_scene_viewport_chrome(viewport, "Grid: 1 m", "Snap: On"));

        let patched = &state.host_presentation;
        assert_eq!(
            patched
                .host_scene_data
                .document_dock
                .pane
                .viewport
                .grid_mode,
            "VisibleAndSnap"
        );
        assert!(Arc::ptr_eq(
            patched
                .host_scene_data
                .document_dock
                .pane
                .viewport
                .toolbar_surface_frame
                .as_ref()
                .expect("toolbar frame"),
            &toolbar_frame,
        ));
        assert_eq!(
            patched.workbench_window_nodes.get(0).unwrap().text,
            "Grid: 1 m"
        );
        assert!(patched
            .workbench_window_nodes
            .shares_row_with(&original_nodes, 1));
    }
}
