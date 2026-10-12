use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use crate::core::play::{PlayKind, PlayMode, PlayPreviewCaptureError, PlayPreviewFrame};
use crate::ui::retained_host::host_contract::data::HostViewportOverlayImageData;
use crate::ui::retained_host::PaneSurfaceHostContext;

use super::*;

impl RetainedEditorHost {
    pub(super) fn poll_play_preview_frame_for_native_host(&mut self) {
        let size = ZrRuntimeViewportSizeV1::new(self.viewport_size.x, self.viewport_size.y);
        let play_mode = self.runtime.play_sessions().mode_snapshot();
        let image_updated = match play_mode {
            PlayMode::Playing {
                kind: PlayKind::Play,
            } => {
                static PLAY_POLL_ENTERED: std::sync::Once = std::sync::Once::new();
                PLAY_POLL_ENTERED
                    .call_once(|| eprintln!("mvp_play_boundary editor_play_preview_poll_entered"));
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
                    static CAPTURE_ENTERED: std::sync::Once = std::sync::Once::new();
                    static CAPTURE_RETURNED: std::sync::Once = std::sync::Once::new();
                    CAPTURE_ENTERED.call_once(|| {
                        eprintln!("mvp_play_boundary editor_first_preview_capture_entered")
                    });
                    let capture = self.capture_preview_frame_for_native_host(size);
                    CAPTURE_RETURNED.call_once(|| {
                        eprintln!("mvp_play_boundary editor_first_preview_capture_returned")
                    });
                    match capture {
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
                    match self.capture_preview_frame_for_native_host(size) {
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

    fn capture_preview_frame_for_native_host(
        &self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<Option<PlayPreviewFrame>, PlayPreviewCaptureError> {
        static TRACE_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("ZIRCON_TRACE_PLAY_STOP")
                .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        });
        if !trace_enabled {
            return self.runtime.play_sessions().capture_preview_frame(size);
        }

        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let started = std::time::Instant::now();
        eprintln!(
            "mvp_play_trace component=editor_preview seq={sequence} stage=capture_enter size={}x{}",
            size.width, size.height
        );
        let result = self.runtime.play_sessions().capture_preview_frame(size);
        let elapsed_us = started.elapsed().as_micros();
        match &result {
            Ok(Some(_)) => eprintln!(
                "mvp_play_trace component=editor_preview seq={sequence} stage=capture_return result=frame elapsed_us={elapsed_us}"
            ),
            Ok(None) => eprintln!(
                "mvp_play_trace component=editor_preview seq={sequence} stage=capture_return result=empty elapsed_us={elapsed_us}"
            ),
            Err(_) => eprintln!(
                "mvp_play_trace component=editor_preview seq={sequence} stage=capture_error elapsed_us={elapsed_us}"
            ),
        }
        result
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
