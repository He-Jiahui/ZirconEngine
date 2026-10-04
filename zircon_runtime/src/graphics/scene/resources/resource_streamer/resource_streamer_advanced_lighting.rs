use std::sync::Arc;

use crate::core::framework::render::{RenderFrameSubmissionTransaction, RenderImageDescriptor};
use crate::core::resource::ResourceId;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::super::PostProcessLutTextureResource;
use super::ResourceStreamer;

/// 辐照度体积借用 LUT 上传资源的设备纹理；外部只拿稳定的 Arc 视图与描述符，
/// 具体上传仍参与 ResourceStreamer 的帧事务。
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
