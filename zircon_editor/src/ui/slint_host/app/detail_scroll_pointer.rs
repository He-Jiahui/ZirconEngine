use super::*;

impl SlintEditorHost {
    pub(super) fn console_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.focus_callback_source_window();
        self.console_scroll_surface
            .set_size(self.resolve_callback_surface_size_for_kind(
                width,
                height,
                self.console_scroll_surface.size(),
                ViewContentKind::Console,
            ));
        let chrome = self.runtime.chrome_snapshot();
        self.sync_console_pointer_layout(&chrome);
        match self
            .console_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(()) => {
                self.apply_console_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn inspector_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.focus_callback_source_window();
        self.inspector_scroll_surface
            .set_size(self.resolve_callback_surface_size_for_kind(
                width,
                height,
                self.inspector_scroll_surface.size(),
                ViewContentKind::Inspector,
            ));
        self.sync_inspector_pointer_layout();
        match self
            .inspector_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(()) => {
                self.apply_inspector_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn browser_asset_details_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
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
