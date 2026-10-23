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
                    if let Some(changed) = self.poll_gpu_play_preview() {
                        if changed {
                            self.ui.request_redraw_region(
                                self.ui.get_host_window_bootstrap().viewport_content_frame,
                            );
                        }
                        return;
                    }
                    let Some(capture) = self.poll_cpu_preview_frame(size) else {
                        return;
                    };
                    match capture {
                        Ok(Some(frame))
                            if self
                                .runtime
                                .play_sessions()
                                .preview_identity_is_current(frame.identity()) =>
                        {
                            self.ui
                                .global::<PaneSurfaceHostContext>()
                                .set_game_viewport_frame(frame)
                        }
                        Ok(Some(_)) => false,
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
                    let Some(capture) = self.poll_cpu_preview_frame(size) else {
                        return;
                    };
                    match capture {
                        Ok(Some(frame))
                            if self
                                .runtime
                                .play_sessions()
                                .preview_identity_is_current(frame.identity()) =>
                        {
                            self.set_simulate_viewport_frame_with_gizmo(frame)
                        }
                        Ok(Some(_)) => false,
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
            PlayMode::Edit
            | PlayMode::Building { .. }
            | PlayMode::Starting { .. }
            | PlayMode::Stopping { .. }
            | PlayMode::CleanupFailed { .. } => {
                self.play_gpu_identity = None;
                self.play_gpu_unavailable = None;
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

    fn poll_gpu_play_preview(&mut self) -> Option<bool> {
        if !self.ui.direct_viewport_products_active() {
            return None;
        }
        let Some(crate::core::play::WorldDomain::Play(instance)) =
            self.runtime.play_sessions().attached_world_domain()
        else {
            return None;
        };
        if self.play_gpu_unavailable == Some(instance) {
            return None;
        }
        match self
            .runtime
            .play_sessions()
            .preview_render_extract(self.viewport_size)
        {
            Ok(Some((instance, gateway, extract))) => {
                let surface_key = crate::core::play::PlayPreviewFrameIdentity::viewport_surface_key(
                    instance, &gateway,
                );
                self.play_gpu_identity = Some((instance, gateway));
                if let Err(error) =
                    self.viewport
                        .submit_play_extract(&surface_key, extract, self.viewport_size)
                {
                    self.set_status_line(error.to_string());
                }
                // Product polling owns publication; do not fall back to RGBA while it is pending.
                Some(false)
            }
            Ok(None) => {
                self.play_gpu_unavailable = Some(instance);
                None
            }
            Err(error) => {
                self.set_status_line(error.to_string());
                Some(false)
            }
        }
    }

    fn poll_cpu_preview_frame(
        &self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Option<Result<Option<PlayPreviewFrame>, PlayPreviewCaptureError>> {
        if self.runtime.async_play_enabled() {
            self.runtime.poll_native_preview_frame(size)
        } else {
            Some(self.capture_preview_frame_for_native_host(size))
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
