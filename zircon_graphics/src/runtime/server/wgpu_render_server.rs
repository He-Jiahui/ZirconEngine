use std::sync::Mutex;

use super::render_server_state::RenderServerState;

pub struct WgpuRenderServer {
    pub(in crate::runtime::server) state: Mutex<RenderServerState>,
}
