use super::super::{RetainedEditorHost, SceneEntry, ViewContentKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_hierarchy_pointer_target(
        &mut self,
        width: f32,
        height: f32,
        focus_source_window: bool,
    ) -> Vec<SceneEntry> {
        self.use_committed_pointer_layout();
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        if focus_source_window {
            self.focus_callback_source_window();
        }
        scene_entries
    }
}
