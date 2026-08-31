use std::path::Path;

use zircon_runtime::core::runtime::diagnostics::profiling;

/// Owns the optional core profiling capture started by the manual product test.
pub(super) struct RealtimeIblCpuProfileCapture {
    owns_capture: bool,
    stopped: bool,
}

impl RealtimeIblCpuProfileCapture {
    pub(super) fn feature_enabled() -> bool {
        profiling::feature_enabled()
    }

    pub(super) fn begin() -> Self {
        if profiling::capture_active() {
            return Self {
                owns_capture: false,
                stopped: true,
            };
        }
        let owns_capture =
            profiling::start_capture(profiling::ProfileCaptureConfig::default()).active;
        Self {
            owns_capture,
            stopped: false,
        }
    }

    pub(super) const fn has_owned_capture(&self) -> bool {
        self.owns_capture
    }

    pub(super) fn stop(&mut self) {
        if self.owns_capture && !self.stopped {
            profiling::stop_capture();
            self.stopped = true;
        }
    }
}

impl Drop for RealtimeIblCpuProfileCapture {
    fn drop(&mut self) {
        self.stop();
        if self.owns_capture {
            profiling::reset_capture();
        }
    }
}

pub(super) fn clear_current_cpu_timing_sidecar(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("remove stale realtime IBL CPU timing report: {error}"),
    }
}
