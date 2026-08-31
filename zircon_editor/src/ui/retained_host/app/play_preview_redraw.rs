use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use crate::core::play::{PlayKind, PlayMode, PlayPreviewFrame};
use crate::ui::retained_host::host_contract::data::HostViewportOverlayImageData;

use super::*;

impl RetainedEditorHost {
    pub(super) fn poll_play_preview_frame_for_native_host(&mut self) {
        let size = ZrRuntimeViewportSizeV1::new(self.viewport_size.x, self.viewport_size.y);
        let play_mode = self.runtime.play_sessions().mode_snapshot();
        let image_updated = match play_mode {
            PlayMode::Playing {
                kind: PlayKind::Play,
            } => {
                let simulate_cleared = self
                    .ui
                    .global::<PaneSurfaceHostContext>()
                    .clear_simulate_viewport_image();
                let visible = self
                    .ui
                    .global::<PaneSurfaceHostContext>()
                    .game_viewport_visible();
                let game_changed = if !visible {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "play.preview.hidden_capture_skipped_count",
                        1
                    );
                    false
                } else if size.width == 0 || size.height == 0 {
                    self.ui
                        .global::<PaneSurfaceHostContext>()
                        .clear_game_viewport_image()
                } else {
                    match self.runtime.play_sessions().capture_preview_frame(size) {
                        Ok(Some(frame)) => self
                            .ui
                            .global::<PaneSurfaceHostContext>()
                            .set_game_viewport_frame(frame),
                        Ok(None) => self
                            .ui
                            .global::<PaneSurfaceHostContext>()
                            .clear_game_viewport_image(),
                        Err(error) => {
                            let cleared = self
                                .ui
                                .global::<PaneSurfaceHostContext>()
                                .clear_game_viewport_image();
                            self.set_status_line(error.to_string());
                            cleared
                        }
                    }
                };
                simulate_cleared | game_changed
            }
            PlayMode::Playing {
                kind: PlayKind::Simulate,
            } => {
                let game_cleared = self
                    .ui
                    .global::<PaneSurfaceHostContext>()
                    .clear_game_viewport_image();
                let visible = self
                    .ui
                    .global::<PaneSurfaceHostContext>()
                    .scene_viewport_visible();
                let simulate_changed = if !visible {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "play.simulate.hidden_capture_skipped_count",
                        1
                    );
                    false
                } else if size.width == 0 || size.height == 0 {
                    self.ui
                        .global::<PaneSurfaceHostContext>()
                        .clear_simulate_viewport_image()
                } else {
                    match self.runtime.play_sessions().capture_preview_frame(size) {
                        Ok(Some(frame)) => self.set_simulate_viewport_frame_with_gizmo(frame),
                        Ok(None) => self
                            .ui
                            .global::<PaneSurfaceHostContext>()
                            .clear_simulate_viewport_image(),
                        Err(error) => {
                            let cleared = self
                                .ui
                                .global::<PaneSurfaceHostContext>()
                                .clear_simulate_viewport_image();
                            self.set_status_line(error.to_string());
                            cleared
                        }
                    }
                };
                game_cleared | simulate_changed
            }
            PlayMode::Edit | PlayMode::Building { .. } | PlayMode::CleanupFailed { .. } => {
                let pane = self.ui.global::<PaneSurfaceHostContext>();
                pane.clear_game_viewport_image() | pane.clear_simulate_viewport_image()
            }
        };

        if image_updated {
            let frame = self.ui.get_host_window_bootstrap().viewport_content_frame;
            self.record_paint_only_invalidation(HostInvalidationMask::VIEWPORT_IMAGE);
            self.ui.request_redraw_region(frame);
        }
    }

    fn set_simulate_viewport_frame_with_gizmo(&mut self, frame: PlayPreviewFrame) -> bool {
        let overlay = match self.runtime.play_gizmo_overlay_snapshot(frame.identity()) {
            Ok(Some(snapshot)) => {
                let (resource_scope, viewport, lines) = snapshot.into_raster_parts();
                let overlay = HostViewportOverlayImageData::from_screen_lines(
                    resource_scope.as_str(),
                    viewport,
                    &lines,
                );
                if let Some(overlay) = overlay.as_ref() {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "play.gizmo.overlay_raster_bytes",
                        overlay.rgba.len()
                    );
                }
                overlay
            }
            Ok(None) => None,
            Err(error) => {
                self.set_status_line(error.to_string());
                None
            }
        };
        self.ui
            .global::<PaneSurfaceHostContext>()
            .set_simulate_viewport_frame(frame, overlay)
    }
}
