use std::collections::HashMap;
use std::sync::Arc;

use zircon_asset::pipeline::manager::ProjectAssetManager;

use super::super::fallback::create_fallback_texture;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            asset_manager,
            models: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            fallback_texture: Arc::new(create_fallback_texture(device, queue, texture_layout)),
        }
    }
}
