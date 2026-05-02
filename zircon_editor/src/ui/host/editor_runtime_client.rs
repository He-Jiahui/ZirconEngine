use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeFrameV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

/// Editor-owned handle to a dynamically loaded runtime session.
pub trait EditorRuntimeClient {
    fn session_handle(&self) -> ZrRuntimeSessionHandle;

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), String>;

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, String>;
}

pub type SharedEditorRuntimeClient = Arc<dyn EditorRuntimeClient>;

#[derive(Debug, Default)]
pub struct DetachedEditorRuntimeClient;

impl EditorRuntimeClient for DetachedEditorRuntimeClient {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn handle_event(&self, _event: ZrRuntimeEventV1) -> Result<(), String> {
        Err("editor runtime client is not attached".to_string())
    }

    fn capture_frame(
        &self,
        _viewport: ZrRuntimeViewportHandle,
        _size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, String> {
        Err("editor runtime client is not attached".to_string())
    }
}
