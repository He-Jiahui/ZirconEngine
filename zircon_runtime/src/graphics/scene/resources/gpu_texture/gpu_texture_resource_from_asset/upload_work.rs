use zr_rhi_wgpu::WgpuTextureUploadBatch;

use super::super::GpuTextureResource;

/// Deferred GPU work created while materializing one texture asset.
///
/// The resource is ready for bindings immediately, but its uploads and mip commands enter the
/// backend-owned coordinator before any frame packet that consumes those bindings is flushed.
pub(in crate::graphics::scene::resources) struct GpuTextureUploadWork {
    pub(in crate::graphics::scene::resources) resource: GpuTextureResource,
    pub(in crate::graphics::scene::resources) upload_batch: WgpuTextureUploadBatch,
    pub(in crate::graphics::scene::resources) pre_upload_commands: Vec<wgpu::CommandBuffer>,
    pub(in crate::graphics::scene::resources) post_upload_commands: Vec<wgpu::CommandBuffer>,
}

impl GpuTextureUploadWork {
    pub(super) fn new(
        resource: GpuTextureResource,
        upload_batch: WgpuTextureUploadBatch,
        pre_upload_commands: Vec<wgpu::CommandBuffer>,
        post_upload_commands: Vec<wgpu::CommandBuffer>,
    ) -> Self {
        Self {
            resource,
            upload_batch,
            pre_upload_commands,
            post_upload_commands,
        }
    }
}
