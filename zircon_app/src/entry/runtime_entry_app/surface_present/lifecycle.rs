use zircon_runtime::diagnostic_log::{write_log, write_warn};

use super::super::RuntimeEntryApp;
use crate::entry::runtime_library::RuntimeLibraryError;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn close_primary_window_after_request(&mut self) {
        self.teardown_surface_present();
        self.presenter = None;
        self.window = None;
    }

    pub(super) fn disable_surface_present(&mut self) {
        if let Err(error) = self.release_surface_present() {
            write_warn(
                "runtime_surface_present",
                format!("runtime_product_teardown surface_unbind=failed error={error}"),
            );
        }
    }

    fn teardown_surface_present(&mut self) {
        if let Err(error) = self.release_surface_present() {
            self.report_fatal_failure(
                "runtime_surface_present",
                format!("viewport={:?}", self.viewport),
                format!("runtime surface unbind failed: {error}"),
                "verify the runtime surface lifecycle and restart zircon_runtime",
            );
        }
    }

    fn release_surface_present(&mut self) -> Result<(), RuntimeLibraryError> {
        let mut release_result = Ok(());
        if self.surface_present_enabled || self.surface_present_attempted {
            match self.session.unbind_viewport_surface(self.viewport) {
                Ok(true) => write_log(
                    "runtime_surface_present",
                    "runtime_product_teardown surface_unbind=ok",
                ),
                Ok(false) => write_log(
                    "runtime_surface_present",
                    "runtime_product_teardown surface_unbind=unavailable",
                ),
                Err(error) => release_result = Err(error),
            }
        }
        self.surface_present_enabled = false;
        self.surface_present_attempted = false;
        release_result
    }

    pub(in crate::entry::runtime_entry_app) fn enable_surface_present(&mut self) {
        self.surface_present_enabled = true;
        write_log("runtime_surface_present", "runtime_surface_present_enabled");
    }

    pub(in crate::entry::runtime_entry_app) fn fallback_surface_present(&mut self) {
        self.disable_surface_present();
        write_log(
            "runtime_surface_present",
            "runtime_surface_present_fallback",
        );
    }
}

impl Drop for RuntimeEntryApp {
    fn drop(&mut self) {
        let cadence = self.frame_cadence.report();
        write_log(
            "runtime_frame_cadence",
            format!(
                "runtime_frame_cadence_summary policy={} request_attempts={} requests_accepted={} requests_coalesced={} requests_ignored={} pumps={} idle_suppressed={} redraw_requests={} focus_transitions={} occlusion_transitions={} low_power_pumps={} low_power_suppressed={}",
                self.frame_cadence.policy().as_str(),
                cadence.frame_requests,
                cadence.frame_requests_accepted,
                cadence.frame_requests_coalesced,
                cadence.frame_requests_ignored,
                cadence.frame_pumps,
                cadence.idle_pumps_suppressed,
                cadence.redraw_requests,
                cadence.focus_transitions,
                cadence.occlusion_transitions,
                cadence.low_power_pumps,
                cadence.low_power_pumps_suppressed,
            ),
        );
        #[cfg(feature = "gamepad-gilrs")]
        super::super::gamepad::clear_gamepad_rumble_effects(&mut self.gamepad_rumble_effects);
        self.teardown_surface_present();
    }
}
