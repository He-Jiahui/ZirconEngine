//! 窗口重绘事件中的呈现分派与成功帧后的产品验收动作。
//! 原生与 CPU 路径只在已成功呈现后共享捕获、诊断和退出计数；降级须显式选择。

use winit::event_loop::ActiveEventLoop;
use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime::diagnostic_log::write_log;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    /// 响应一次窗口重绘；原生表面优先，CPU 捕获仅用于显式降级诊断。
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
        if !self.ensure_reference_cpu_presenter(event_loop) {
            return;
        }
        let reference_cpu_result = if let Some(presenter) = self.presenter.as_mut() {
            let capture_started_at = std::time::Instant::now();
            zircon_runtime::profile_counter!(
                "app",
                "runtime_entry.reference_cpu_presenter.capture_request",
                1_u8
            );
            match self
                .session
                .capture_frame(self.viewport, self.viewport_size)
            {
                Ok(frame) => presenter.present(&frame, capture_started_at).map_err(|error| {
                        (
                            format!(
                                "viewport={:?} size={}x{} frame={}x{}",
                                self.viewport,
                                self.viewport_size.width,
                                self.viewport_size.height,
                                frame.width(),
                                frame.height()
                            ),
                            format!("reference CPU presentation failed: {error}"),
                            "verify the graphics adapter and window surface, then restart zircon_runtime",
                        )
                    }),
                Err(error) => Err((
                    format!(
                        "viewport={:?} size={}x{}",
                        self.viewport, self.viewport_size.width, self.viewport_size.height
                    ),
                    format!("reference CPU frame capture failed: {error}"),
                    "verify the graphics adapter and runtime project, then restart zircon_runtime",
                )),
            }
        } else {
            return;
        };
        match reference_cpu_result {
            Ok(()) => {
                zircon_runtime::profile_counter!(
                    "app",
                    "runtime_entry.reference_cpu_presenter.presented",
                    1_u8
                );
                self.complete_presented_frame(event_loop);
            }
            Err((context, error, recovery_hint)) => {
                self.record_reference_cpu_presenter_drop();
                self.report_fatal_failure("runtime_surface_present", context, error, recovery_hint);
                event_loop.exit();
            }
        }
    }
}

impl RuntimeEntryApp {
    fn record_reference_cpu_presenter_drop(&mut self) {
        if let Some(presenter) = self.presenter.as_mut() {
            presenter.record_dropped_frame();
        }
    }

    /// 仅在实际呈现成功后执行一次性捕获/诊断，再计入产品退出帧数。
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

    /// 显式请求的首帧 PNG 必须来自 Runtime 捕获；写入成功后才锁定一次性状态。
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
#[path = "tests/redraw.rs"]
mod tests;
