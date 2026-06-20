use super::super::super::*;

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
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        surface.tree_bridge.sync(
            AssetFolderTreePointerLayout::from_snapshot(snapshot, surface.tree_size),
            surface.tree_state.clone(),
        );
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(snapshot, surface.content_size),
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
}
