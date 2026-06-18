use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_welcome_recent_pointer_layout(
        &mut self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) {
        let pane_size = self
            .resolve_welcome_recent_pointer_size()
            .unwrap_or(self.welcome_recent_pointer_size);
        if pane_size.width <= 0.0 || pane_size.height <= 0.0 {
            self.apply_welcome_recent_pointer_state_to_ui();
            return;
        }
        self.welcome_recent_pointer_size = pane_size;
        let recent_project_paths = workbench_snapshot_access::welcome_recent_project_paths(chrome);

        self.welcome_recent_pointer_bridge.sync(
            WelcomeRecentPointerLayout {
                pane_size,
                recent_project_paths,
            },
            self.welcome_recent_pointer_state.clone(),
        );
        self.apply_welcome_recent_pointer_state_to_ui();
    }

    pub(in crate::ui::retained_host::app) fn sync_welcome_recent_pointer_size(&mut self) {
        let pane_size = self
            .resolve_welcome_recent_pointer_size()
            .unwrap_or(self.welcome_recent_pointer_size);
        if pane_size.width <= 0.0 || pane_size.height <= 0.0 {
            self.apply_welcome_recent_pointer_state_to_ui();
            return;
        }
        self.welcome_recent_pointer_size = pane_size;

        self.welcome_recent_pointer_bridge
            .sync_pane_size(pane_size, self.welcome_recent_pointer_state.clone());
        self.apply_welcome_recent_pointer_state_to_ui();
    }

    fn resolve_welcome_recent_pointer_size(&self) -> Option<UiSize> {
        if self.welcome_recent_pointer_size.width > 0.0
            && self.welcome_recent_pointer_size.height > 0.0
        {
            return Some(self.welcome_recent_pointer_size);
        }

        self.template_bridge
            .control_frame(callback_dispatch::PANE_SURFACE_CONTROL_ID)
            .map(|frame| UiSize::new(frame.width.max(0.0), frame.height.max(0.0)))
            .filter(|size| size.width > 0.0 && size.height > 0.0)
    }

    pub(in crate::ui::retained_host::app) fn apply_welcome_recent_pointer_state_to_ui(&self) {
        let pane_surface_host = self.pane_surface_host();
        pane_surface_host
            .set_welcome_recent_scroll_px(self.welcome_recent_pointer_state.scroll_offset);
        pane_surface_host.set_hovered_welcome_recent_index(
            self.welcome_recent_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        pane_surface_host.set_hovered_welcome_recent_action(
            match self.welcome_recent_pointer_state.hovered_action {
                Some(WelcomeRecentPointerAction::Open) => 0,
                Some(WelcomeRecentPointerAction::Remove) => 1,
                None => -1,
            },
        );
    }
}
