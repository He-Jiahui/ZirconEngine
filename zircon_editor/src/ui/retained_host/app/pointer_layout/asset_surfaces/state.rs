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
    ) -> Option<Arc<crate::ui::workbench::snapshot::AssetWorkspaceSnapshot>> {
        self.asset_surface_pointer_state(surface_mode)?
            .snapshot
            .clone()
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn pointer_snapshot_reuses_the_committed_asset_projection() {
        let source = include_str!("state.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(
            !production.contains("self.runtime.editor_snapshot()"),
            "pointer-frequency callbacks must not build the complete editor snapshot"
        );
    }

    #[test]
    fn unchanged_pointer_sizes_do_not_rebuild_list_layouts() {
        let content = include_str!("../../asset_content_pointer/target/dispatch.rs");
        let tree = include_str!("../../asset_tree_pointer/target.rs");
        let reference = include_str!("../../asset_reference_pointer/target/dispatch.rs");

        assert!(content.contains("if surface.content_size != target.content_size"));
        assert!(tree.contains("if surface.tree_size != tree_size"));
        assert!(reference.contains("if list.size != target.list_size"));
    }
}
