use winit::event_loop::ActiveEventLoop;
use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime::diagnostic_log::write_log;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn present_redraw_frame(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        zircon_runtime::profile_frame!("app", "runtime_redraw");
        zircon_runtime::profile_scope!("app", "runtime_entry", "redraw_requested");
        if self.surface_present_enabled {
            match self
                .session
                .present_viewport(self.viewport, self.viewport_size)
            {
                Ok(true) => {
                    zircon_runtime::profile_counter!("app", "runtime_entry.native_present", 1_u8);
                    self.complete_presented_frame(event_loop);
                    return;
                }
                Ok(false) => {
                    self.report_fatal_failure(
                        "runtime_surface_present",
                        format!(
                            "viewport={:?} size={}x{}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                        "native surface presentation returned unavailable after a successful bind",
                        "verify the runtime surface contract and restart zircon_runtime",
                    );
                    event_loop.exit();
                    return;
                }
                Err(error) => {
                    self.report_fatal_failure(
                        "runtime_surface_present",
                        format!(
                            "viewport={:?} size={}x{}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                        format!("native surface presentation failed: {error}"),
                        "verify the graphics adapter and window surface, then restart zircon_runtime",
                    );
                    event_loop.exit();
                    return;
                }
            }
        }
        if !self.ensure_fallback_presenter(event_loop) {
            return;
        }
        let fallback_result = if let Some(presenter) = self.presenter.as_mut() {
            zircon_runtime::profile_counter!("app", "runtime_entry.fallback_capture_request", 1_u8);
            match self
                .session
                .capture_frame(self.viewport, self.viewport_size)
            {
                Ok(frame) => {
                    zircon_runtime::profile_counter!(
                        "app",
                        "runtime_entry.fallback_rgba_bytes",
                        frame.rgba().len()
                    );
                    presenter.present(&frame).map_err(|error| {
                        (
                            format!(
                                "viewport={:?} size={}x{} frame={}x{}",
                                self.viewport,
                                self.viewport_size.width,
                                self.viewport_size.height,
                                frame.width(),
                                frame.height()
                            ),
                            format!("fallback presentation failed: {error}"),
                            "verify the graphics adapter and window surface, then restart zircon_runtime",
                        )
                    })
                }
                Err(error) => Err((
                    format!(
                        "viewport={:?} size={}x{}",
                        self.viewport, self.viewport_size.width, self.viewport_size.height
                    ),
                    format!("frame capture failed: {error}"),
                    "verify the graphics adapter and runtime project, then restart zircon_runtime",
                )),
            }
        } else {
            return;
        };
        match fallback_result {
            Ok(()) => {
                zircon_runtime::profile_counter!("app", "runtime_entry.fallback_cpu_present", 1_u8);
                self.complete_presented_frame(event_loop);
            }
            Err((context, error, recovery_hint)) => {
                self.report_fatal_failure("runtime_surface_present", context, error, recovery_hint);
                event_loop.exit();
            }
        }
    }
}

impl RuntimeEntryApp {
    fn complete_presented_frame(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_counter!("app", "runtime_entry.presented_frame", 1_u8);
        if let Err(error) = self.capture_first_presented_frame_if_requested() {
            self.report_fatal_failure(
                "runtime_frame_capture",
                self.first_frame_capture_path
                    .as_ref()
                    .map(ResolvedProjectPath::display_path)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<not-requested>".to_owned()),
                format!("first presented frame capture failed: {error}"),
                "choose a writable PNG capture path and verify the runtime can capture an RGBA frame before retrying zircon_runtime",
            );
            event_loop.exit();
            return;
        }
        if self.require_persisted_scene_diagnostics {
            if let Err(error) = self.emit_first_frame_product_diagnostics_once() {
                self.report_fatal_failure(
                    "runtime_product_diagnostics",
                    format!(
                        "viewport={:?} size={}x{}",
                        self.viewport, self.viewport_size.width, self.viewport_size.height
                    ),
                    error,
                    "verify the loaded runtime supports diagnostics and the F2 scene renders before retrying zircon_runtime",
                );
                event_loop.exit();
                return;
            }
        }
        self.presented_frame_count = self.presented_frame_count.saturating_add(1);
        if let Some(diagnostic) = presented_frame_exit_diagnostic(
            self.presented_frame_count,
            self.exit_after_presented_frames,
        ) {
            write_log("runtime_surface_present", diagnostic);
            event_loop.exit();
        }
    }

    fn emit_first_frame_product_diagnostics_once(&mut self) -> Result<(), String> {
        if should_emit_first_frame_product_diagnostics(self.first_frame_product_diagnostics_emitted)
        {
            self.emit_first_frame_product_diagnostics()?;
            self.first_frame_product_diagnostics_emitted = true;
        }
        Ok(())
    }

    fn capture_first_presented_frame_if_requested(&mut self) -> Result<(), String> {
        if !should_capture_first_presented_frame(
            self.first_frame_capture_path.as_ref(),
            self.first_frame_capture_written,
        ) {
            return Ok(());
        }
        let Some(path) = self.first_frame_capture_path.clone() else {
            return Ok(());
        };
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.explicit_frame_capture_request",
            1_u8
        );
        let frame = self
            .session
            .capture_frame(self.viewport, self.viewport_size)
            .map_err(|error| format!("capture runtime frame: {error}"))?;
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.explicit_frame_capture_rgba_bytes",
            frame.rgba().len()
        );
        super::super::frame_capture::write_runtime_frame_png(
            &path,
            frame.width(),
            frame.height(),
            frame.rgba(),
        )?;
        self.first_frame_capture_written = true;
        write_log(
            "runtime_surface_present",
            format!(
                "runtime_product_frame_capture_written path={} frame={}x{}",
                path.display_path().display(),
                frame.width(),
                frame.height()
            ),
        );
        Ok(())
    }
}

fn presented_frame_exit_diagnostic(
    presented_frame_count: u64,
    limit: Option<std::num::NonZeroU64>,
) -> Option<String> {
    let limit = limit?;
    (presented_frame_count >= limit.get()).then(|| {
        if limit == std::num::NonZeroU64::MIN {
            "runtime_first_frame_presented".to_string()
        } else {
            format!(
                "runtime_presented_frame_limit_reached limit={} count={presented_frame_count}",
                limit
            )
        }
    })
}

fn should_emit_first_frame_product_diagnostics(emitted: bool) -> bool {
    !emitted
}

fn should_capture_first_presented_frame(path: Option<&ResolvedProjectPath>, written: bool) -> bool {
    path.is_some() && !written
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use zircon_runtime::asset::project::ProjectPaths;

    use super::{
        presented_frame_exit_diagnostic, should_capture_first_presented_frame,
        should_emit_first_frame_product_diagnostics,
    };

    #[test]
    fn first_frame_exit_emits_a_presented_frame_diagnostic() {
        assert_eq!(
            presented_frame_exit_diagnostic(1, Some(NonZeroU64::MIN)),
            Some("runtime_first_frame_presented".to_string())
        );
    }

    #[test]
    fn presented_frame_exit_waits_for_the_configured_successful_present_count() {
        let limit = NonZeroU64::new(120).unwrap();

        assert_eq!(presented_frame_exit_diagnostic(119, Some(limit)), None);
        assert_eq!(
            presented_frame_exit_diagnostic(120, Some(limit)),
            Some("runtime_presented_frame_limit_reached limit=120 count=120".to_string())
        );
        assert_eq!(presented_frame_exit_diagnostic(120, None), None);
        assert!(should_emit_first_frame_product_diagnostics(false));
    }

    #[test]
    fn product_frame_diagnostics_are_not_repeated_after_the_first_present() {
        assert!(!should_emit_first_frame_product_diagnostics(true));
    }

    #[test]
    fn requested_first_frame_capture_runs_once_after_a_presented_frame() {
        let path = ProjectPaths::resolve_path("E:/evidence/runtime-first-frame.png")
            .expect("capture path should resolve");

        assert!(should_capture_first_presented_frame(Some(&path), false));
        assert!(!should_capture_first_presented_frame(Some(&path), true));
        assert!(!should_capture_first_presented_frame(None, false));
    }

    #[test]
    fn frame_capture_projects_to_the_runtime_entry_root_sibling() {
        let source = include_str!("redraw.rs");
        let writer = ["frame_capture::", "write_runtime_frame_png"].concat();
        let root_sibling = ["super::super::", writer.as_str()].concat();
        let expected_call = format!("        {root_sibling}(");
        let calls = source
            .lines()
            .filter(|line| line.contains(&writer))
            .collect::<Vec<_>>();

        assert_eq!(calls, vec![expected_call.as_str()]);
    }

    #[test]
    fn present_paths_keep_the_p1_capture_and_present_measurement_points() {
        let source = include_str!("redraw.rs");

        for name in [
            "runtime_entry.native_present",
            "runtime_entry.fallback_capture_request",
            "runtime_entry.fallback_rgba_bytes",
            "runtime_entry.fallback_cpu_present",
            "runtime_entry.presented_frame",
            "runtime_entry.explicit_frame_capture_request",
            "runtime_entry.explicit_frame_capture_rgba_bytes",
        ] {
            assert!(
                source.contains(name),
                "P1 presentation reporting must retain the `{name}` counter"
            );
        }
    }
}
