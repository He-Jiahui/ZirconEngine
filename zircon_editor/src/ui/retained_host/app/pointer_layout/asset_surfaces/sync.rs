use super::super::super::*;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::workbench::asset_content_layout::AssetContentSurfaceProfile;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_asset_pointer_layouts(
        &mut self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) {
        self.sync_asset_pointer_layout("activity", &chrome.asset_activity);
        self.sync_asset_pointer_layout("browser", &chrome.asset_browser);
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_pointer_layout(
        &mut self,
        surface_mode: &str,
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
    ) {
        let Some(surface_profile) = AssetContentSurfaceProfile::from_surface_mode(surface_mode)
        else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        record_current_ui_perf_counter(UiPerfCounter::AssetPointerSnapshotCloneCount, 1.0);
        surface.snapshot = Some(Arc::new(snapshot.pointer_projection()));
        surface.tree_bridge.sync(
            AssetFolderTreePointerLayout::from_snapshot(snapshot, surface.tree_size),
            surface.tree_state.clone(),
        );
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(
                snapshot,
                surface.content_size,
                surface_profile,
            ),
            surface.content_state.clone(),
        );
        surface.references.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                surface.references.size,
            ),
            surface.references.state.clone(),
        );
        surface.used_by.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.used_by,
                surface.used_by.size,
            ),
            surface.used_by.state.clone(),
        );
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_pointer_geometries(&mut self) {
        zircon_runtime::profile_scope!("editor", "retained_host", "asset_pointer_geometries");
        self.sync_asset_pointer_geometry("activity");
        self.sync_asset_pointer_geometry("browser");
    }

    fn sync_asset_pointer_geometry(&mut self, surface_mode: &str) {
        let state_changed = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                return;
            };
            let mut state_changed = false;
            if let Some(state) = surface.tree_bridge.sync_pane_size(surface.tree_size) {
                surface.tree_state = state;
                state_changed = true;
            }
            if let Some(state) = surface.content_bridge.sync_pane_size(surface.content_size) {
                surface.content_state = state;
                state_changed = true;
            }
            if let Some(state) = surface
                .references
                .bridge
                .sync_pane_size(surface.references.size)
            {
                surface.references.state = state;
                state_changed = true;
            }
            if let Some(state) = surface.used_by.bridge.sync_pane_size(surface.used_by.size) {
                surface.used_by.state = state;
                state_changed = true;
            }
            state_changed
        };
        if state_changed {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
    }
}
