use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::{RuntimeEntryApp, RuntimeEntryAppConfig, RuntimeEntryAppFailureState};
use crate::entry::runtime_library::{RuntimeLibraryError, RuntimeSession};

impl RuntimeEntryApp {
    pub(in crate::entry) fn new(
        session: RuntimeSession,
        config: RuntimeEntryAppConfig,
        failure_state: RuntimeEntryAppFailureState,
    ) -> Self {
        let window_descriptor = config.window_descriptor;
        let window_size = window_descriptor.resolution.physical_size();
        Self {
            window: None,
            window_descriptor,
            frame_cadence: super::event_loop_policy::RuntimeFrameCadence::new(
                config.event_loop_policy,
            ),
            window_lifecycle_policy: config.window_lifecycle_policy,
            presenter: None,
            surface_present_enabled: false,
            surface_present_failed: false,
            surface_present_attempted: false,
            exit_after_first_presented_frame: config.exit_after_first_presented_frame,
            first_frame_capture_path: config.first_frame_capture_path,
            require_persisted_scene_diagnostics: config.require_persisted_scene_diagnostics,
            first_frame_capture_written: false,
            first_frame_product_diagnostics_emitted: false,
            mvp_input_probe_submitted: false,
            failure_state,
            session,
            viewport: ZrRuntimeViewportHandle::new(1),
            viewport_size: ZrRuntimeViewportSizeV1::new(window_size.x, window_size.y),
            last_pointer_position: None,
            #[cfg(feature = "gamepad-gilrs")]
            gamepads: super::gamepad::create_gilrs(),
            #[cfg(feature = "gamepad-gilrs")]
            gamepad_connections_announced: false,
            #[cfg(feature = "gamepad-gilrs")]
            gamepad_rumble_effects: None,
        }
    }

    pub(super) fn report_fatal_failure(
        &self,
        component: &'static str,
        requested: impl std::fmt::Display,
        cause: impl std::fmt::Display,
        recovery: &'static str,
    ) {
        let failure =
            super::failure::RuntimeEntryAppFailure::new(component, requested, cause, recovery);
        zircon_runtime::diagnostic_log::write_error(component, failure.to_string());
        self.failure_state.record(failure);
    }

    pub(super) fn resize_viewport(
        &mut self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<(), RuntimeLibraryError> {
        let size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        self.viewport_size = size;
        self.session
            .handle_event(ZrRuntimeEventV1::viewport_resized(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                size,
            ))
    }
}
