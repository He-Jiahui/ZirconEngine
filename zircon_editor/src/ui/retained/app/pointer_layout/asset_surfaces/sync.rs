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

    // 资产快照变化时重建树、内容和引用列表的指针布局；仅 pane 尺寸变化走下方几何同步路径。
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
        surface.sync_snapshot_pointer_layouts(snapshot, surface_profile);
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

impl AssetSurfacePointerState {
    fn sync_snapshot_pointer_layouts(
        &mut self,
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        surface_profile: AssetContentSurfaceProfile,
    ) {
        self.snapshot = Some(Arc::new(snapshot.pointer_projection()));
        self.tree_bridge.sync(
            AssetFolderTreePointerLayout::from_snapshot(snapshot, self.tree_size),
            self.tree_state.clone(),
        );
        self.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(
                snapshot,
                self.content_size,
                surface_profile,
            ),
            self.content_state.clone(),
        );
        self.references.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                self.references.size,
            ),
            self.references.state.clone(),
        );
        self.used_by.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.used_by,
                self.used_by.size,
            ),
            self.used_by.state.clone(),
        );

        // Painting and the next sync must use the offsets clamped for this snapshot.
        self.tree_state = self.tree_bridge.pointer_state();
        self.content_state = self.content_bridge.pointer_state();
        self.references.state = self.references.bridge.pointer_state();
        self.used_by.state = self.used_by.bridge.pointer_state();
    }
}

#[cfg(test)]
#[path = "tests/scroll.rs"]
mod tests;
