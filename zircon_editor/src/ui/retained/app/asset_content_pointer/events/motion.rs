use super::super::super::{RetainedEditorHost, UiPoint};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_content_pointer_moved(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        if surface_mode == "browser" {
            record_current_ui_perf_counter(
                UiPerfCounter::AssetBrowserLogicalItemCount,
                target.snapshot.visible_assets.len() as f64,
            );
        }
        let previous_size = self
            .asset_surface_pointer_state(surface_mode)
            .map(|surface| surface.content_size);
        let point = UiPoint::new(x, y);
        let Some(state) =
            self.dispatch_prepared_asset_content_pointer(surface_mode, &target, false, |bridge| {
                bridge.update_hovered_row(point)
            })
        else {
            return;
        };
        if let Some(state) = state {
            self.write_asset_content_pointer_state(surface_mode, state);
        }
        let current_window = self
            .asset_surface_pointer_state(surface_mode)
            .map(|surface| (surface.content_size, surface.content_state.scroll_offset));
        if let Some((size, scroll_px)) =
            current_window.filter(|(size, _)| previous_size != Some(*size))
        {
            self.refresh_scrolled_asset_previews(
                surface_mode,
                target.snapshot.as_ref(),
                size,
                scroll_px,
            );
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_content_pointer_scrolled(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        let previous_window = self
            .asset_surface_pointer_state(surface_mode)
            .map(|surface| (surface.content_size, surface.content_state.scroll_offset));
        let point = UiPoint::new(x, y);
        let Some(dispatch) =
            self.dispatch_prepared_asset_content_pointer(surface_mode, &target, false, |bridge| {
                bridge.handle_scroll(point, delta)
            })
        else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                let changed =
                    self.asset_surface_pointer_state(surface_mode)
                        .is_some_and(|surface| {
                            previous_window
                                != Some((surface.content_size, dispatch.state.scroll_offset))
                        });
                self.write_asset_content_pointer_state(surface_mode, dispatch.state);
                if changed {
                    let current_window = self
                        .asset_surface_pointer_state(surface_mode)
                        .map(|surface| (surface.content_size, surface.content_state.scroll_offset));
                    if let Some((size, scroll_px)) = current_window {
                        self.refresh_scrolled_asset_previews(
                            surface_mode,
                            target.snapshot.as_ref(),
                            size,
                            scroll_px,
                        );
                    }
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
