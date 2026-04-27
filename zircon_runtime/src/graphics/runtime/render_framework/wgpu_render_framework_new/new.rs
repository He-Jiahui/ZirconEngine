use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::RenderFeatureDescriptor;
use crate::rhi::RenderDevice;
use crate::rhi_wgpu::WgpuRenderDevice;

use crate::{GraphicsError, SceneRenderer};

use super::super::capability_summary::capability_summary;
use super::super::render_framework_state::RenderFrameworkState;
use super::super::wgpu_render_framework::WgpuRenderFramework;
use super::create_default_pipelines::create_default_pipelines;

impl WgpuRenderFramework {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_features(asset_manager, Vec::new())
    }

    pub fn new_with_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Result<Self, GraphicsError> {
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let render_device = WgpuRenderDevice::new_headless();
        Ok(Self {
            state: Mutex::new(RenderFrameworkState {
                renderer: SceneRenderer::new_with_plugin_render_features(
                    asset_manager,
                    render_features.clone(),
                )?,
                next_viewport_id: 1,
                next_history_id: 1,
                pipelines: create_default_pipelines(&render_features),
                viewports: HashMap::new(),
                stats: crate::core::framework::render::RenderStats {
                    capabilities: capability_summary(render_device.caps()),
                    ..crate::core::framework::render::RenderStats::default()
                },
            }),
        })
    }
}
