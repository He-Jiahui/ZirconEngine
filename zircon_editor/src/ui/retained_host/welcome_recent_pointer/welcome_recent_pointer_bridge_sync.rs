use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;
use zircon_runtime_interface::ui::layout::UiSize;

impl WelcomeRecentPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: WelcomeRecentPointerLayout,
        state: WelcomeRecentPointerState,
    ) -> bool {
        if self.layout == layout && self.state == state {
            return false;
        }

        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
        true
    }

    pub(crate) fn sync_pane_size(
        &mut self,
        pane_size: UiSize,
        state: WelcomeRecentPointerState,
    ) -> bool {
        if self.layout.pane_size == pane_size && self.state == state {
            return false;
        }

        self.layout.pane_size = pane_size;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_pane_size_preserves_recent_project_paths() {
        let mut bridge = WelcomeRecentPointerBridge::new();
        let state = WelcomeRecentPointerState::default();
        let project_paths = vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")];

        assert!(bridge.sync(
            WelcomeRecentPointerLayout {
                pane_size: UiSize::new(120.0, 80.0),
                recent_project_paths: project_paths.clone(),
            },
            state.clone(),
        ));

        assert!(!bridge.sync_pane_size(UiSize::new(120.0, 80.0), state.clone()));
        assert_eq!(bridge.layout.recent_project_paths, project_paths);

        assert!(bridge.sync_pane_size(UiSize::new(180.0, 80.0), state));
        assert_eq!(
            bridge.layout.recent_project_paths,
            vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")]
        );
    }
}
