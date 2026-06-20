use super::super::super::*;

impl RetainedEditorHost {
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
