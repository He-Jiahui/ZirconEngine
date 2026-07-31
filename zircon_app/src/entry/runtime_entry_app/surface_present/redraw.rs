use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::{write_log, write_warn};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn present_redraw_frame(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        zircon_runtime::profile_frame!("app", "runtime_redraw");
        zircon_runtime::profile_scope!("app", "runtime_entry", "redraw_requested");
        if self.surface_present_enabled && !self.surface_present_failed {
            match self
                .session
                .present_viewport(self.viewport, self.viewport_size)
            {
                Ok(true) => {
                    self.complete_first_presented_frame(event_loop);
                    return;
                }
                Ok(false) => {
                    write_warn(
                        "runtime_surface_present",
                        format!(
                            "runtime_surface_present_returned_false viewport={:?} size={}x{}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                    );
                    self.fail_surface_present();
                }
                Err(error) => {
                    write_warn(
                        "runtime_surface_present",
                        format!(
                            "runtime_surface_present_error viewport={:?} size={}x{} error={error}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                    );
                    self.fail_surface_present();
                }
            }
        }
        if !self.ensure_fallback_presenter(event_loop) {
            return;
        }
        if let Some(presenter) = self.presenter.as_mut() {
            match self
                .session
                .capture_frame(self.viewport, self.viewport_size)
            {
                Ok(frame) => {
                    if let Err(error) = presenter.present(&frame) {
                        self.report_fatal_failure(
                            "runtime_surface_present",
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
                        );
                        event_loop.exit();
                    } else {
                        self.complete_first_presented_frame(event_loop);
                    }
                }
                Err(error) => {
                    self.report_fatal_failure(
                        "runtime_surface_present",
                        format!(
                            "viewport={:?} size={}x{}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                        format!("frame capture failed: {error}"),
                        "verify the graphics adapter and runtime project, then restart zircon_runtime",
                    );
                    event_loop.exit();
                }
            }
        }
    }
}

impl RuntimeEntryApp {
    fn complete_first_presented_frame(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Err(error) = self.capture_first_presented_frame_if_requested() {
            self.report_fatal_failure(
                "runtime_frame_capture",
                self.first_frame_capture_path
                    .as_deref()
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
        if let Some(diagnostic) =
            first_presented_frame_diagnostic(self.exit_after_first_presented_frame)
        {
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
            self.first_frame_capture_path.as_deref(),
            self.first_frame_capture_written,
        ) {
            return Ok(());
        }
        let Some(path) = self.first_frame_capture_path.clone() else {
            return Ok(());
        };
        let frame = self
            .session
            .capture_frame(self.viewport, self.viewport_size)
            .map_err(|error| format!("capture runtime frame: {error}"))?;
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
                path.display(),
                frame.width(),
                frame.height()
            ),
        );
        Ok(())
    }
}

fn first_presented_frame_diagnostic(enabled: bool) -> Option<&'static str> {
    enabled.then_some("runtime_first_frame_presented")
}

fn should_emit_first_frame_product_diagnostics(emitted: bool) -> bool {
    !emitted
}

fn should_capture_first_presented_frame(path: Option<&std::path::Path>, written: bool) -> bool {
    path.is_some() && !written
}

#[cfg(test)]
mod tests {
    use super::{
        first_presented_frame_diagnostic, should_capture_first_presented_frame,
        should_emit_first_frame_product_diagnostics,
    };

    #[test]
    fn first_frame_exit_emits_a_presented_frame_diagnostic() {
        assert_eq!(
            first_presented_frame_diagnostic(true),
            Some("runtime_first_frame_presented")
        );
    }

    #[test]
    fn continuous_runtime_records_product_diagnostics_without_requesting_exit() {
        assert_eq!(first_presented_frame_diagnostic(false), None);
        assert!(should_emit_first_frame_product_diagnostics(false));
    }

    #[test]
    fn product_frame_diagnostics_are_not_repeated_after_the_first_present() {
        assert!(!should_emit_first_frame_product_diagnostics(true));
    }

    #[test]
    fn requested_first_frame_capture_runs_once_after_a_presented_frame() {
        let path = std::path::Path::new("E:/evidence/runtime-first-frame.png");

        assert!(should_capture_first_presented_frame(Some(path), false));
        assert!(!should_capture_first_presented_frame(Some(path), true));
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
}
