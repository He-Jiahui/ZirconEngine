use super::*;

impl SlintEditorHost {
    pub(super) fn hierarchy_pointer_clicked(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        self.focus_callback_source_window();
        match callback_dispatch::dispatch_shared_hierarchy_pointer_click(
            &self.runtime,
            &mut self.hierarchy_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.pointer.state;
                self.apply_hierarchy_pointer_state_to_ui();
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn hierarchy_pointer_moved(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        match self
            .hierarchy_pointer_bridge
            .handle_move(UiPoint::new(x, y))
        {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.state;
                self.apply_hierarchy_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn hierarchy_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        self.focus_callback_source_window();
        match self
            .hierarchy_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.state;
                self.apply_hierarchy_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
