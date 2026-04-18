use crossbeam_channel::{Receiver, Sender};
use std::thread::JoinHandle;

use crate::types::ViewportFrame;

use super::super::render_thread::RenderThreadCommand;

pub struct RenderService {
    pub(in crate::service) command_tx: Sender<RenderThreadCommand>,
    pub(in crate::service) frame_rx: Receiver<ViewportFrame>,
    pub(in crate::service) join: Option<JoinHandle<()>>,
}
