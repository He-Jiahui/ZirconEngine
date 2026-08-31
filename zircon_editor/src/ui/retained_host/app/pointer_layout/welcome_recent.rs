use super::super::*;
use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_viewport_for_layout;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_welcome_recent_pointer_layout(
        &mut self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) {
        let pane_size = self
            .resolve_welcome_recent_pointer_size()
            .unwrap_or(self.welcome_recent_pointer_size);
        if pane_size.width <= 0.0 || pane_size.height <= 0.0 {
            return;
        }
        self.welcome_recent_pointer_size = pane_size;
        let viewport = self.resolve_welcome_recent_pointer_viewport(pane_size);
        let recent_project_paths = workbench_snapshot_access::welcome_recent_project_paths(chrome);

        if self
            .welcome_recent_pointer_bridge
            .sync(WelcomeRecentPointerLayout {
                viewport,
                recent_project_paths,
            })
        {
            self.apply_welcome_recent_pointer_state_to_ui();
        }
    }

    pub(in crate::ui::retained_host::app) fn sync_welcome_recent_pointer_size(&mut self) -> bool {
        let pane_size = self
            .resolve_welcome_recent_pointer_size()
            .unwrap_or(self.welcome_recent_pointer_size);
        if pane_size.width <= 0.0 || pane_size.height <= 0.0 {
            return false;
        }
        self.welcome_recent_pointer_size = pane_size;
        let viewport = self.resolve_welcome_recent_pointer_viewport(pane_size);

        self.welcome_recent_pointer_bridge.sync_viewport(viewport)
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

    fn resolve_welcome_recent_pointer_viewport(&self, pane_size: UiSize) -> UiFrame {
        let welcome = self.pane_surface_host().get_welcome_pane();
        welcome_recent_viewport_for_layout(&welcome.layout, pane_size)
    }

    pub(in crate::ui::retained_host::app) fn apply_welcome_recent_pointer_state_to_ui(&self) {
        let state = self.welcome_recent_pointer_bridge.state();
        let pane_surface_host = self.pane_surface_host();
        pane_surface_host.set_welcome_recent_scroll_px(state.scroll_offset);
        pane_surface_host.set_hovered_welcome_recent_index(
            state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        pane_surface_host.set_hovered_welcome_recent_action(match state.hovered_action {
            Some(WelcomeRecentPointerAction::Open) => 0,
            Some(WelcomeRecentPointerAction::Safe) => 1,
            Some(WelcomeRecentPointerAction::Recover) => 2,
            Some(WelcomeRecentPointerAction::Remove) => 3,
            None => -1,
        });
    }
}
