use zircon_runtime::diagnostic_log::{write_log, write_warn};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn close_primary_window_after_request(&mut self) {
        self.disable_surface_present();
        self.presenter = None;
        self.window = None;
    }

    pub(super) fn disable_surface_present(&mut self) {
        if self.surface_present_enabled || self.surface_present_attempted {
            let _ = self.session.unbind_viewport_surface(self.viewport);
        }
        self.surface_present_enabled = false;
        self.surface_present_attempted = false;
    }

    pub(in crate::entry::runtime_entry_app) fn enable_surface_present(&mut self) {
        self.surface_present_enabled = true;
        self.surface_present_failed = false;
        write_log("runtime_surface_present", "runtime_surface_present_enabled");
    }

    pub(in crate::entry::runtime_entry_app) fn fallback_surface_present(&mut self) {
        self.disable_surface_present();
        write_log(
            "runtime_surface_present",
            "runtime_surface_present_fallback",
        );
    }

    pub(in crate::entry::runtime_entry_app) fn fail_surface_present(&mut self) {
        self.surface_present_failed = true;
        write_warn("runtime_surface_present", "runtime_surface_present_failed");
        self.fallback_surface_present();
    }
}

impl Drop for RuntimeEntryApp {
    fn drop(&mut self) {
        self.disable_surface_present();
    }
}
