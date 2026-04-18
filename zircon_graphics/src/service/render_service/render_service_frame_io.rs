use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};

use super::super::render_thread::RenderThreadCommand;
use super::render_service::RenderService;

impl RenderService {
    pub fn submit_frame(&self, frame: EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.command_tx
            .send(RenderThreadCommand::Frame(frame))
            .map_err(|_| GraphicsError::Channel("render command receiver dropped".to_string()))
    }

    pub fn try_recv_latest_frame(&self) -> Option<ViewportFrame> {
        let mut latest = None;
        while let Ok(frame) = self.frame_rx.try_recv() {
            latest = Some(frame);
        }
        latest
    }
}
