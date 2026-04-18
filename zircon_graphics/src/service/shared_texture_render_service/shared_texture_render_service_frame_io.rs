use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrameTextureHandle};

use super::super::render_thread::RenderThreadCommand;
use super::shared_texture_render_service::SharedTextureRenderService;

impl SharedTextureRenderService {
    pub fn submit_frame(&self, frame: EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.command_tx
            .send(RenderThreadCommand::Frame(frame))
            .map_err(|_| GraphicsError::Channel("render command receiver dropped".to_string()))
    }

    pub fn try_recv_latest_frame(&self) -> Option<ViewportFrameTextureHandle> {
        let mut latest = None;
        while let Ok(frame) = self.frame_rx.try_recv() {
            latest = Some(frame);
        }
        latest
    }
}
