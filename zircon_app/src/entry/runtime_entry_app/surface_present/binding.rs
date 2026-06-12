use winit::window::Window;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{window_surface::runtime_native_surface_target, RuntimeEntryApp};
use crate::entry::runtime_library::RuntimeLibraryError;

const FORCE_CAPTURE_PRESENT_ENV: &str = "ZR_RUNTIME_FORCE_CAPTURE_PRESENT";

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn bind_current_window_surface(
        &mut self,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(window) = self.window.clone() else {
            return Ok(false);
        };
        self.bind_window_surface(window.as_ref())
    }

    pub(in crate::entry::runtime_entry_app) fn bind_window_surface(
        &mut self,
        window: &dyn Window,
    ) -> Result<bool, RuntimeLibraryError> {
        if force_capture_present() {
            write_log(
                "runtime_surface_present",
                "runtime_force_capture_present_enabled",
            );
            return Ok(false);
        }
        if !self.session.supports_viewport_surface_present() {
            return Ok(false);
        }
        let Some(target) = runtime_native_surface_target(window) else {
            return Ok(false);
        };
        self.surface_present_attempted = true;
        self.session
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                self.viewport_size,
                target,
            ))
    }
}

fn force_capture_present() -> bool {
    std::env::var(FORCE_CAPTURE_PRESENT_ENV).is_ok_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
    })
}
