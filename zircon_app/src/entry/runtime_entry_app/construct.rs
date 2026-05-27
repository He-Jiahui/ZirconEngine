use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::RuntimeEntryApp;
use super::RuntimeEntryAppConfig;
use crate::entry::runtime_library::{RuntimeLibraryError, RuntimeSession};

impl RuntimeEntryApp {
    pub(in crate::entry) fn new(session: RuntimeSession, config: RuntimeEntryAppConfig) -> Self {
        let window_descriptor = config.window_descriptor;
        let window_size = window_descriptor.resolution.physical_size();
        Self {
            window: None,
            window_descriptor,
            event_loop_policy: config.event_loop_policy,
            window_lifecycle_policy: config.window_lifecycle_policy,
            presenter: None,
            surface_present_enabled: false,
            surface_present_failed: false,
            surface_present_attempted: false,
            session,
            viewport: ZrRuntimeViewportHandle::new(1),
            viewport_size: ZrRuntimeViewportSizeV1::new(window_size.x, window_size.y),
            #[cfg(feature = "gamepad-gilrs")]
            gamepads: super::gamepad::create_gilrs(),
            #[cfg(feature = "gamepad-gilrs")]
            gamepad_connections_announced: false,
            #[cfg(feature = "gamepad-gilrs")]
            gamepad_rumble_effects: None,
        }
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
