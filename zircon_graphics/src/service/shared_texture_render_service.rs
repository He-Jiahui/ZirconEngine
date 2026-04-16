use crossbeam_channel::{Receiver, Sender};
use std::thread::JoinHandle;

use crate::types::ViewportFrameTextureHandle;

use super::render_thread_command::RenderThreadCommand;

pub struct SharedTextureRenderService {
    pub(in crate::service) command_tx: Sender<RenderThreadCommand>,
    pub(in crate::service) frame_rx: Receiver<ViewportFrameTextureHandle>,
    pub(in crate::service) join: Option<JoinHandle<()>>,
}
