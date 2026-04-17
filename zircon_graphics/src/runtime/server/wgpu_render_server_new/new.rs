use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_asset::ProjectAssetManager;
use zircon_rhi::RenderDevice;
use zircon_rhi_wgpu::WgpuRenderDevice;

use crate::{GraphicsError, SceneRenderer};

use super::super::capability_summary::capability_summary;
use super::super::render_server_state::RenderServerState;
use super::super::wgpu_render_server::WgpuRenderServer;
use super::create_default_pipelines::create_default_pipelines;

impl WgpuRenderServer {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        let render_device = WgpuRenderDevice::new_headless();
        Ok(Self {
            state: Mutex::new(RenderServerState {
                renderer: SceneRenderer::new(asset_manager)?,
                next_viewport_id: 1,
                next_history_id: 1,
                pipelines: create_default_pipelines(),
                viewports: HashMap::new(),
                stats: zircon_render_server::RenderStats {
                    capabilities: capability_summary(render_device.caps()),
                    ..zircon_render_server::RenderStats::default()
                },
            }),
        })
    }
}
