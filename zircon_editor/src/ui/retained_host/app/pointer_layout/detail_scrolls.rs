use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_detail_pointer_layouts(
        &mut self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) {
        self.sync_console_pointer_layout(&chrome.status_line);
        self.sync_inspector_pointer_layout();
        self.sync_browser_asset_details_pointer_layout(&chrome.asset_browser);
    }

    pub(in crate::ui::retained_host::app) fn sync_console_pointer_layout(
        &mut self,
        status_line: &str,
    ) {
        if !self.console_scroll_surface.has_size() {
            self.apply_console_pointer_state_to_ui();
            return;
        }

        let size = self.console_scroll_surface.size();
        self.console_scroll_surface.sync(console_scroll_layout(
            size,
            console_content_extent(status_line, size.width, false, ""),
        ));
        self.apply_console_pointer_state_to_ui();
    }

    pub(in crate::ui::retained_host::app) fn apply_console_pointer_state_to_ui(&self) {
        self.pane_surface_host()
            .set_console_scroll_px(self.console_scroll_surface.scroll_offset());
    }

    pub(in crate::ui::retained_host::app) fn sync_inspector_pointer_layout(&mut self) {
        if !self.inspector_scroll_surface.has_size() {
            self.apply_inspector_pointer_state_to_ui();
            return;
        }

        self.inspector_scroll_surface.sync(inspector_scroll_layout(
            self.inspector_scroll_surface.size(),
        ));
        self.apply_inspector_pointer_state_to_ui();
    }

    pub(in crate::ui::retained_host::app) fn apply_inspector_pointer_state_to_ui(&self) {
        self.pane_surface_host()
            .set_inspector_scroll_px(self.inspector_scroll_surface.scroll_offset());
    }

    pub(in crate::ui::retained_host::app) fn sync_browser_asset_details_pointer_layout(
        &mut self,
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
    ) {
        if !self.browser_asset_details_scroll_surface.has_size() {
            self.apply_browser_asset_details_pointer_state_to_ui();
            return;
        }

        self.browser_asset_details_scroll_surface
            .sync(asset_details_scroll_layout(
                self.browser_asset_details_scroll_surface.size(),
                &snapshot.selection,
            ));
        self.apply_browser_asset_details_pointer_state_to_ui();
    }

    pub(in crate::ui::retained_host::app) fn apply_browser_asset_details_pointer_state_to_ui(
        &self,
    ) {
        self.pane_surface_host()
            .set_browser_asset_details_scroll_px(
                self.browser_asset_details_scroll_surface.scroll_offset(),
            );
    }
}
