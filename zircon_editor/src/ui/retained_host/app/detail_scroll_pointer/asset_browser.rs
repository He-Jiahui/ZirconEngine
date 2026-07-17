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
        let size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.browser_asset_details_scroll_surface.size(),
            ViewContentKind::AssetBrowser,
        );
        if self.browser_asset_details_scroll_surface.set_size(size) {
            let Some(snapshot) = self.asset_workspace_snapshot_for_pointer("browser") else {
                self.set_status_line("Asset Browser pointer projection is not ready");
                return;
            };
            self.sync_browser_asset_details_pointer_layout(snapshot.as_ref());
        }
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn asset_details_scroll_reuses_the_committed_browser_projection() {
        let source = include_str!("asset_browser.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(!production.contains("self.runtime.editor_snapshot()"));
    }
}
