use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_detail_pointer_layouts(
        &mut self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) {
        if let Some(size) = self.ui.console_output_viewport_size(
            self.callback_source_window.as_ref().map(|id| id.0.as_str()),
        ) {
            self.console_scroll_surface.set_size(size);
        }
        self.sync_console_pointer_layout(&chrome.console_output);
        self.sync_inspector_pointer_layout();
        self.sync_browser_asset_details_pointer_layout(&chrome.asset_browser);
    }

    pub(in crate::ui::retained_host::app) fn sync_console_pointer_layout(
        &mut self,
        output: &crate::ui::workbench::snapshot::ConsoleOutputSnapshot,
    ) {
        if !self.console_scroll_surface.has_size() {
            return;
        }

        let size = self.console_scroll_surface.size();
        if self
            .console_scroll_surface
            .sync_following_tail(console_scroll_layout(
                size,
                console_snapshot_content_extent(output),
            ))
        {
            self.apply_console_pointer_state_to_ui();
        }
    }

    pub(in crate::ui::retained_host::app) fn apply_console_pointer_state_to_ui(&self) {
        self.pane_surface_host()
            .set_console_scroll_px(self.console_scroll_surface.scroll_offset());
    }

    pub(in crate::ui::retained_host::app) fn sync_inspector_pointer_layout(&mut self) {
        if !self.inspector_scroll_surface.has_size() {
            return;
        }

        if self.inspector_scroll_surface.sync(inspector_scroll_layout(
            self.inspector_scroll_surface.size(),
        )) {
            self.apply_inspector_pointer_state_to_ui();
        }
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
            return;
        }

        if self
            .browser_asset_details_scroll_surface
            .sync(asset_details_scroll_layout(
                self.browser_asset_details_scroll_surface.size(),
                &snapshot.selection,
            ))
        {
            self.apply_browser_asset_details_pointer_state_to_ui();
        }
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
