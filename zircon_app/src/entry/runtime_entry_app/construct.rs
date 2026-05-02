use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::RuntimeEntryApp;
use crate::entry::runtime_library::{RuntimeLibraryError, RuntimeSession};

impl RuntimeEntryApp {
    pub(in crate::entry) fn new(session: RuntimeSession) -> Self {
        Self {
            window: None,
            presenter: None,
            session,
            viewport: ZrRuntimeViewportHandle::new(1),
            viewport_size: ZrRuntimeViewportSizeV1::new(1280, 720),
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
