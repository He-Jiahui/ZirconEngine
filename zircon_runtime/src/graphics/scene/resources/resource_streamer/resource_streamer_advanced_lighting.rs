use std::sync::Arc;

use crate::core::framework::render::RenderImageDescriptor;
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

use super::super::PostProcessLutTextureResource;
use super::ResourceStreamer;

#[derive(Clone)]
pub(crate) struct IrradianceVolumeTextureBinding {
    resource: Arc<PostProcessLutTextureResource>,
}

impl IrradianceVolumeTextureBinding {
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        self.resource.view()
    }

    pub(crate) fn descriptor(&self) -> &RenderImageDescriptor {
        &self.resource.descriptor
    }
}

impl ResourceStreamer {
    pub(crate) fn ensure_irradiance_volume_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        self.ensure_post_process_lut_texture(device, queue, id)
    }

    pub(crate) fn irradiance_volume_texture(
        &self,
        id: ResourceId,
    ) -> Option<IrradianceVolumeTextureBinding> {
        self.post_process_lut_textures
            .get(&id)
            .map(|prepared| IrradianceVolumeTextureBinding {
                resource: Arc::clone(&prepared.resource),
            })
    }
}
