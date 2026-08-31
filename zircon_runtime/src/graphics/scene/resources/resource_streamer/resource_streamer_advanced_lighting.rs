use std::sync::Arc;

use crate::core::framework::render::{RenderFrameSubmissionTransaction, RenderImageDescriptor};
use crate::core::resource::ResourceId;
use crate::graphics::backend::RenderBackend;
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
        backend: &RenderBackend,
        id: ResourceId,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        self.ensure_post_process_lut_texture(backend, id, submission_transaction)
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
