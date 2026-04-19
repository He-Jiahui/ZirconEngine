use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::rhi::RenderDevice;
use crate::rhi_wgpu::WgpuRenderDevice;

use crate::{GraphicsError, SceneRenderer};

use super::super::capability_summary::capability_summary;
use super::super::render_framework_state::RenderFrameworkState;
use super::super::wgpu_render_framework::WgpuRenderFramework;
use super::create_default_pipelines::create_default_pipelines;

impl WgpuRenderFramework {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        let render_device = WgpuRenderDevice::new_headless();
        Ok(Self {
            state: Mutex::new(RenderFrameworkState {
                renderer: SceneRenderer::new(asset_manager)?,
                next_viewport_id: 1,
                next_history_id: 1,
                pipelines: create_default_pipelines(),
                viewports: HashMap::new(),
                stats: crate::core::framework::render::RenderStats {
                    capabilities: capability_summary(render_device.caps()),
                    ..crate::core::framework::render::RenderStats::default()
                },
            }),
        })
    }
}
