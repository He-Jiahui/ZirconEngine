use super::super::*;

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

    pub(in crate::ui::retained_host::app) fn apply_asset_pointer_state_to_ui(
        &self,
        surface_mode: &str,
    ) {
        let Some(surface) = self.asset_surface_pointer_state(surface_mode) else {
            return;
        };
        let pane_surface_host = self.pane_surface_host();

        let tree_hovered = surface
            .tree_state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let content_hovered = surface
            .content_state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let references_hovered = surface
            .references
            .state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let used_by_hovered = surface
            .used_by
            .state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);

        match surface_mode {
            "activity" => {
                pane_surface_host.set_activity_asset_tree_hovered_index(tree_hovered);
                pane_surface_host
                    .set_activity_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                pane_surface_host.set_activity_asset_content_hovered_index(content_hovered);
                pane_surface_host
                    .set_activity_asset_content_scroll_px(surface.content_state.scroll_offset);
                pane_surface_host.set_activity_asset_references_hovered_index(references_hovered);
                pane_surface_host.set_activity_asset_references_scroll_px(
                    surface.references.state.scroll_offset,
                );
                pane_surface_host.set_activity_asset_used_by_hovered_index(used_by_hovered);
                pane_surface_host
                    .set_activity_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            "browser" => {
                pane_surface_host.set_browser_asset_tree_hovered_index(tree_hovered);
                pane_surface_host
                    .set_browser_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                pane_surface_host.set_browser_asset_content_hovered_index(content_hovered);
                pane_surface_host
                    .set_browser_asset_content_scroll_px(surface.content_state.scroll_offset);
                pane_surface_host.set_browser_asset_references_hovered_index(references_hovered);
                pane_surface_host
                    .set_browser_asset_references_scroll_px(surface.references.state.scroll_offset);
                pane_surface_host.set_browser_asset_used_by_hovered_index(used_by_hovered);
                pane_surface_host
                    .set_browser_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            _ => {}
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_surface_pointer_state(
        &self,
        surface_mode: &str,
    ) -> Option<&AssetSurfacePointerState> {
        match surface_mode {
            "activity" => Some(&self.activity_asset_pointer),
            "browser" => Some(&self.browser_asset_pointer),
            _ => None,
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_surface_pointer_state_mut(
        &mut self,
        surface_mode: &str,
    ) -> Option<&mut AssetSurfacePointerState> {
        match surface_mode {
            "activity" => Some(&mut self.activity_asset_pointer),
            "browser" => Some(&mut self.browser_asset_pointer),
            _ => None,
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_workspace_snapshot_for_pointer(
        &self,
        surface_mode: &str,
    ) -> Option<crate::ui::workbench::snapshot::AssetWorkspaceSnapshot> {
        let snapshot = self.runtime.editor_snapshot();
        match surface_mode {
            "activity" => Some(snapshot.asset_activity),
            "browser" => Some(snapshot.asset_browser),
            _ => None,
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_reference_layout(
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        list_kind: &str,
        pane_size: UiSize,
    ) -> Option<AssetReferenceListPointerLayout> {
        match list_kind {
            "references" => Some(AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                pane_size,
            )),
            "used_by" => Some(AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.used_by,
                pane_size,
            )),
            _ => None,
        }
    }
}
