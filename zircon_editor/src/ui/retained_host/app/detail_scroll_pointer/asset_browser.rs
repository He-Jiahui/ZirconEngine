use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn browser_asset_details_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        self.browser_asset_details_scroll_surface.set_size(
            self.resolve_callback_surface_size_for_kind(
                width,
                height,
                self.browser_asset_details_scroll_surface.size(),
                ViewContentKind::AssetBrowser,
            ),
        );
        let snapshot = self.runtime.editor_snapshot().asset_browser;
        self.sync_browser_asset_details_pointer_layout(&snapshot);
        match self
            .browser_asset_details_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(()) => {
                self.apply_browser_asset_details_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
