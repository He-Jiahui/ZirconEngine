use std::sync::Arc;

use zircon_runtime::scene::WorldInspectionHierarchyRow;

use super::super::{RetainedEditorHost, ViewContentKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_hierarchy_pointer_target(
        &mut self,
        width: f32,
        height: f32,
        focus_source_window: bool,
    ) -> Arc<[WorldInspectionHierarchyRow]> {
        self.use_committed_pointer_layout();
        let target_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = Arc::clone(&self.hierarchy_scene_entries);
        if self.hierarchy_pointer_size != target_size {
            self.hierarchy_pointer_size = target_size;
            self.sync_hierarchy_pointer_layout(scene_entries.as_ref());
        }
        if focus_source_window {
            self.focus_callback_source_window();
        }
        scene_entries
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn hierarchy_pointer_reuses_the_committed_scene_projection() {
        let source = include_str!("target.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(!production.contains("self.runtime.editor_snapshot()"));
        assert!(production.contains("if self.hierarchy_pointer_size != target_size"));
    }
}
